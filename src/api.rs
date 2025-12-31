use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router, debug_handler,
    extract::{Path, State},
    handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use crossbeam_queue::ArrayQueue;

use crate::tx_auction::InternalTransaction;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SendTransaction {
    pub tx: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SendBundle {
    pub txs: Vec<String>,
}

pub async fn send_transaction(
    s: State<RouterState>,
    Json(input): Json<SendTransaction>,
) -> impl IntoResponse {
    if let Err(_) = s.pending_tx.push(InternalTransaction {
        id: 0u64, // TODO: Figure out ID
        txs: vec![input.tx],
        tip_lamports: 0u64,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    }) {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::ACCEPTED
}
async fn send_bundle(s: State<RouterState>, Json(input): Json<SendBundle>) -> impl IntoResponse {
    // return error if too many transaction in a bundle
    if input.txs.len() > 5 {
        return StatusCode::BAD_REQUEST;
    }

    if let Err(_) = s.pending_tx.push(InternalTransaction {
        id: 0u64, // TODO: Figure out ID
        txs: input.txs,
        tip_lamports: 0u64,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    }) {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::ACCEPTED
}

#[derive(Clone)]
pub struct RouterState {
    pub pending_tx: Arc<ArrayQueue<InternalTransaction>>,
}

pub async fn create_router(pending_tx: Arc<ArrayQueue<InternalTransaction>>) {
    tracing::info!("Starting Client API server");
    let app = Router::new()
        .route("/healthz", get(|| async { "Server is online" }))
        .route("/v1/transactions", post(send_transaction))
        .route("/v1/bundle", post(send_bundle))
        .with_state(RouterState { pending_tx });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Error: Unable to bind with the port");

    axum::serve(listener, app)
        .await
        .expect("Error: Unable to serve the axum app")
}
