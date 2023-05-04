mod config;
mod torrents;
mod middleware;
use std::ops::DerefMut;
use std::path::Path;

use anyhow::Result;
use log::{info, warn};


use crate::config::AppConfig;
use crate::torrents::{Torrents};

async fn verify_session(config: &AppConfig) -> Result<()> {
    let endpoint = format!("{}/auth/verify", config.endpoint);
    let resp = config.client.get(endpoint).send().await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid session"))
    }
}

async fn fetch_and_process_torrents(config: &AppConfig) -> Result<()> {
    // Fetch torrents from the API
    verify_session(config).await?;
    let endpoint = format!("{}/torrents", config.endpoint);
    let resp = config.client.get(endpoint).send().await?.json::<Torrents>().await?;
    println!("Processing {} torrents", resp.torrents.len());

    // Lock and obtain the torrent_map for mutation
    let mut cache_guard = config.cache.lock().await;
    let torrent_map = cache_guard.deref_mut();

    for (hash, torrent) in resp.torrents {
        if let Some(prev_torrent) = torrent_map.get(&hash) {
            // Call vfs/forget if percent_complete has changed
            if torrent.percent_complete != prev_torrent.percent_complete {
                info!("Percent complete changed for {} from {} to {}", torrent.name, prev_torrent.percent_complete, torrent.percent_complete);

                let torrent_dir = match torrent.directory.strip_prefix(&config.mount_directory) {
                    Ok(dir) => {
                        if dir == torrent.directory {
                            torrent.directory
                                .strip_prefix(&Path::new("/")
                                    .join(&config.mount_directory))
                                .unwrap_or(&torrent.directory)
                        } else {
                            dir
                        }
                    }
                    Err(_) => {
                        warn!("Torrent directory {} is not a child of mount directory {}", torrent.directory.display(), config.mount_directory.display());
                        &torrent.directory
                    }
                };

                info!("Calling vfs/forget for {}", torrent_dir.display());
                // if torrent_dir contains a non-utf8 character, just forget the whole thing
                let torrent_dir_str = match torrent_dir.to_str().unwrap_or_default() {
                    "" => "/",
                    s => s,
                };

                config.client
                    .post(format!("{}/vfs/forget?dir={}", config.rclone_remote, torrent_dir_str))
                    .send()
                    .await?;
            }
        } else {
            // Call vfs/forget for new torrent
            config.client
                .post(format!("{}/vfs/forget?dir={}", config.rclone_remote, torrent.directory.display()))
                .send()
                .await?;
            info!("New Torrent {} {} {}", torrent.name, torrent.percent_complete, torrent.directory.display());
            torrent_map.put(hash.clone(), torrent);
        }
    }
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

