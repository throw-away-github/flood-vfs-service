use crate::middleware;
use crate::torrents::Torrent;
use lru::LruCache;
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use std::env;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::Mutex;

#[derive(Debug, StructOpt)]
struct AppConfig {
    #[structopt(short = "e", long = "endpoint", env = "ENDPOINT", parse(try_from_str))]
    endpoint: Url,
    #[structopt(
        short = "i",
        long = "poll-interval",
        env = "POLL_INTERVAL",
        default_value = "60"
    )]
    poll_interval: u64,
    #[structopt(
        short = "r",
        long = "rclone-remote",
        env = "RCLONE_REMOTE",
        parse(try_from_str)
    )]
    rclone_remote: Url,
    #[structopt(
        short = "m",
        long = "mount-directory",
        env = "MOUNT_DIRECTORY",
        parse(from_os_str)
    )]
    mount_directory: PathBuf,
}

pub struct AppState {
    pub(crate) endpoint: Url,
    pub(crate) poll_interval: u64,
    pub(crate) rclone_remote: Url,
    pub(crate) mount_directory: PathBuf,
    pub(crate) client: ClientWithMiddleware,
    pub(crate) cache: Arc<Mutex<LruCache<String, Torrent>>>,
}

impl AppState {
    pub fn from_env() -> Self {
        let app_config = AppConfig::from_args();
        // create a reqwest client to use for all requests
        const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .cookie_store(true)
            .build()
            .unwrap();
        let client = ClientBuilder::new(client)
            .with(middleware::LoggingMiddleware)
            .build();

        let cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())));

        AppState {
            endpoint: app_config.endpoint,
            poll_interval: app_config.poll_interval,
            rclone_remote: app_config.rclone_remote,
            mount_directory: app_config.mount_directory,
            client,
            cache,
        }
    }
}
