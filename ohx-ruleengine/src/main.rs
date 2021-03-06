mod rule_config;
mod rule;
mod rules_registry;
mod modules_registry;
mod buildin_modules;
mod engine;

use env_logger::{Env, TimestampPrecision, DEFAULT_FILTER_ENV};
use std::path::Path;
use structopt::StructOpt;
use log::{info, error};

use libohxcore::{wait_until_known_time, common_config, shutdown_on_ctrl_c};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging
    let mut builder = env_logger::Builder::from_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "info"));
    builder
        .format_timestamp(Some(TimestampPrecision::Seconds))
        .format_module_path(false)
        .init();

    // Command line / environment / file configuration
    let config: rule_config::Config = rule_config::Config::from_args();
    let (mut shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);

    shutdown_on_ctrl_c(shutdown_tx.clone());
    create_root_directory(&config.common)?;
    wait_until_known_time(false).await?;

    let _ = tokio::time::delay_for(Duration::from_secs(3)).await;
    info!("Timeout: Shutting down");
    shutdown_tx.send(()).await.unwrap();

    Ok(())
}

/// Creates all OHX root directory subdirectories required to run the OHX core service
fn create_root_directory(common_config: &common_config::Config) -> Result<(), std::io::Error> {
    let path = common_config.get_root_directory();
    if !common_config.create_root && !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "OHX Root directory does not exist. Consider using --create-root").into());
    }

    std::fs::create_dir_all(path.join("rules"))?;
    std::fs::create_dir_all(path.join("scripts"))?;
    std::fs::create_dir_all(path.join("config"))?;
    Ok(())
}
