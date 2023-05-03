use opentelemetry::sdk::export::trace::stdout;
use reqwest_middleware::{Result};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

#[allow(dead_code)]
pub(crate) async fn init_logger() -> Result<()> {
    let tracer = stdout::new_pipeline().install_simple();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default().with(telemetry);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    Ok(())
}
