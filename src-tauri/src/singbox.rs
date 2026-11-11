use crate::core_manager::{CoreManager, LiveStats, RunMode, ServerConfig};
use crate::elevate;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

const CLASH_API_BASE: &str = "http://127.0.0.1:9090";

pub struct SingBoxCore {
    binary_path: PathBuf,
    config_dir: PathBuf,
    child: Arc<Mutex<Option<Child>>>,
    connected: Arc<AtomicBool>,
    http_client: reqwest::Client,
}

impl SingBoxCore {
    pub fn new(binary_path: PathBuf, config_dir: PathBuf) -> Self {
        Self {
            binary_path,
            config_dir,
            child: Arc::new(Mutex::new(None)),
            connected: Arc::new(AtomicBool::new(false)),
            http_client: reqwest::Client::new(),
        }
    }

    fn config_path(&self) -> PathBuf {
        self.config_dir.join("singbox-active.json")
    }

    /// Builds a minimal sing-box config: one outbound for the chosen
    /// server, a Clash-API endpoint for stats, and either a tun or a
    /// mixed (http+socks) inbound depending on the selected mode.
    fn build_config(&self, server: &ServerConfig, mode: RunMode) -> serde_json::Value {
        let inbound = match mode {
            RunMode::Tun => json!({
                "type": "tun",
                "interface_name": "nebula-tun0",
                "inet4_address": "172.19.0.1/30",
                "auto_route": true,
                "strict_route": true,
                "stack": "system"
            }),
            RunMode::Proxy => json!({
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": 2080
            }),
        };

        json!({
            "log": { "level": "info" },
            "experimental": {
                "clash_api": {
                    "external_controller": "127.0.0.1:9090",
                    "external_ui": ""
                }
            },
            "inbounds": [inbound],
            "outbounds": [
                server.raw_outbound.clone(),
                { "type": "direct", "tag": "direct" }
            ],
            "route": {
                "auto_detect_interface": true,
                "final": server.raw_outbound.get("tag").cloned().unwrap_or(json!("proxy"))
            }
        })
    }
}

#[async_trait]
impl CoreManager for SingBoxCore {
    fn name(&self) -> &'static str {
        "sing-box"
    }

    async fn connect(&self, server: &ServerConfig, mode: RunMode) -> Result<()> {
        // make sure any previous instance is gone before starting a new one
        self.disconnect().await.ok();

        let config = self.build_config(server, mode);
        tokio::fs::create_dir_all(&self.config_dir).await?;
        tokio::fs::write(self.config_path(), serde_json::to_vec_pretty(&config)?).await?;

        // TUN mode needs root/admin on every platform (WinTun driver must
        // also be installed on Windows for the tun inbound to come up at
        // all); Proxy mode only opens local HTTP/SOCKS listeners and never
        // elevates. See elevate.rs for exactly how each OS is handled, and
        // its own caveats about what could not be exercised from here.
        let elevated = matches!(mode, RunMode::Tun);
        let args = vec!["run".to_string(), "-c".to_string(), self.config_path().display().to_string()];
        let child = elevate::spawn(&self.binary_path, &args, elevated)
            .with_context(|| format!("failed to spawn {:?} (elevated: {elevated})", self.binary_path))?;

        *self.child.lock().await = Some(child);
        self.connected.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
        }
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn stats(&self) -> Result<LiveStats> {
        if !self.connected.load(Ordering::SeqCst) {
            return Ok(LiveStats::default());
        }
        // sing-box's Clash-compatible API streams traffic over a websocket
        // at /traffic; a production client should keep that socket open
        // rather than polling. This does a cheap one-shot read instead so
        // the abstraction stays simple here.
        let resp = self
            .http_client
            .get(format!("{CLASH_API_BASE}/traffic"))
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let body: serde_json::Value = r.json().await.unwrap_or_default();
                Ok(LiveStats {
                    down_bytes_per_sec: body.get("down").and_then(|v| v.as_u64()).unwrap_or(0),
                    up_bytes_per_sec: body.get("up").and_then(|v| v.as_u64()).unwrap_or(0),
                    ping_ms: None,
                })
            }
            _ => Ok(LiveStats::default()),
        }
    }

    async fn measure_ping(&self, server: &ServerConfig) -> Result<u32> {
        let start = std::time::Instant::now();
        let target = format!("{}:{}", server.address, server.port);
        match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            tokio::net::TcpStream::connect(&target),
        )
        .await
        {
            Ok(Ok(_)) => Ok(start.elapsed().as_millis() as u32),
            Ok(Err(e)) => bail!("tcp connect failed for {target}: {e}"),
            Err(_) => bail!("ping timed out for {target}"),
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}
