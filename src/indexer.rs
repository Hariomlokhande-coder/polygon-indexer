use std::sync::{Arc, Mutex};
use rusqlite::{Connection, Transaction};
use crate::{config::Config, aggregator, rpc, parser, db};
use eyre::Result;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub async fn run(cfg: Config, conn: Arc<Mutex<Connection>>) -> Result<()> {
    let backfill: u64 = 5000;                // blocks to scan on startup
    let lookback: u64 = 100;                 // blocks to scan per loop
    let rpc_pause = Duration::from_millis(200); // pause between RPC requests
    let mut retry_delay = 10;                // retry backoff in seconds

    info!("Indexer started with lookback = {} blocks", lookback);

    // ---------------------------
    // One-time backfill at startup
    // ---------------------------
    match rpc::get_block_number(&cfg.rpc_http_url).await {
        Ok(latest_block) => {
            retry_delay = 10; // reset after success
            let target_block = latest_block.saturating_sub(cfg.confirmations);
            let start_block = target_block.saturating_sub(backfill);

            info!("Backfill: scanning {} → {}", start_block, target_block);

            for token in &cfg.token_set {
                match rpc::get_transfer_logs(&cfg.rpc_http_url, token, start_block, target_block).await {
                    Ok(logs) => {
                        let mut processed_count = 0;
                        let mut db = conn.lock().unwrap();

                        // batch writes
                        let tx: Transaction = db.transaction()?;
                        for log in logs {
                            if let Some(transfer) = parser::decode_transfer(&log) {
                                let amount = Decimal::from_u128(transfer.value_u128)
                                    .unwrap_or(Decimal::ZERO)
                                    / Decimal::from(10u64.pow(18));

                                let direction = if cfg.exchange_set.contains(&transfer.to) {
                                    Some("IN")
                                } else if cfg.exchange_set.contains(&transfer.from) {
                                    Some("OUT")
                                } else {
                                    None
                                };

                                if let Some(dir) = direction {
                                    if let Err(e) = db::record_transfer(
                                        &tx,
                                        transfer.block_number as i64,
                                        &transfer.tx_hash,
                                        transfer.log_index as i64,
                                        token,
                                        &transfer.from.to_string(),
                                        &transfer.to.to_string(),
                                        amount,
                                        dir,
                                    ) {
                                        error!("Backfill insert failed: {:?}", e);
                                    } else {
                                        processed_count += 1;
                                    }
                                }
                            }
                        }
                        tx.commit()?; // commit batch

                        if let Err(e) = aggregator::update_netflows(&mut db) {
                            error!("Aggregator failed (backfill): {:?}", e);
                        }

                        info!("Backfilled {} transfers for token {}", processed_count, token);
                    }
                    Err(e) => warn!("Backfill failed for {}: {:?}", token, e),
                }

                sleep(rpc_pause).await; // avoid hammering
            }
        }
        Err(e) => {
            warn!("Failed to get latest block for backfill: {:?}", e);
            retry_delay = (retry_delay * 2).min(120);
        }
    }

    // ---------------------------
    // Continuous live indexing
    // ---------------------------
    loop {
        info!("Checking latest block...");

        match rpc::get_block_number(&cfg.rpc_http_url).await {
            Ok(latest_block) => {
                retry_delay = 10;
                let target_block = latest_block.saturating_sub(cfg.confirmations);
                info!("Live: Polygon block {} (up to {})", latest_block, target_block);

                let mut total_transfers = 0;

                for token in &cfg.token_set {
                    match rpc::get_transfer_logs(
                        &cfg.rpc_http_url,
                        token,
                        target_block.saturating_sub(lookback),
                        target_block,
                    ).await {
                        Ok(logs) => {
                            let mut processed_count = 0;
                            let mut db = conn.lock().unwrap();

                            let tx: Transaction = db.transaction()?;
                            for log in logs {
                                if let Some(transfer) = parser::decode_transfer(&log) {
                                    let amount = Decimal::from_u128(transfer.value_u128)
                                        .unwrap_or(Decimal::ZERO)
                                        / Decimal::from(10u64.pow(18));

                                    let direction = if cfg.exchange_set.contains(&transfer.to) {
                                        info!("Inflow {} POL → {:?} (block {})",
                                            amount, transfer.to, transfer.block_number);
                                        Some("IN")
                                    } else if cfg.exchange_set.contains(&transfer.from) {
                                        info!("Outflow {} POL ← {:?} (block {})",
                                            amount, transfer.from, transfer.block_number);
                                        Some("OUT")
                                    } else {
                                        None
                                    };

                                    if let Some(dir) = direction {
                                        if let Err(e) = db::record_transfer(
                                            &tx,
                                            transfer.block_number as i64,
                                            &transfer.tx_hash,
                                            transfer.log_index as i64,
                                            token,
                                            &transfer.from.to_string(),
                                            &transfer.to.to_string(),
                                            amount,
                                            dir,
                                        ) {
                                            error!("Insert failed: {:?}", e);
                                        } else {
                                            processed_count += 1;
                                            total_transfers += 1;
                                        }
                                    }
                                }
                            }
                            tx.commit()?; // commit writes

                            if let Err(e) = aggregator::update_netflows(&mut db) {
                                error!("Aggregator failed: {:?}", e);
                            }

                            info!("Indexed block {} for {} → {} transfers",
                                target_block, token, processed_count);
                        }
                        Err(e) => warn!("Fetch logs failed for {}: {:?}", token, e),
                    }

                    sleep(rpc_pause).await;
                }

                info!("Completed block {} → {} transfers", target_block, total_transfers);
            }
            Err(e) => {
                warn!("RPC failed this round: {:?}", e);
                retry_delay = (retry_delay * 2).min(120);
            }
        }

        sleep(Duration::from_secs(retry_delay)).await;
    }
}