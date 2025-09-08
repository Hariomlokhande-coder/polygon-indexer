use eyre::Result;
use rusqlite::{params, Connection};
use rust_decimal::Decimal;

const INIT_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS exchanges (
  id      INTEGER PRIMARY KEY AUTOINCREMENT,
  address TEXT UNIQUE NOT NULL,
  label   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS transfers (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  block_number  INTEGER NOT NULL,
  tx_hash       TEXT NOT NULL,
  log_index     INTEGER NOT NULL,
  token_address TEXT NOT NULL,
  from_address  TEXT NOT NULL,
  to_address    TEXT NOT NULL,
  amount        TEXT NOT NULL, -- Decimal stored as string
  direction     TEXT NOT NULL CHECK (direction IN ('IN','OUT')),
  timestamp     TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(tx_hash, log_index, token_address)
);

CREATE TABLE IF NOT EXISTS netflows (
  token_address  TEXT NOT NULL PRIMARY KEY,
  cumulative_net TEXT NOT NULL, -- Decimal stored as string
  last_block     INTEGER NOT NULL,
  updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

/// Connect to SQLite (with WAL mode for performance)
pub fn connect(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    Ok(conn)
}

/// Run schema migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(INIT_SQL)?;
    Ok(())
}

/// Insert or update a transfer
pub fn record_transfer(
    conn: &Connection,
    block_number: i64,
    tx_hash: &str,
    log_index: i64,
    token_address: &str,
    from: &str,
    to: &str,
    amount: Decimal,
    direction: &str,
) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO transfers (
            block_number, tx_hash, log_index,
            token_address, from_address, to_address,
            amount, direction, timestamp
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
        ON CONFLICT(tx_hash, log_index, token_address) DO UPDATE SET
            amount    = excluded.amount,
            direction = excluded.direction,
            timestamp = excluded.timestamp
        "#,
        params![
            block_number,
            tx_hash,
            log_index,
            token_address,
            from,
            to,
            amount.to_string(),
            direction
        ],
    )?;
    Ok(())
}