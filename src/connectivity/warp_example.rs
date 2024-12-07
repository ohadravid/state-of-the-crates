use serde::{Deserialize, Serialize};

use warp::Filter;

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}


#[tokio::main]
pub async fn main() {
    let routes = warp::post()
        .and(warp::path("users"))
        .and(warp::body::json())
        .map(|mut payload: CreateUser| {
            warp::reply::json(&1337)
        });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
}