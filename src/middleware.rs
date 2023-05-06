use std::str::FromStr;
use std::time::Instant;

use log::{Level, LevelFilter};
use log::{debug, error};

use reqwest::Response;
use reqwest::{Request, StatusCode};

use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

#[allow(dead_code)]
pub(crate) async fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;
    use env_logger::Builder;
    use std::io::Write;

    let filter = if let Ok(log_filter) = std::env::var("RUST_APP_LOG") {
        LevelFilter::from_str(&log_filter).unwrap_or(LevelFilter::Info)
    } else {
        LevelFilter::Info
    };

    let logger = Builder::new()
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
        .filter(None, filter)
        .try_init();

    if let Err(e) = logger {
        error!("Failed to initialize logger: {}", e);
        return Err(Box::new(e));
    }

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

