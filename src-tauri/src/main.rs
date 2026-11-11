#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core_manager;
mod elevate;
mod geo;
mod singbox;
mod store;
mod subscription;
mod tray;

use core_manager::{CoreManager, LiveStats, RunMode, ServerConfig};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use singbox::SingBoxCore;
use std::sync::Arc;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDto {
    pub id: String,
    pub name: String,
    pub url: String,
    pub collapsed: bool,
    pub servers: Vec<ServerDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDto {
    pub id: String,
    pub country: String,
    pub flag: String,
    pub protocol: String,
    pub ping: u32,
    pub lat: f64,
    pub lng: f64,
    pub address: String,
    pub port: u16,
    pub raw_outbound: serde_json::Value,
}

/// Shared app state handed to every Tauri command. `core` is behind a
/// trait object so swapping sing-box for Xray-core later is a one-line
/// change here rather than a rewrite of the command layer.
struct AppState {
    core: Arc<dyn CoreManager>,
    config_dir: std::path::PathBuf,
    subscriptions: tokio::sync::Mutex<Vec<SubscriptionDto>>,
    settings: tokio::sync::Mutex<serde_json::Value>,
}

#[tauri::command]
async fn list_subscriptions(state: tauri::State<'_, AppState>) -> Result<Vec<SubscriptionDto>, String> {
    Ok(state.subscriptions.lock().await.clone())
}

#[tauri::command]
async fn add_subscription(state: tauri::State<'_, AppState>, url: String) -> Result<SubscriptionDto, String> {
    let links = subscription::fetch_links(&url).await.map_err(|e| e.to_string())?;
    if links.is_empty() {
        return Err("подписка не вернула ни одной валидной ссылки (vless/vmess/trojan/hysteria2/ss)".into());
    }

    let mut servers = Vec::new();
    for link in &links {
        match subscription::parse_link(link) {
            Ok(mut parsed) => {
                subscription::attach_coordinates(&mut parsed);
                servers.push(ServerDto {
                    id: parsed.id,
                    country: parsed.country,
                    flag: parsed.flag,
                    protocol: parsed.protocol,
                    ping: parsed.ping,
                    lat: parsed.lat,
                    lng: parsed.lng,
                    address: parsed.address,
                    port: parsed.port,
                    raw_outbound: parsed.raw_outbound,
                });
            }
            // a single bad link in a subscription shouldn't fail the whole import
            Err(e) => eprintln!("skipping unparsable link: {e}"),
        }
    }

    let sub = SubscriptionDto {
        id: format!("sub-{}", uuid::Uuid::new_v4()),
        name: format!("Подписка ({} серверов)", servers.len()),
        url,
        collapsed: false,
        servers,
    };

    let mut subs = state.subscriptions.lock().await;
    subs.push(sub.clone());
    store::save_subscriptions(&state.config_dir, &subs).await.map_err(|e| e.to_string())?;
    Ok(sub)
}

#[tauri::command]
async fn remove_subscription(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut subs = state.subscriptions.lock().await;
    subs.retain(|s| s.id != id);
    store::save_subscriptions(&state.config_dir, &subs).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn connect(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    server_id: String,
    mode: String,
) -> Result<(), String> {
    let run_mode = match mode.as_str() {
        "tun" => RunMode::Tun,
        "proxy" => RunMode::Proxy,
        other => return Err(format!("unknown mode: {other}")),
    };

    let server_cfg = find_server_config(&state, &server_id)
        .await
        .ok_or_else(|| format!("server {server_id} not found"))?;

    tray::set_tray_status(&app, "connecting");
    state
        .core
        .connect(&server_cfg, run_mode)
        .await
        .map_err(|e| e.to_string())?;
    tray::set_tray_status(&app, "connected");
    Ok(())
}

#[tauri::command]
async fn disconnect(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.core.disconnect().await.map_err(|e| e.to_string())?;
    tray::set_tray_status(&app, "disconnected");
    Ok(())
}

#[tauri::command]
async fn get_stats(state: tauri::State<'_, AppState>) -> Result<LiveStats, String> {
    state.core.stats().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ping_all(state: tauri::State<'_, AppState>) -> Result<Vec<SubscriptionDto>, String> {
    let mut subs = state.subscriptions.lock().await;

    // measure every server concurrently instead of one at a time
    let core = state.core.clone();
    let configs: Vec<(usize, usize, ServerConfig)> = subs
        .iter()
        .enumerate()
        .flat_map(|(si, sub)| {
            sub.servers.iter().enumerate().map(move |(ri, srv)| {
                (
                    si,
                    ri,
                    ServerConfig {
                        id: srv.id.clone(),
                        protocol: srv.protocol.clone(),
                        address: srv.address.clone(),
                        port: srv.port,
                        raw_outbound: srv.raw_outbound.clone(),
                    },
                )
            })
        })
        .collect();

    let measurements = join_all(configs.into_iter().map(|(si, ri, cfg)| {
        let core = core.clone();
        async move {
            let ping = core.measure_ping(&cfg).await.unwrap_or(9999);
            (si, ri, ping)
        }
    }))
    .await;

    for (si, ri, ping) in measurements {
        if let Some(sub) = subs.get_mut(si) {
            if let Some(srv) = sub.servers.get_mut(ri) {
                srv.ping = ping;
            }
        }
    }

    store::save_subscriptions(&state.config_dir, &subs).await.map_err(|e| e.to_string())?;
    Ok(subs.clone())
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(state.settings.lock().await.clone())
}

#[tauri::command]
async fn save_settings(state: tauri::State<'_, AppState>, settings: serde_json::Value) -> Result<(), String> {
    let mut current = state.settings.lock().await;
    *current = settings;
    store::save_settings(&state.config_dir, &current).await.map_err(|e| e.to_string())
}

async fn find_server_config(state: &AppState, server_id: &str) -> Option<ServerConfig> {
    let subs = state.subscriptions.lock().await;
    let dto = subs.iter().flat_map(|s| &s.servers).find(|s| s.id == server_id)?;
    Some(ServerConfig {
        id: dto.id.clone(),
        protocol: dto.protocol.clone(),
        address: dto.address.clone(),
        port: dto.port,
        raw_outbound: dto.raw_outbound.clone(),
    })
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let config_dir = app.path_resolver().app_config_dir().unwrap_or_default();
            let binary_path = config_dir
                .join("bin")
                .join(if cfg!(windows) { "sing-box.exe" } else { "sing-box" });
            let core: Arc<dyn CoreManager> = Arc::new(SingBoxCore::new(binary_path, config_dir.join("singbox")));

            // block_on is fine here: setup() runs once, before the event
            // loop, and these reads are tiny local JSON files.
            let subs = tauri::async_runtime::block_on(store::load_subscriptions(&config_dir));
            let settings = tauri::async_runtime::block_on(store::load_settings(&config_dir));

            app.manage(AppState {
                core,
                config_dir,
                subscriptions: tokio::sync::Mutex::new(subs),
                settings: tokio::sync::Mutex::new(settings),
            });
            Ok(())
        })
        .system_tray(tray::build_tray())
        .on_system_tray_event(tray::handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            list_subscriptions,
            add_subscription,
            remove_subscription,
            connect,
            disconnect,
            get_stats,
            ping_all,
            get_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nebula VPN");
}
