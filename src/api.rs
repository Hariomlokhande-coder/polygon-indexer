use axum::{
    extract::Query,
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex},
};
use rusqlite::Connection;
use crate::config::Config;
use crate::models::{NetFlow, Transfer};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use tracing::{info};
use tower_http::cors::{CorsLayer, Any};
use tokio::task;

#[derive(Deserialize)]
pub struct NetFlowQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct TransferQuery {
    pub token: String,
    pub limit: Option<u32>, // defaults to 10
}

pub async fn serve(cfg: Config, conn: Arc<Mutex<Connection>>) -> eyre::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Polygon Indexer API running" }))
        .route("/netflow", get({
            let conn = Arc::clone(&conn);
            move |q: Query<NetFlowQuery>| {
                let conn = Arc::clone(&conn);
                async move { Json(get_netflow(conn, &q.token).await) }
            }
        }))
        .route("/transfers", get({
            let conn = Arc::clone(&conn);
            move |q: Query<TransferQuery>| {
                let conn = Arc::clone(&conn);
                async move { Json(get_transfers(conn, &q.token, q.limit.unwrap_or(10)).await) }
            }
        }))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], cfg.port));
    info!("API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

// ---------- DB wrappers (spawn_blocking) ----------

async fn get_netflow(conn: Arc<Mutex<Connection>>, token: &str) -> NetFlow {
    let token = token.to_string();
    task::spawn_blocking(move || {
        let db = conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT token_address, cumulative_net, last_block, updated_at
             FROM netflows WHERE LOWER(token_address) = LOWER(?1)",
        ).unwrap();

        let row = stmt.query_row([token.clone()], |r| {
            let token_address: String = r.get(0)?;
            let cumulative_net_str: String = r.get(1)?;
            let last_block: i64 = r.get(2)?;
            let updated_at_str: String = r.get(3)?;

            let cumulative_net = Decimal::from_str(&cumulative_net_str).unwrap_or(Decimal::ZERO);
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(NetFlow {
                token_address,
                cumulative_net,
                last_block,
                updated_at,
            })
        });

        row.unwrap_or(NetFlow {
            token_address: token,
            cumulative_net: Decimal::ZERO,
            last_block: 0,
            updated_at: Utc::now(),
        })
    })
    .await
    .unwrap()
}

async fn get_transfers(conn: Arc<Mutex<Connection>>, token: &str, limit: u32) -> Vec<Transfer> {
    let token = token.to_string();
    task::spawn_blocking(move || {
        let db = conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT tx_hash, block_number, from_address, to_address, token_address, amount, direction, timestamp
             FROM transfers
             WHERE LOWER(token_address) = LOWER(?1)
             ORDER BY block_number DESC
             LIMIT ?2",
        ).unwrap();

        let rows = stmt.query_map((&token, limit as i64), |r| {
            Ok(Transfer {
                tx_hash: r.get(0)?,
                block_number: r.get(1)?,
                from_address: r.get(2)?,
                to_address: r.get(3)?,
                token_address: r.get(4)?,
                amount: r.get(5)?,
                direction: r.get(6)?,
                timestamp: r.get(7)?,
            })
        });

        rows.unwrap()
            .filter_map(Result::ok)
            .collect()
    })
    .await
    .unwrap()
}
