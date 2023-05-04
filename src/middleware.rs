use std::time::Instant;

use log::LevelFilter;
use log::{debug, error};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use reqwest::Response;
use reqwest::{Request, StatusCode};

use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

#[allow(dead_code)]
pub(crate) async fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    // Configure log4rs
    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} [{l}] {m}{n}")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console_appender)))
        .build(
            Root::builder()
                .appender("console")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

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

