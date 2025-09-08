// src/rpc.rs
use eyre::{eyre, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    #[allow(dead_code)]
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,

    #[serde(rename = "blockNumber")]
    pub block_number_hex: String,

    #[serde(rename = "transactionHash")]
    pub tx_hash: String,

    #[serde(rename = "logIndex")] // ✅ Needed for uniqueness in parser
    pub log_index_hex: String,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: T,
}

/// ERC20 Transfer event topic keccak256("Transfer(address,address,uint256)")
pub const TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

/// Get the latest block number with retries and timeout
pub async fn get_block_number(rpc_url: &str) -> Result<u64> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    for attempt in 1..=3 {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_blockNumber",
            "params": []
        });

        info!("📡 Sending eth_blockNumber → {}", rpc_url);

        let res = client.post(rpc_url).json(&payload).send().await;

        match res {
            Ok(resp) => {
                if resp.status() != StatusCode::OK {
                    return Err(eyre!("RPC error: HTTP {}", resp.status()));
                }
                let text = resp.text().await?;
                info!("📩 Raw blockNumber response: {}", text);

                let parsed: RpcResponse<String> = serde_json::from_str(&text)?;
                let block_number =
                    u64::from_str_radix(parsed.result.trim_start_matches("0x"), 16)?;
                return Ok(block_number);
            }
            Err(e) if attempt < 3 => {
                eprintln!(
                    "⚠️ RPC request failed (attempt {}): {}. Retrying...",
                    attempt, e
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => return Err(eyre!("❌ RPC request failed after 3 retries: {}", e)),
        }
    }

    Err(eyre!("Unreachable: retries exhausted"))
}

/// Fetch ERC20 Transfer logs for a token in a block range
pub async fn get_transfer_logs(
    rpc_url: &str,
    token_address: &str,
    from_block: u64,
    to_block: u64,
) -> Result<Vec<Log>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_getLogs",
        "params": [{
            "fromBlock": format!("0x{:x}", from_block),
            "toBlock": format!("0x{:x}", to_block),
            "address": token_address,
            "topics": [TRANSFER_TOPIC]
        }]
    });

    info!(
        "📡 Sending eth_getLogs → {} (range {} → {}, token {})",
        rpc_url, from_block, to_block, token_address
    );

    let resp = client.post(rpc_url).json(&payload).send().await?;
    let text = resp.text().await?;
    info!("📩 Raw getLogs response: {}", text);

    let parsed: RpcResponse<Vec<Log>> = serde_json::from_str(&text)?;
    Ok(parsed.result)
}
