use std::time::Instant;

use log::{debug, error, trace};
use reqwest::{Request, Response};

use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

#[derive(Clone, Debug)]
pub struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let start = Instant::now();
        trace!(
            "[REQUEST] Method: {} - URL: {} - Headers: {:?}",
            req.method(),
            req.url(),
            req.headers()
        );

        let response = next.run(req, extensions).await;

        match &response {
            Ok(response) => {
                trace!(
                    "[RESPONSE] Status: {} - Headers: {:?}",
                    response.status(),
                    response.headers()
                );
                debug!("Request took: {:?}", start.elapsed());
            }
            Err(e) => {
                error!("Request failed: {}", e);
            }
        }
        response
    }
}
