mod config;
mod logger;
mod middleware;
mod torrents;

use std::ops::DerefMut;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::{error, info, trace, warn};

use crate::config::AppState;
use crate::torrents::{Torrent, Torrents};

async fn verify_session(app_state: &AppState) -> Result<()> {
    let endpoint = format!("{}/auth/verify", app_state.endpoint);
    let resp = app_state.client.get(endpoint).send().await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid session"))
    }
}

async fn fetch_torrents(app_state: &AppState) -> Result<Torrents> {
    verify_session(app_state).await?;
    let endpoint = format!("{}/torrents", app_state.endpoint);
    app_state
        .client
        .get(endpoint)
        .send()
        .await?
        .json::<Torrents>()
        .await
        .map_err(|e| e.into())
}

async fn forget_directory(app_state: &AppState, directory: &str) -> Result<()> {
    app_state
        .client
        .post(format!(
            "{}/vfs/forget?dir={}",
            app_state.rclone_remote, directory
        ))
        .send()
        .await
        .context("Error calling vfs/forget")?;
    Ok(())
}

fn get_relative_directory(mount_directory: &Path, torrent_directory: &Path) -> PathBuf {
    if let Ok(relative_directory) = torrent_directory.strip_prefix(mount_directory) {
        trace!(
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

async fn forget_torrent(app_state: &AppState, torrent: &Torrent) {
    let directory = get_relative_directory(&app_state.mount_directory, &torrent.directory);
    let directory_str = directory.to_str().unwrap_or("/torrents");
    forget_directory(app_state, directory_str)
        .await
        .unwrap_or_else(|e| warn!("Failed to forget directory {}: {:?}", directory_str, e));
}

async fn process_torrent_addition(app_state: &AppState, torrent: &Torrent) {
    info!(
        "New torrent: {} {} {}",
        torrent.name,
        torrent.percent_complete,
        torrent.directory.display()
    );
    forget_torrent(app_state, torrent).await;
}

async fn process_torrent_update(
    app_state: &AppState,
    prev_torrent: &Torrent,
    new_torrent: &Torrent,
) {
    if new_torrent.percent_complete != prev_torrent.percent_complete {
        info!(
            "Percent complete changed for {} from {} to {}",
            new_torrent.name, prev_torrent.percent_complete, new_torrent.percent_complete
        );
        forget_torrent(app_state, new_torrent).await;
    }
}

async fn process_torrents(app_state: &AppState, torrents: Torrents) -> Result<()> {
    let mut cache_guard = app_state.cache.lock().await;
    let torrent_map = cache_guard.deref_mut();

    for (hash, torrent) in torrents.torrents {
        match torrent_map.get(&hash) {
            Some(prev_torrent) => process_torrent_update(app_state, prev_torrent, &torrent).await,
            None => process_torrent_addition(app_state, &torrent).await,
        }
        torrent_map.put(hash, torrent);
    }
    Ok(())
}

async fn fetch_and_process_torrents(app_state: &AppState) -> Result<()> {
    let torrents = fetch_torrents(app_state)
        .await
        .context("Error fetching torrents")?;
    let size = torrents.torrents.len();
    info!("Processing {} torrents", size);
    let start = tokio::time::Instant::now();
    process_torrents(app_state, torrents).await?;
    info!("Processed {} torrents in {:?}", size, start.elapsed());
    Ok(())
}

async fn torrent_poller(app_state: AppState) {
    let interval = tokio::time::Duration::from_secs(app_state.poll_interval);
    loop {
        tokio::time::sleep(interval).await;
        fetch_and_process_torrents(&app_state)
            .await
            .unwrap_or_else(|e| {
                error!("Error fetching torrents: {:?}", e);
            });
    }
}

#[tokio::main]
async fn main() {
    logger::init_logger().await.unwrap();
    let app_state = AppState::from_env();

    // Pass the shared cache to the torrent_poller
    torrent_poller(app_state).await;
}
