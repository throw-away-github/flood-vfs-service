use std::str::FromStr;
use std::time::Instant;
use colored::Colorize;
use std::io::Write;

use log::{Level, LevelFilter};
use log::{debug, error};

use reqwest::Response;
use reqwest::{Request, StatusCode};

use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

#[allow(dead_code)]
pub(crate) async fn init_logger() -> anyhow::Result<()> {
    let log_filter = std::env::var("RUST_APP_LOG")
        .map(|log_filter| LevelFilter::from_str(&log_filter))
        .unwrap_or_else(|_| Ok(LevelFilter::Info))?;

    env_logger::Builder::new()
        .format(move |buf, record| {
            let level = match record.level() {
                Level::Error => record.level().to_string().red().bold(),
                Level::Warn => record.level().to_string().yellow().bold(),
                Level::Info => record.level().to_string().cyan(),
                Level::Debug => record.level().to_string().magenta().dimmed(),
                _ => record.level().to_string().dimmed(),
            };
            writeln!(
                buf,
                "{d} [{l}] {m}",
                d = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S %Z"),
                l = level,
                m = record.args()
            )
        })
        .filter(None, log_filter)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logger: {}", e))?;

    Ok(())
}


async fn log_request(request: &Request) {
    // Log request details
    debug!(
        "[REQUEST] Method: {} - URL: {} - Headers: {:?}",
        request.method(),
        request.url(),
        request.headers()
    );
}

async fn log_response(response: &Response) {
    let status: StatusCode = response.status();
    let headers = response.headers();

    // Log response details
    debug!(
        "[RESPONSE] Status: {} - Headers: {:?}",
        status,
        headers
    );
}

#[derive(Clone)]
pub struct LoggingMiddleware;


#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {

    async fn handle(&self, req: Request, extensions: &mut Extensions, next: Next<'_>) -> reqwest_middleware::Result<Response> {
        let start = Instant::now();
        log_request(&req).await;

        let response = next.run(req, extensions).await;

        match response {
            Ok(response) => {
                log_response(&response).await;
                debug!("Request took: {:?}", start.elapsed());
                Ok(response)
            }
            Err(e) => {
                error!("Request failed: {}", e);
                Err(e)
            }
        }
    }
}

