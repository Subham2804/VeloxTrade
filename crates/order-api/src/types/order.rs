use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub user_id: u64,
    pub market_id: u64,
    pub side: Side,
    pub qty: u64,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersRequest {
    pub orders: Vec<OrderRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    /// Monotonic id from `AtomicU64`; matches global arrival sequence.
    pub order_id: u64,
    pub user_id: u64,
    pub market_id: u64,
    pub side: Side,
    pub qty: u64,
    pub price: f64,
    /// Wall-clock ms when the order was accepted (after validation).
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueuedOrder {
    pub order_id: u64,
    pub arrived_at_ms: i64,
    pub request: OrderRequest,
}

pub fn validate(req: &OrderRequest) -> Result<(), String> {
    if req.qty == 0 {
        return Err("quantity must be greater than 0".into());
    }
    if req.price <= 0.0 {
        return Err("price must be greater than 0".into());
    }
    Ok(())
}

pub fn order_response_from(req: OrderRequest, order_id: u64, timestamp_ms: i64) -> OrderResponse {
    OrderResponse {
        order_id,
        user_id: req.user_id,
        market_id: req.market_id,
        side: req.side,
        qty: req.qty,
        price: req.price,
        timestamp_ms,
    }
}
