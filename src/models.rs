// src/models.rs
use serde::Serialize;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Represents a single ERC20 transfer involving Binance
#[derive(Debug, Serialize)]
pub struct Transfer {
    pub tx_hash: String,
    pub block_number: i64,
    pub from_address: String,
    pub to_address: String,
    pub token_address: String,
    pub amount: String,        // keep as String (safe for DB + API)
    pub direction: String,     // "IN" or "OUT"
    pub timestamp: String,     // store + return as RFC3339 string
}

/// Represents aggregated netflows for a token
#[derive(Debug, Serialize)]
pub struct NetFlow {
    pub token_address: String,
    pub cumulative_net: Decimal,   // keep Decimal (math friendly)
    pub last_block: i64,
    pub updated_at: DateTime<Utc>, // DateTime for consistency
}

