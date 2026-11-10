use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub id: String,
    pub protocol: String, // "vless-reality" | "vmess" | "trojan" | "hysteria2" | "shadowsocks" | "xhttp-reality"
    pub address: String,
    pub port: u16,
    pub raw_outbound: serde_json::Value, // protocol-specific outbound block, already parsed from the subscription link
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunMode {
    Tun,
    Proxy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LiveStats {
    pub down_bytes_per_sec: u64,
    pub up_bytes_per_sec: u64,
    pub ping_ms: Option<u32>,
}

/// Every proxy engine (sing-box today, Xray-core later) implements this so
/// the Tauri command layer and the UI never need to know which binary is
/// actually running underneath.
#[async_trait]
pub trait CoreManager: Send + Sync {
    /// Human-readable id, e.g. "sing-box" or "xray".
    fn name(&self) -> &'static str;

    /// Write the engine-specific config to disk and spawn the child process
    /// in the requested mode (TUN needs elevated/system VPN permissions on
    /// every platform; Proxy only opens local HTTP/SOCKS listeners).
    async fn connect(&self, server: &ServerConfig, mode: RunMode) -> anyhow::Result<()>;

    /// Terminate the child process and tear down any TUN interface.
    async fn disconnect(&self) -> anyhow::Result<()>;

    /// Pull current throughput/latency, typically via the engine's local
    /// stats API (sing-box exposes a Clash-compatible API for this).
    async fn stats(&self) -> anyhow::Result<LiveStats>;

    /// One-shot latency probe against a candidate server, used by the
    /// sidebar's "Auto" button and per-server ping refresh.
    async fn measure_ping(&self, server: &ServerConfig) -> anyhow::Result<u32>;

    fn is_connected(&self) -> bool;
}
