#![feature(associated_type_defaults)]

mod certificates;
mod http_server;
mod errors;
mod addon_manage_backends;
mod registries;
mod ipc;
mod interconnects;
mod core_config;

use env_logger::{Env, TimestampPrecision, DEFAULT_FILTER_ENV};
use std::path::Path;
use structopt::StructOpt;
use log::{info, error};

use libohx::{common_config, wait_until_known_time};
use snafu::Error;
use futures_util::future::select;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging
    let mut builder = env_logger::Builder::from_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "info"));
    builder
        .format_timestamp(Some(TimestampPrecision::Seconds))
        .format_module_path(false)
        .init();

    // Command line / environment / file configuration
    let config: core_config::Config = core_config::Config::from_args();
    let common_config: common_config::Config = common_config::Config::from_args();

    create_root_directory(&common_config, &config)?;
    wait_until_known_time(false)?;
    certificates::check_gen_certificates(&common_config.get_certs_directory())?;

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);

    use http_server::HttpService;
    let mut http_service = HttpService::new(common_config.get_root_directory());

    let entries = http_service.redirect_entries();
    entries.add("core".to_owned(), "192.168.1.3".to_owned(), "common".to_owned());
    let entries = http_service.redirect_entries();
    entries.add("core".to_owned(), "192.168.1.3".to_owned(), "general".to_owned());

    // Start certificate refresh task with graceful shutdown warp channel
    let (certificate_refresher, mut cert_watch_shutdown_tx) = certificates::RefreshSelfSigned::new(http_service.control(),common_config.get_certs_directory());
    tokio::spawn(async move { certificate_refresher.run().await; });

    let http_shutdown = http_service.control();
    tokio::spawn(async move {
        let _ = shutdown_rx.recv().await;
        http_shutdown.shutdown().await;
        let _ = cert_watch_shutdown_tx.send(()).await;
    });

    // Ctrl+C task
    let mut shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        loop {
            let _ = tokio::signal::ctrl_c().await;
            info!("Ctrl+C: Shutting down");
            shutdown_tx_clone.send(()).await.unwrap();
        }
    });

    if let Err(e) = http_service.run().await {
        error!("{}", e);
    }

//    let mut shutdown_tx_clone = shutdown_tx.clone();
//    tokio::spawn(async move {
//        loop {
//            let _ = tokio::time::delay_for(Duration::from_secs(3)).await;
//            info!("Timeout: Shutting down");
//            shutdown_tx_clone.send(()).await.unwrap();
//        }
//    });

    Ok(())
}

fn create_root_directory(common_config: &common_config::Config, config: &core_config::Config) -> Result<(), std::io::Error> {
    let path = common_config.get_root_directory();
    if !config.create_root && !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "OHX Root directory does not exist. Consider using --create-root").into());
    }

    std::fs::create_dir_all(path.join("addons_http"))?;
    std::fs::create_dir_all(path.join("backups"))?;
    std::fs::create_dir_all(path.join("certs"))?;
    std::fs::create_dir_all(path.join("config"))?;
    std::fs::create_dir_all(path.join("interconnects"))?;
    std::fs::create_dir_all(path.join("rules"))?;
    std::fs::create_dir_all(path.join("scripts"))?;
    std::fs::create_dir_all(path.join("webui"))?;
    Ok(())
}
