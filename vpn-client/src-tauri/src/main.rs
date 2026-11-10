#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core_manager;
mod singbox;
mod tray;

use core_manager::{CoreManager, LiveStats, RunMode, ServerConfig};
use serde::{Deserialize, Serialize};
use singbox::SingBoxCore;
use std::sync::Arc;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDto {
    pub id: String,
    pub name: String,
    pub url: String,
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
}

/// Shared app state handed to every Tauri command. `core` is behind a
/// trait object so swapping sing-box for Xray-core later is a one-line
/// change here rather than a rewrite of the command layer.
struct AppState {
    core: Arc<dyn CoreManager>,
    // TODO: load/persist real subscriptions (SQLite or a flat JSON file
    // under the app's config dir); this starts empty and the frontend
    // falls back to its own mock data until that lands.
    subscriptions: tokio::sync::Mutex<Vec<SubscriptionDto>>,
}

#[tauri::command]
async fn list_subscriptions(state: tauri::State<'_, AppState>) -> Result<Vec<SubscriptionDto>, String> {
    Ok(state.subscriptions.lock().await.clone())
}

#[tauri::command]
async fn add_subscription(state: tauri::State<'_, AppState>, url: String) -> Result<SubscriptionDto, String> {
    // TODO: fetch `url`, parse the subscription payload (base64 list of
    // vless://, vmess://, trojan://, hysteria2://, ss:// links or a
    // sing-box/clash JSON) into ServerConfig entries, and persist them.
    let sub = SubscriptionDto {
        id: uuid_v4(),
        name: "Новая подписка".into(),
        url,
        servers: vec![],
    };
    state.subscriptions.lock().await.push(sub.clone());
    Ok(sub)
}

#[tauri::command]
async fn remove_subscription(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state.subscriptions.lock().await.retain(|s| s.id != id);
    Ok(())
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
async fn ping_all(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // TODO: fan out state.core.measure_ping() across every known server
    // concurrently (tokio::join!/FuturesUnordered) and write the results
    // back into `subscriptions`, then emit an event so the UI refreshes
    // without a full re-fetch.
    let _ = &state.core;
    Ok(())
}

async fn find_server_config(state: &AppState, server_id: &str) -> Option<ServerConfig> {
    let subs = state.subscriptions.lock().await;
    let dto = subs.iter().flat_map(|s| &s.servers).find(|s| s.id == server_id)?;
    Some(ServerConfig {
        id: dto.id.clone(),
        protocol: dto.protocol.clone(),
        address: String::new(), // filled in once real subscription parsing lands
        port: 0,
        raw_outbound: serde_json::json!({}),
    })
}

fn uuid_v4() -> String {
    // lightweight placeholder to avoid pulling in the `uuid` crate for a
    // scaffold; swap for `uuid::Uuid::new_v4()` in the real build.
    use std::time::{SystemTime, UNIX_EPOCH};
    format!("sub-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path_resolver().app_config_dir().unwrap_or_default();
            let binary_path = app_dir.join("bin").join(if cfg!(windows) { "sing-box.exe" } else { "sing-box" });
            let core: Arc<dyn CoreManager> = Arc::new(SingBoxCore::new(binary_path, app_dir.join("singbox")));

            app.manage(AppState {
                core,
                subscriptions: tokio::sync::Mutex::new(vec![]),
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nebula VPN");
}
