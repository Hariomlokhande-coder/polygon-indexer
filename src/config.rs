use dotenvy::dotenv;
use eyre::Result;
use serde::Deserialize;
use std::{collections::HashSet, env};
use alloy::primitives::Address;
use tracing::info;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub rpc_http_url: String,       // ✅ HTTP RPC URL
    pub db_path: String,
    pub confirmations: u64,
    pub exchange_set: HashSet<Address>,
    pub token_set: HashSet<String>,
    pub port: u16,
}

pub fn load() -> Result<Config> {
    dotenv().ok(); // ✅ Load from .env file

    // ✅ Load RPC URL (prefer HTTP, fallback to polygon-rpc.com)
    let rpc_http_url = env::var("RPC_HTTP_URL")
        .or_else(|_| env::var("POLYGON_RPC")) // alias support
        .unwrap_or_else(|_| "https://polygon-rpc.com".to_string());

    // ✅ SQLite DB path (default: netflow.db)
    let db_path = env::var("DATABASE_URL").unwrap_or_else(|_| "netflow.db".to_string());

    // ✅ Block confirmations (default: 2)
    let confirmations = env::var("CONFIRMATIONS")
        .unwrap_or_else(|_| "2".to_string())
        .parse()
        .unwrap_or(2);

    // ✅ API port (default: 8080)
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    // ✅ Binance exchange wallets (default: empty set)
    let exchange_set: HashSet<Address> = env::var("EXCHANGE_ADDRESSES")
        .or_else(|_| env::var("BINANCE_WALLETS"))
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.parse::<Address>().ok())
        .collect();

    // ✅ Token contract addresses (default: empty set)
    let token_set: HashSet<String> = env::var("TOKEN_ADDRESSES")
        .or_else(|_| env::var("POL_TOKEN").map(|s| s.to_string()))
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let cfg = Config {
        rpc_http_url,
        db_path,
        confirmations,
        exchange_set,
        token_set,
        port,
    };

    // ✅ Log loaded config for debugging
    info!("Loaded config: {:?}", cfg);

    Ok(cfg)
}
