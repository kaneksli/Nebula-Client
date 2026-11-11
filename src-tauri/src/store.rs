use crate::SubscriptionDto;
use anyhow::Result;
use std::path::{Path, PathBuf};

fn subs_path(config_dir: &Path) -> PathBuf {
    config_dir.join("subscriptions.json")
}

fn settings_path(config_dir: &Path) -> PathBuf {
    config_dir.join("settings.json")
}

pub async fn load_subscriptions(config_dir: &Path) -> Vec<SubscriptionDto> {
    let path = subs_path(config_dir);
    match tokio::fs::read(&path).await {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => Vec::new(), // first run: no file yet, not an error
    }
}

pub async fn save_subscriptions(config_dir: &Path, subs: &[SubscriptionDto]) -> Result<()> {
    tokio::fs::create_dir_all(config_dir).await?;
    let bytes = serde_json::to_vec_pretty(subs)?;
    tokio::fs::write(subs_path(config_dir), bytes).await?;
    Ok(())
}

pub async fn load_settings(config_dir: &Path) -> serde_json::Value {
    let path = settings_path(config_dir);
    match tokio::fs::read(&path).await {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|_| default_settings()),
        Err(_) => default_settings(),
    }
}

pub async fn save_settings(config_dir: &Path, settings: &serde_json::Value) -> Result<()> {
    tokio::fs::create_dir_all(config_dir).await?;
    let bytes = serde_json::to_vec_pretty(settings)?;
    tokio::fs::write(settings_path(config_dir), bytes).await?;
    Ok(())
}

fn default_settings() -> serde_json::Value {
    serde_json::json!({
        "background": "globe",
        "accentColor": "#3ddc97",
        "theme": "dark",
        "autoStart": false,
        "autoConnect": false,
        "autoUpdateSubscriptions": false,
        "core": "sing-box",
        "globalShortcut": "Ctrl+Alt+V"
    })
}
