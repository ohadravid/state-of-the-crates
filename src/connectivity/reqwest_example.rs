use std::time::Duration;

use serde_json::json;


#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _body = reqwest::get("https://www.rust-lang.org")
        .await?
        .text()
        .await?;

    let client = reqwest::ClientBuilder::new()
        .http1_only()
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(180))
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .build()?;

    let res = client.post("http://localhost:3000/users")
        .bearer_auth("hello2025")
        .json(&json!({ "username": "root" }))
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    dbg!(res);
        
    Ok(())
}
