mod config;
mod torrents;
mod middleware;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::{info, warn};

use crate::config::AppConfig;
use crate::torrents::{Torrent, Torrents};

async fn verify_session(config: &AppConfig) -> Result<()> {
    let endpoint = format!("{}/auth/verify", config.endpoint);
    let resp = config.client.get(endpoint).send().await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid session"))
    }
}

async fn fetch_torrents(app_config: &AppConfig) -> Result<Torrents> {
    verify_session(app_config).await?;
    let endpoint = format!("{}/torrents", app_config.endpoint);
    app_config
        .client
        .get(endpoint)
        .send().await?
        .json::<Torrents>().await
        .map_err(|e| e.into())
}

async fn forget_directory(app_config: &AppConfig, directory: &str) -> Result<()> {
    app_config
        .client
        .post(format!(
            "{}/vfs/forget?dir={}",
            app_config.rclone_remote,
            directory
        ))
        .send()
        .await
        .context("Error calling vfs/forget")?;
    Ok(())
}

fn get_relative_directory(mount_directory: &Path, torrent_directory: &Path) -> PathBuf {
    if let Ok(relative_directory) = torrent_directory.strip_prefix(mount_directory) {
        info!(
            "Removing {} from {}",
            mount_directory.display(),
            torrent_directory.display()
        );
        relative_directory.to_path_buf()
    } else {
        warn!(
            "Torrent directory {} is not a child of mount directory {}",
            torrent_directory.display(),
            mount_directory.display()
        );
        torrent_directory.to_path_buf()
    }
}

async fn forget_torrent(app_config: &AppConfig, torrent: &Torrent) {
    let directory = get_relative_directory(&app_config.mount_directory, &torrent.directory);
    let directory_str = directory.to_str().unwrap_or("/torrents");
    forget_directory(app_config, directory_str).await.unwrap_or_else(|e| {
        warn!("Failed to forget directory {}: {:?}", directory_str, e)
    });
}

async fn process_torrent_addition(
    app_config: &AppConfig,
    torrent: &Torrent,
) {
    forget_torrent(app_config, torrent).await;
    info!(
        "New torrent: {} {} {}",
        torrent.name, torrent.percent_complete, torrent.directory.display()
    );
}

async fn process_torrent_update(
    app_config: &AppConfig,
    prev_torrent: &Torrent,
    new_torrent: &Torrent,
) {
    if new_torrent.percent_complete != prev_torrent.percent_complete {
        info!(
            "Percent complete changed for {} from {} to {}",
            new_torrent.name,
            prev_torrent.percent_complete,
            new_torrent.percent_complete
        );
        forget_torrent(app_config, new_torrent).await;
    }
}

async fn process_torrents(app_config: &AppConfig, torrents: Torrents) -> Result<()> {
    let mut cache_guard = app_config.cache.lock().await;
    let torrent_map = cache_guard.deref_mut();

    for (hash, torrent) in torrents.torrents {
        match torrent_map.get(&hash) {
            Some(prev_torrent) => process_torrent_update(app_config, prev_torrent, &torrent).await,
            None => process_torrent_addition(app_config, &torrent).await,
        }
        torrent_map.put(hash, torrent);
    }
    Ok(())
}

async fn fetch_and_process_torrents(app_config: &AppConfig) -> Result<()> {
    let torrents = fetch_torrents(app_config).await.context("Error fetching torrents")?;
    let size = torrents.torrents.len();
    info!("Processing {} torrents", size);
    let start = tokio::time::Instant::now();
    process_torrents(app_config, torrents).await?;
    info!("Processed {} torrents in {:?}", size, start.elapsed());
    Ok(())
}

async fn torrent_poller(config: AppConfig) {
    let interval = tokio::time::Duration::from_secs(config.poll_interval);
    loop {
        tokio::time::sleep(interval).await;
        fetch_and_process_torrents(&config).await.unwrap_or_default();
    }
}

#[tokio::main]
async fn main() {
    middleware::init_logger().await.unwrap();
    let config = AppConfig::from_env();

    // Pass the shared cache to the torrent_poller
    torrent_poller(config).await;
}

