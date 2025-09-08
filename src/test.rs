use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://polygon-mainnet.public.blastapi.io";
    let token = "0x65E64963F9C5a663e7d7E986De45A9D8324aC0CE";

    let client = Client::new();

    // latest block number
    let block_number: serde_json::Value = client.post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_blockNumber",
            "params": []
        }))
        .send().await?
        .json().await?;

    println!("Latest block = {}", block_number);

    // fetch logs in last 10 blocks
    let latest_hex = block_number["result"].as_str().unwrap();
    let latest_block = u64::from_str_radix(latest_hex.trim_start_matches("0x"), 16).unwrap();
    let from_block = format!("0x{:x}", latest_block - 10);
    let to_block = format!("0x{:x}", latest_block);

    let logs: serde_json::Value = client.post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "eth_getLogs",
            "params": [{
                "fromBlock": from_block,
                "toBlock": to_block,
                "address": token,
                "topics": [
                    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
                ]
            }]
        }))
        .send().await?
        .json().await?;

    println!("Logs = {:#?}", logs);

    Ok(())
}