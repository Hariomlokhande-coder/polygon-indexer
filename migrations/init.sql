-- ================================================
-- ✅ Transfers table: store every raw transfer
-- ================================================
CREATE TABLE IF NOT EXISTS transfers (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    tx_hash       TEXT NOT NULL,
    log_index     INTEGER NOT NULL,
    block_number  INTEGER NOT NULL,
    from_address  TEXT NOT NULL,
    to_address    TEXT NOT NULL,
    token_address TEXT NOT NULL,
    amount        TEXT NOT NULL, -- Decimal stored as string
    direction     TEXT NOT NULL CHECK (direction IN ('IN','OUT')), -- enforce only IN/OUT
    timestamp     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),

    -- prevent duplicate insertions of the same log
    UNIQUE(tx_hash, log_index, token_address)
);

-- ================================================
-- ✅ Netflows table: pre-aggregated balances
-- ================================================
CREATE TABLE IF NOT EXISTS netflows (
    token_address   TEXT PRIMARY KEY,
    cumulative_net  TEXT NOT NULL DEFAULT '0', -- cumulative inflow - outflow
    inflow_total    TEXT NOT NULL DEFAULT '0', -- total inflows
    outflow_total   TEXT NOT NULL DEFAULT '0', -- total outflows
    last_block      INTEGER NOT NULL DEFAULT 0,
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- ================================================
-- ✅ Binance addresses (exchange set)
-- ================================================
CREATE TABLE IF NOT EXISTS binance_addresses (
    address TEXT PRIMARY KEY
);

-- ================================================
-- ✅ Indexes for query performance
-- ================================================
CREATE INDEX IF NOT EXISTS idx_transfers_block
    ON transfers (block_number);

CREATE INDEX IF NOT EXISTS idx_transfers_token
    ON transfers (token_address);

CREATE INDEX IF NOT EXISTS idx_transfers_from
    ON transfers (from_address);

CREATE INDEX IF NOT EXISTS idx_transfers_to
    ON transfers (to_address);

-- Composite index for common queries
CREATE INDEX IF NOT EXISTS idx_transfers_from_to
    ON transfers (from_address, to_address);

-- Index to speed up recent history queries
CREATE INDEX IF NOT EXISTS idx_transfers_token_block
    ON transfers (token_address, block_number DESC);