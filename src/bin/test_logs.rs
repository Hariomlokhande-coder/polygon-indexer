use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RpcResponse<T> {
    Success { result: T },
    Error { error: RpcError },
}

#[derive(Debug, Deserialize)]
struct RpcError {
    #[allow(dead_code)]
    code: i64,
    #[allow(dead_code)]
    message: String,
}

#[derive(Debug, Deserialize)]
struct Log {
    #[allow(dead_code)]
    address: String,
    topics: Vec<String>,
    data: String,

    #[serde(rename = "blockNumber")]
    block_number: String,

    #[serde(rename = "transactionHash")]
    tx_hash: String,
}

const RPC_URL: &str = "https://polygon-mainnet.core.chainstack.com/cb0aa24e92d67c58f4bfafc34244e3e7";
const POL_TOKEN: &str = "0x65E64963F9C5a663e7d7E986De45A9D8324aC0CE";
const TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

fn decode_address(topic: &str) -> String {
    if topic.len() >= 66 {
        format!("0x{}", &topic[26..])
    } else {
        topic.to_string()
    }
}

fn decode_uint256(data: &str) -> u128 {
    u128::from_str_radix(data.trim_start_matches("0x"), 16).unwrap_or(0)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();
    println!("Fetching latest POL token transfers...");

    // get latest block
    let block_res: RpcResponse<String> = client
        .post(RPC_URL)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_blockNumber",
            "params": []
        }))
        .send()
        .await?
        .json()
        .await?;

    let latest_block_hex = match block_res {
        RpcResponse::Success { result } => result,
        RpcResponse::Error { error } => {
            eprintln!("RPC error while fetching block: {:?}", error);
            return Ok(());
        }
    };

    let latest_block =
        u64::from_str_radix(latest_block_hex.trim_start_matches("0x"), 16)?;
    println!("Latest block: {}", latest_block);

    // fetch logs
    let logs_res: RpcResponse<Vec<Log>> = client
        .post(RPC_URL)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getLogs",
            "params": [{
                "fromBlock": format!("0x{:x}", latest_block.saturating_sub(10)), // <= 10 block range
                "toBlock": format!("0x{:x}", latest_block),
                "address": POL_TOKEN,
                "topics": [TRANSFER_TOPIC]
            }]
        }))
        .send()
        .await?
        .json()
        .await?;

    match logs_res {
        RpcResponse::Success { result: logs } => {
            println!("Fetched {} logs", logs.len());
            for log in logs.iter().take(5) {
                let from = decode_address(&log.topics[1]);
                let to = decode_address(&log.topics[2]);
                let value = decode_uint256(&log.data);

                println!(
                    "Tx: {} | Block: {} | From: {} | To: {} | Value: {}",
                    log.tx_hash, log.block_number, from, to, value
                );
            }
        }
        RpcResponse::Error { error } => {
            eprintln!("RPC error while fetching logs: {:?}", error);
        }
    }

    Ok(())
}