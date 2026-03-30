use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::post,
    Router,
};
use crossbeam_channel::{bounded, Sender, TrySendError};
use order_api::types::order::{
    order_response_from, validate, OrderResponse, OrdersRequest, QueuedOrder,
};

/// Max orders buffered between HTTP accept and downstream processing.
const ORDER_QUEUE_CAPACITY: usize = 10_000;

#[derive(Clone)]
struct AppState {
    next_order_id: Arc<AtomicU64>,
    order_tx: Sender<QueuedOrder>,
}

#[tokio::main]
async fn main() {
    let (order_tx, order_rx) = bounded(ORDER_QUEUE_CAPACITY);

    std::thread::spawn(move || {
        for _ in order_rx {}
    });

    let state = AppState {
        next_order_id: Arc::new(AtomicU64::new(0)),
        order_tx,
    };

    let app = Router::new()
        .route("/orders", post(post_orders))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("bind port 3000");
    println!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("server");
}

fn next_order_id(counter: &AtomicU64) -> u64 {
    counter.fetch_add(1, Ordering::SeqCst) + 1
}

async fn post_orders(
    State(state): State<AppState>,
    Json(body): Json<OrdersRequest>,
) -> Result<Json<Vec<OrderResponse>>, (StatusCode, String)> {
    if body.orders.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "orders must contain at least one entry".into(),
        ));
    }
    for o in &body.orders {
        validate(o).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    }

    let mut out = Vec::with_capacity(body.orders.len());

    for req in body.orders {
        let arrived_at_ms = chrono::Utc::now().timestamp_millis();
        let order_id = next_order_id(&state.next_order_id);
        let queued = QueuedOrder {
            order_id,
            arrived_at_ms,
            request: req.clone(),
        };
        match state.order_tx.try_send(queued) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "order queue is full; retry later".into(),
                ));
            }
            Err(TrySendError::Disconnected(_)) => {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "order queue is unavailable".into(),
                ));
            }
        }
        out.push(order_response_from(req, order_id, arrived_at_ms));
    }

    Ok(Json(out))
}
