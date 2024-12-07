use axum::{routing::get, Router};

pub fn metrics_router() -> Router {
    Router::new().route("/metrics", get(metrics))
}

async fn metrics() -> String {
    prometheus::TextEncoder::new()
        .encode_to_string(&prometheus::gather())
        .unwrap()
}

use prometheus::{register_int_counter, register_int_counter_vec, IntCounter, IntCounterVec};
use std::sync::LazyLock;

pub static REQUEST_COUNT: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!("request_count", "Number of requests received").unwrap()
});

pub static ERROR_COUNT: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!("error_count", "Number of errors by type", &["type"]).unwrap()
});
