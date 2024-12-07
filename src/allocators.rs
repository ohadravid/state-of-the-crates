use axum::{
    body::{Body, Bytes},
    extract::Path,
    routing::post,
    Router,
};
use futures::stream::StreamExt;
use image::{ImageFormat, ImageReader};
use std::io::Cursor;

// #[cfg(not(target_env = "msvc"))]
// use tikv_jemallocator::Jemalloc;

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/resize/w/:w/h/:h", post(resize_image));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn resize_image(Path((width, height)): Path<(u32, u32)>, body: Body) -> Bytes {
    let mut body = body.into_data_stream();
    let mut data = Vec::with_capacity(128 * 1024);

    while let Some(bytes) = body.next().await {
        let bytes = bytes.unwrap();
        data.extend(bytes);
    }

    let resized_image = tokio::task::spawn_blocking(move || {
        let img = ImageReader::with_format(Cursor::new(data), ImageFormat::Jpeg)
            .decode()
            .unwrap();
        img.resize(width, height, image::imageops::FilterType::Lanczos3)
            .into_bytes()
    })
    .await
    .unwrap();

    Bytes::from(resized_image)
}
