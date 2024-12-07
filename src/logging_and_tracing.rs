use tracing::{debug, info, info_span, Instrument};

#[tracing::instrument]
fn hello(year: usize) {
    println!("Hello, {}!", year);
    info!("Done saying hello");
}

async fn goodbye() {
    println!("Goodbye!");
    debug!("Done saying goodbye");
}

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    hello(2025);

    goodbye()
        .instrument(info_span!("goodbye", year = 2024))
        .await;
}
