//! Example of cURL request:
//!
//! ```sh
//! curl -XPOST --json '{"username":"a"}' -H "Authorization: Bearer hello2025" http://localhost:3000/users
//! curl -H "Authorization: Bearer hello2025" http://localhost:3000/metrics
//! ```

use std::collections::{hash_map::Entry, HashMap};

use crate::metrics;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tower_http::validate_request::ValidateRequestHeaderLayer;

type UsersDB = HashMap<String, u64>;

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .layer(ValidateRequestHeaderLayer::bearer("hello2025"))
        .with_state(Arc::new(Mutex::new(UsersDB::new())))
        .merge(metrics::metrics_router());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, 2025!"
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

async fn create_user(
    State(state): State<Arc<Mutex<UsersDB>>>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<serde_json::Value>) {
    crate::metrics::REQUEST_COUNT.inc();

    match state.lock().unwrap().entry(payload.username) {
        Entry::Occupied(_) => {
            crate::metrics::ERROR_COUNT.with_label_values(&["conflict"]).inc();
            (
                StatusCode::CONFLICT,
                Json(json!({ "error": "User already exists" })),
            )
        }

        Entry::Vacant(entry) => {
            entry.insert(1337);
            (StatusCode::CREATED, Json(json!(1337)))
        }
    }
}
