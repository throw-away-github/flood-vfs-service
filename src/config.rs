use std::env;
use std::num::NonZeroUsize;
use std::sync::Arc;
use lru::LruCache;
use reqwest_tracing::TracingMiddleware;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tokio::sync::Mutex;
use crate::torrents::Torrent;

pub struct AppConfig {
    pub(crate) endpoint: String,
    pub(crate) poll_interval: u64,
    pub(crate) rclone_remote: String,
    pub(crate) mount_directory: String,
    pub(crate) client: ClientWithMiddleware,
    pub(crate) cache: Arc<Mutex<LruCache<String, Torrent>>>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let endpoint = match env::var("ENDPOINT") {
            Ok(endpoint) => endpoint,
            Err(e) => panic!("ENDPOINT env var not set: {:?}", e),
        };
        let poll_interval = env::var("POLL_INTERVAL")
            .unwrap_or_default()
            .parse().unwrap_or(60);
        let rclone_remote = match env::var("RCLONE_REMOTE") {
            Ok(remote) => remote,
            Err(e) => panic!("RCLONE_REMOTE env var not set: {:?}", e),
        };
        let mount_directory = env::var("MOUNT_DIRECTORY").unwrap_or_default();
        // create a reqwest client to use for all requests
        const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .cookie_store(true)
            .build()
            .unwrap();
        let client = ClientBuilder::new(client)
            .with(TracingMiddleware::default())
            .build();

        let cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())));

        AppConfig {
            endpoint,
            poll_interval,
            rclone_remote,
            mount_directory,
            client,
            cache,
        }
    }
}
