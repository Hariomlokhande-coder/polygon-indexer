// src/parser.rs
use alloy::primitives::Address;
use crate::rpc::Log;

/// A decoded ERC20 Transfer
#[derive(Debug, Clone)]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub value_u128: u128,    // raw token units (assumes < 2^128 for simplicity)
    pub block_number: u64,
    pub tx_hash: String,
    pub log_index: u64,      //  added for uniqueness
}

fn topic_to_address(topic: &str) -> Option<Address> {
    // topics[1] and topics[2] are 32-byte (padded) hex strings; address is the last 20 bytes
    let s = topic.trim_start_matches("0x");
    let bytes = hex::decode(s).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let addr_bytes: [u8; 20] = bytes[12..32].try_into().ok()?;
    Some(Address::from(addr_bytes))
}

/// Decode a single log into `Transfer`
pub fn decode_transfer(log: &Log) -> Option<Transfer> {
    if log.topics.len() < 3 {
        return None;
    }

    let from = topic_to_address(&log.topics[1])?;
    let to = topic_to_address(&log.topics[2])?;

    let value_hex = log.data.trim_start_matches("0x");
    let value_u128 = u128::from_str_radix(value_hex, 16).unwrap_or(0);

    let block_number =
        u64::from_str_radix(log.block_number_hex.trim_start_matches("0x"), 16).ok()?;

    let log_index =
        u64::from_str_radix(log.log_index_hex.trim_start_matches("0x"), 16).unwrap_or(0);

    Some(Transfer {
        from,
        to,
        value_u128,
        block_number,
        tx_hash: log.tx_hash.clone(),
        log_index, // ✅ included
    })
}
