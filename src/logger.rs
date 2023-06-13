use colored::Colorize;
use log::{Level, LevelFilter};
use std::io::Write;
use std::str::FromStr;

// Define ColorLevel to colorize log levels
trait ColorLevel {
    fn color(&self) -> colored::ColoredString;
}

impl ColorLevel for Level {
    fn color(&self) -> colored::ColoredString {
        match self {
            Level::Error => self.to_string().red().bold(),
            Level::Warn => self.to_string().yellow().bold(),
            Level::Info => self.to_string().cyan(),
            Level::Debug => self.to_string().magenta().dimmed(),
            _ => self.to_string().dimmed(),
        }
    }
}

// Initialize application-level logger
pub(crate) async fn init_logger() -> anyhow::Result<()> {
    let log_filter = std::env::var("LOG_LEVEL")
        .map(|log_filter| LevelFilter::from_str(&log_filter))
        .unwrap_or_else(|_| {
            println!(
                "Log Level Defaulting to {}",
                ColorLevel::color(&Level::Info)
            );
            Ok(LevelFilter::Info)
        })?;

    env_logger::Builder::new()
        .format(move |buf, record| {
            writeln!(
                buf,
                "{d} [{l}] {m}",
                d = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S %Z"),
                l = record.level().color(),
                m = record.args()
            )
        })
        .filter(None, log_filter)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logger: {}", e))?;

    Ok(())
}
