mod config;
mod torrents;
mod middleware;
use std::ops::DerefMut;
use anyhow::Result;

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
            if torrent.percent_complete != prev_torrent.percent_complete {
                println!("Percent complete changed for {} from {} to {}", torrent.name, prev_torrent.percent_complete, torrent.percent_complete);
                // remove the mount directory from the torrent directory
                let torrent_dir = torrent.directory.replace(&config.mount_directory, "");
                // Call vfs/forget if percent_complete has changed
                println!("Calling vfs/forget for {}", torrent_dir);
                config.client
                    .post(format!("{}/vfs/forget?dir={}", config.rclone_remote, torrent_dir))
                    .send()
                    .await?;
            }
        } else {
            // Call vfs/forget for new torrent
            config.client
                .post(format!("{}/vfs/forget?dir={}", config.rclone_remote, torrent.directory))
                .send()
                .await?;
            println!("New Torrent {} {} {}", torrent.name, torrent.percent_complete, torrent.directory);
            torrent_map.put(hash.clone(), torrent);
        }

    }


    Ok(())
}

async fn torrent_poller(config: AppConfig) {
    let interval = tokio::time::Duration::from_secs(config.poll_interval);
    loop {
        tokio::time::sleep(interval).await;
        if let Err(e) = fetch_and_process_torrents(&config).await {
            eprintln!("Error: {:?}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    // TODO: figure out how to use the tracing output
    // middleware::init_logger().await.unwrap();
    let config = AppConfig::from_env();

    // Pass the shared cache to the torrent_poller
    torrent_poller(config).await;
}

