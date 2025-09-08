mod config;
mod db;
mod api;
mod indexer;
mod models;
mod aggregator;
mod rpc;
mod parser;

use std::sync::{Arc, Mutex};
use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Force logging to stdout with DEBUG level for visibility
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)  // show everything (INFO, DEBUG, WARN, ERROR)
        .with_writer(std::io::stdout)           // force logs to stdout
        .with_target(false)                     // cleaner logs (no module names unless needed)
        .init();

    println!("Logger initialized (DEBUG mode)");

    info!("Polygon Indexer starting...");

    // Load configuration
    let cfg = config::load()?;
    info!("Loaded config:");
    info!("  RPC URL: {}", cfg.rpc_http_url);
    info!("  DB Path: {}", cfg.db_path);
    info!("  Port: {}", cfg.port);
    info!("  Confirmations: {}", cfg.confirmations);
    info!("  Tokens tracked: {:?}", cfg.token_set);
    info!("  Exchanges tracked: {:?}", cfg.exchange_set);

    // Run DB migrations once at startup
    {
        let conn = db::connect(&cfg.db_path)?;
        db::run_migrations(&conn)?;
    }

    // Shared DB connection
    let shared_conn = Arc::new(Mutex::new(db::connect(&cfg.db_path)?));

    // Spawn API task
    let api_handle = tokio::spawn({
        let cfg = cfg.clone();
        let conn = Arc::clone(&shared_conn);
        async move { api::serve(cfg, conn).await }
    });

    // Spawn Indexer task
    let indexer_handle = tokio::spawn({
        let cfg = cfg.clone();
        let conn = Arc::clone(&shared_conn);
        async move { indexer::run(cfg, conn).await }
    });

    // Graceful shutdown
    tokio::select! {
        res = api_handle => match res {
            Ok(Ok(_)) => info!("API exited cleanly"),
            Ok(Err(e)) => error!("API error: {:?}", e),
            Err(e) => error!("API task panicked: {:?}", e),
        },
        res = indexer_handle => match res {
            Ok(Ok(_)) => info!("Indexer exited cleanly"),
            Ok(Err(e)) => error!("Indexer error: {:?}", e),
            Err(e) => error!("Indexer task panicked: {:?}", e),
        },
        _ = signal::ctrl_c() => {
            info!("Shutdown signal received, stopping...");
        }
    }

    info!("Polygon Indexer stopped.");
    Ok(())
}