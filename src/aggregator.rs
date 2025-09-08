use rusqlite::{Connection, params};
use eyre::Result;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromStr;
use tracing::info;

pub fn update_netflows(conn: &Connection) -> Result<()> {
    // Calculate inflows and outflows per token
    let mut stmt = conn.prepare(
        "
        SELECT 
            token_address,
            COALESCE(SUM(CASE WHEN direction = 'IN' THEN CAST(amount AS REAL) ELSE 0 END), 0) as inflow,
            COALESCE(SUM(CASE WHEN direction = 'OUT' THEN CAST(amount AS REAL) ELSE 0 END), 0) as outflow,
            MAX(block_number) as last_block
        FROM transfers
        GROUP BY token_address
        "
    )?;

    let rows = stmt.query_map([], |row| {
        let token_address: String = row.get(0)?;
        let inflow_f64: f64 = row.get(1).unwrap_or(0.0);
        let outflow_f64: f64 = row.get(2).unwrap_or(0.0);
        let last_block: i64 = row.get(3).unwrap_or(0);

        // Convert f64 → Decimal for precision
        let inflow = Decimal::from_str(&inflow_f64.to_string()).unwrap_or(Decimal::ZERO);
        let outflow = Decimal::from_str(&outflow_f64.to_string()).unwrap_or(Decimal::ZERO);

        let net = inflow - outflow;

        Ok((token_address, net, last_block))
    })?;

    for row in rows {
        let (token, net, last_block) = row?;
        conn.execute(
            "
            INSERT INTO netflows (token_address, cumulative_net, last_block, updated_at)
            VALUES (?1, ?2, ?3, datetime('now'))
            ON CONFLICT(token_address) DO UPDATE SET
                cumulative_net = excluded.cumulative_net,
                last_block = excluded.last_block,
                updated_at = excluded.updated_at
            ",
            params![token, net.to_string(), last_block],
        )?;

        info!("💾 Updated netflow for {} => {}", token, net);
    }

    Ok(())
}
