This repository includes all the code samples used for the [State of the Crates 2025](https://ohadravid.github.io/posts/2024-12-state-of-the-crates/) blog post, also replicated below.

It is mostly a collection of usage samples of commonly used crates.

----

One of the best things about Rust is that there are so many high-quality crates for everything and anything you want.

It can be hard to choose, so I wanted to share some of the crates I used this year at $work and explain why.

For context, we're a small-ish Rust & Python shop handling compute-heavy workloads on-prem. Our system is split into a reasonable number of services that talk to each other mostly over RabbitMQ or HTTP.

## TL;DR

**Connectivity**  
- **axum**: HTTP(S) servers with flexible routing and integration  
- **reqwest (with rustls)**: Modern HTTP client with configuration options  
- **prost / tonic**: Protobuf and gRPC ecosystem support  
- **lapin**: AMQP (RabbitMQ) client

**Error Handling**  
- **thiserror**: Errors for libraries  
- **anyhow**: Errors for applications

**Testing**  
- **rstest**: Parametric tests and fixtures (familiar if you know `pytest`)

**Serialization & Data**  
- **serde / serde_json**: General-purpose serialization and JSON handling  
- **bincode**: Compact binary format when JSON is too large  
- **humantime-serde**: Human-readable durations  

**Utilities**  
- **rustc-hash / sha1_smol**: Faster and predictable non-crypto hashing  
- **tikv-jemallocator**: Improved memory allocation performance  
- **uuid**: UUID creation and parsing  
- **chrono**: UTC-based time handling  
- **derivative**: Customize derived trait implementations
- **image**: Working with images

**CLI & Logging**  
- **argh / clap**: Lightweight vs. feature-rich CLI parsing  
- **tracing**: Structured logging and tracing
- **prometheus**: Metrics

**SQL & ORMs**  
- **sea-orm / sea-query / sqlx**: Async-friendly ORM and query building

**ML & Advanced**  
- **half**: Half-sized float types
- **approx**: Approximate comparisons for floats
- **ndarray / nalgebra**: Arrays and linear algebra ops
- **ort**: ONNX runtime for local model inference


## Connectivity

Everything (`async`) is `tokio`.

For building HTTP(S) servers, `axum` is both flexible and simple, yet still scales nicely to handle advanced, complex workflows (you can merge routers, extract routers to functions, share state using a simple mutex, write custom "extractors", integrate tracing, ...).

We use it for everything, from small debug/metric servers to heavy data ingress.

```rust
let app = Router::new()
    .route("/", get(root))
    .route("/users", post(create_user));

async fn root() -> &'static str {
    "Hello, 2025!"
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

async fn create_user(
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<u64>) {
    // ...
    (StatusCode::CREATED, Json(1337))
}
```

There was a time when combining routers was cumbersome (RIP `BoxRoute`) but these days everything is a breeze and `axum` is really a one-size-fits-all solution.

For example sharing state (which can be anything from `HashMap`s to a DB connection) is a call to `with_state` and another parameter to the handler.

```rust
use std::collections::{hash_map::Entry, HashMap};
use axum::extract::State;
use serde_json::json;
use std::sync::{Arc, Mutex};

type UsersDB = HashMap<String, u64>;

let app = Router::new()
    /* .. snip */
    .with_state(Arc::new(Mutex::new(UsersDB::new())));


async fn create_user(
    State(state): State<Arc<Mutex<UsersDB>>>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.lock().unwrap().entry(payload.username) {
        Entry::Occupied(_) => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "User already exists" })),
        ),
        Entry::Vacant(entry) => {
            entry.insert(1337);
            (StatusCode::CREATED, Json(json!(1337)))
        }
    }
}
```

And `tower-http` has useful middlewares that can be added with a call to `layer`.

```rust
let app = Router::new()
    /* .. snip */
    .layer(ValidateRequestHeaderLayer::bearer("hello2025"));
```

We do use `warp` from time to time, usually when we need a "one-off" HTTP server for a test or something similar.

```rust
warp::post()
    .and(warp::path("users"))
    .and(warp::body::json())
    .map(|mut payload: CreateUser| {
        warp::reply::json(&1337)
    });
```

As for HTTP(S) clients, we use `reqwest`, and we use the `rustls` backend (mostly to avoid OpenSSL, both during build and deployment).

One-off requests are what you would expect.

```rust
let body = reqwest::get("https://www.rust-lang.org")
    .await?
    .text()
    .await?;
```

But for production use-cases, you mostly want to use `ClientBuilder`.

```rust
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
```

And you can do pretty much anything HTTP-related (including multipart, body streaming and more) with the many methods of `RequestBuilder`.

```rust
let body = client.post("http://localhost:3000/users")
    .bearer_auth("hello2025")
    .json(&json!({ "username": "root" }))
    .timeout(Duration::from_secs(5))
    .send()
    .await?;
```

Side note: We usually disable HTTP2 because multiple servers and clients (in Rust, NodeJS and Python) have problems with sessions hanging indefinitely at random.

Sometimes you must to use `protobufs` (🙄).

`prost` is the crate to generate and consume `protobufs` data, and `tonic` is the crate for building gRPC clients and servers (which use `prost` for the `protobufs` part).

For RabbitMQ, we use the `lapin` crate (we use the official `rabbitmq-stream-client` for streams, but we are moving away from them entirely).

Side note 2: Most of our workloads would be fine with plain synchronous I/O. But the highest quality libraries are `async`-first (`tonic` and `reqwest` are good examples of this), and for the few cases we do care about it, it is easier to lean into the `async` lifestyle for everything.

## Serialization

Almost everything serialization-related is covered by `serde`, and we tend to use JSON as the serialization format (and `serde_json` to generate/consume it).

Sometimes JSON isn't really a good fit (binary data, size constraints), and using `bincode` with `serde` is great for such use cases.

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World(Vec<Entity>);

let json = serde_json::to_string(&world).unwrap();
let encoded: Vec<u8> = bincode::serialize(&world).unwrap();

assert_eq!(json.len(), 32);
assert_eq!(encoded.len(), 16);
```

There are a few additions worth noting:

- `serde_with` is helpful when small tweaks to the serialization/deserialization are needed
- `humantime-serde` is very handy for human APIs that accept durations (config file, CLI parameters)
-  `serde_qs` for dealing with querystrings

```rust
#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(with = "humantime_serde")]
    timeout: Duration,
}

let config: Config = serde_json::from_str(r#"{"timeout":"3m"}"#).unwrap();
assert_eq!(config.timeout, Duration::from_secs(180));
```

## Error Handling

For libraries, use `thiserror`. For apps, use `anyhow`.

Using `thiserror` allows you to expose an API of possible errors, which requires more effort and is usually only really worth it for libs (do you want to match on specific types of errors and handle them differently?).

You can still use the `#[from]` attribute to get a nice `?` behavior in the library, but you will need to manually construct the more nuanced errors.

For applications, using `anyhow` with its `.context(..)` function removes a lot of the friction of error handling while generating rich error messages. 

```rust
use anyhow::Context;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyLibError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("got an invalid code `{0}`")]
    InvalidCode(u32),
}

fn my_lib_function() -> Result<u32, MyLibError> {
    let hosts = std::fs::read_to_string("/etc/hosts")?;
    Err(MyLibError::InvalidCode(42))
}

fn my_app_function() -> Result<(), anyhow::Error> {
    let res = my_lib_function();

    let res = match res {
        Err(MyLibError::IoError(_)) => todo!("implement retry logic"),
        res => res,
    };

    let _valid_code = res.context("Failed to get a valid code from my_lib_function")?;

    Ok(())
}
```

Calling `my_app_function().unwrap()` will print:

```
thread 'main' panicked at src/error_handling.rs:30:23:
called `Result::unwrap()` on an `Err` value: Failed to get a valid code from my_lib_function

Caused by:
    got an invalid code `42`
```

## Testing

While the built-in `#[test]` attribute is great, the `rstest` crate brings in a bunch of features I missed after working for years with Python's `pytest`.

Re-using tedious-to-construct data and validating the same test against multiple inputs are the backbone of adequate test coverage (even though it can be tempting to overuse `case`s).

```rust
use rstest::{fixture, rstest};

#[fixture]
fn username() -> &'static str {
    "zelda"
}

#[fixture]
async fn db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE users (
            username TEXT PRIMARY KEY,
            fullname TEXT
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

#[fixture]
async fn db_with_user(username: &str, #[future] db: SqlitePool) -> SqlitePool {
    let db = db.await;

    sqlx::query(
        r#"
        INSERT INTO users (username, fullname)
        VALUES ($1, $2)
        "#,
    )
    .bind(username.to_string())
    .bind(format!("Test User {username}"))
    .execute(&db)
    .await
    .unwrap();

    db
}

#[rstest]
#[tokio::test]
async fn test_fetch_profile(username: &str, #[future] db_with_user: SqlitePool) {
    let db = db_with_user.await;

    let res = fetch_profile(username, &db).await;
    assert!(res.is_ok());

    let res = fetch_profile("link", &db).await;
    assert!(res.is_err());
}
```

## Lightning Round I

### (Non Cryptographic) Hashing

Cryptography is hard, but non cryptographic hashing can be useful in a lot of situations (_but only when you have trusted inputs_).

If you need a usize / `HashMap`-style hash, `rustc-hash` is a solid choice. It's much faster than the default hasher.

```rust
let mut hashmap = HashMap::<&'static str, usize, rustc_hash::FxBuildHasher>::default(); 
// or, `use rustc_hash::FxHashMap as HashMap`.

hashmap.insert("black", 0);
hashmap.insert("white", 255);
```

If you need to hash some medium-to-large string or bytes, and you want it to be consistent everywhere, `sha1_smol` is my go-to: easy to recreate the results in the terminal, no dependencies (other than `serde` if you want to).

Again, NOT cryptographic!

```rust
let mut m = sha1_smol::Sha1::new();
m.update(b"Hello 2025!");
let hash = m.digest().to_string();
assert_eq!(hash, "68a0e44442b100a1698afa962aa590cc5e1cbb71"); 
// Same as `echo -n 'Hello 2025!' | sha1sum`, if you trust your terminal.
```

### Allocators

Generally, Rust programs tend to allocate memory pretty liberally, and it's common to write a server that allocate some memory for each request.

Systems allocators don't always like this, and problems like memory fragmentation or unexplained high memory usage got us a few times over years.

For memory-intensive workloads, we switch to `jemalloc` using the `tikv-jemallocator` crate.

We can build a small server that does some non trivial work like resizing JPEG images using the `image` crate, and overload it with `vegeta`.

```rust
let app = Router::new().route("/resize/w/:w/h/:h", post(resize_image));

async fn resize_image(Path((width, height)): Path<(u32, u32)>, body: Body) -> Bytes {
    let mut body = body.into_data_stream();
    let mut data = vec![];

    while let Some(bytes) = body.next().await {
        let bytes = bytes.unwrap();
        data.extend(bytes);
    }

    let resized_image = tokio::task::spawn_blocking(move || {
        let img = image::ImageReader::with_format(Cursor::new(data), image::ImageFormat::Jpeg)
            .decode()
            .unwrap();
        img.resize(width, height, image::imageops::FilterType::Lanczos3)
            .into_bytes()
    })
    .await
    .unwrap();

    Bytes::from(resized_image)
}
```

```bash
for i in {1..100}; do
    WIDTH=$((RANDOM % 500 + 50))
    HEIGHT=$((RANDOM % 500 + 50))
    echo "POST http://localhost:3000/resize/w/$WIDTH/h/$HEIGHT" >> targets.txt
    echo "Content-Type: multipart/form-data" >> targets.txt
    echo "@./example.jpg" >> targets.txt
done
cat targets.txt | vegeta attack -duration=30s -rate=500 | vegeta report
```

On my machine, using `jemallocator` reduces the peak memory consumption from 2.1GB to 1.6GB, while also going back down to a ~300MB once the load is over.

```rust
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

## CLI

While `clap` is the more contemporary choice, I really like `argh` (and our code base is something like 50/50 between both).

My CLI stuff is usually a script for running some part of a thing I'm working on,
and I'll usually delay actually having a CLI as much as possible (and edit the params inline and recompile).

You can slap `argh` on a struct and get a nice interface, and go back to the task at hand.

```rust
use argh::FromArgs;

#[derive(FromArgs)]
/// Reach new heights.
struct Hello {
    /// turn on verbose output
    #[argh(switch, short = 'v')]
    verbose: bool,

    /// year to hello to
    #[argh(option)]
    year: i32,
}

let hello: Hello = argh::from_env();
```

Sometimes less is less; if you are building a CLI-first thing, `clap` is a safer choice.

## Date and Time

We deal exclusively with UTC timestamps, so we use `chrono`.

```rust
use chrono::prelude::*;
let dt = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
println!("Hello {}!", dt);

let last_second = dt - chrono::Duration::seconds(1);
```

If you care about timezones, `jiff` is the new kid on the block and has a more complete story for them (see [`jiff` vs `chrono`, `time`, and more](https://github.com/BurntSushi/jiff/blob/master/COMPARE.md)).

## Lightning Round II

Sometimes you need to customize the `Debug` impl of some struct, and `derivative` is useful if you don't want to manually implement it:

```rust
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
struct Auth {
    pub api_key: String,
    #[derivative(Debug="ignore")]
    pub api_secret: String,
}
```

For running benchmarks, `criterion` has nice reporting, is flexible with generating inputs to functions (for example with `iter_batched`),
and profiling is painless with `cargo flamegraph` (install with `cargo install flamegraph`).

For everything Python-related, use `pyo3` (I also wrote an [entire blog post](https://ohadravid.github.io/posts/2023-03-rusty-python/) about it).

If you work with UUIDs, the `uuid` crate supports anything you need: creating them, parsing them (including `serde` integration) and with support for all versions.

```rust
let uuid = uuid::Uuid::from_u128(0x1337_0042);
assert_eq!(uuid.to_string(), "00000000-0000-0000-0000-000013370042");
assert_eq!(uuid, uuid::uuid!("00000000-0000-0000-0000-000013370042"));
```

## SQL & ORMs

Just Use Postgres for Everything, amirite? But for that, you still need a bunch of client-side stuff - managing connections, running queries, building queries, handling migrations, etc etc.

The `SeaQL` ecosystem of crates (`sea-query`, `sea-orm`, `sea-orm-migration`) has everything you need, and more. We use them with the `sqlx` integration.

Check out their full [`axum_example`](https://github.com/SeaQL/sea-orm/tree/master/examples/axum_example), but here's a taste:

```rust
pub async fn update_post_by_id(
    db: &DbConn,
    id: i32,
    post_data: post::Model,
) -> Result<post::Model, DbErr> {
    let post: post::ActiveModel = post::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
        .map(Into::into)?;

    post::ActiveModel {
        id: post.id,
        title: Set(post_data.title.to_owned()),
        text: Set(post_data.text.to_owned()),
    }
    .update(db)
    .await
}
```

## RIP

Some crates that are with us no longer (except where we forgot to replace them):

- `lazy_static` - If you need a complex struct that is also `static`, now `LazyLock` is part of the standard library:

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

static HASHMAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "foo");
    m.insert(1, "bar");
    m.insert(2, "baz");
    m
});

dbg!(&*HASHMAP);
```

- `once_cell` - Similarly, if you need a global that is only initialized once, you can now use `OnceLock`:

```rust
use std::sync::OnceLock;
static CELL: OnceLock<usize> = OnceLock::new();

let value = CELL.get_or_init(|| 12345);
```

- `async_trait` - Almost RIP. Before Rust 1.75 you couldn't have `async` functions in traits at all, and now you can _almost_ always have them. And even when you can, you'll still get fun `use of async fn in public traits` warning. So just slap `#[async_trait::async_trait]` on them for a bit longer!

## Logging, Tracing and Metrics

The `tracing` crate is wonderful: basic usage is simplicity incarnate, but the possibilities are endless.

Set the env var `RUST_LOG=debug` and off you go.

```rust
use tracing::{info, debug};

#[tracing::instrument]
fn hello(year: usize) {
    println!("Hello, {}!", year);
    info!("Done saying hello");
}

pub fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    hello(2025);
}
```

You can `instrument` async calls dynamically which can be useful for injecting request-level data.

```rust
async fn goodbye() {
    println!("Goodbye!");
    debug!("Done saying goodbye");
}

#[tokio::main]
pub async fn main() {
    // .. snip ..
    goodbye()
        .instrument(info_span!("goodbye", year = 2024))
        .await;
}
```

For metrics, we use the `prometheus` crate.

Defining and using metrics is straightforward.

```rust
use prometheus::{register_int_counter, register_int_counter_vec, IntCounter, IntCounterVec};
use std::sync::LazyLock;

pub static REQUEST_COUNT: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!("request_count", "Number of requests received").unwrap()
});

pub static ERROR_COUNT: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!("error_count", "Number of errors by type", &["type"]).unwrap()
});


// In some function:
REQUEST_COUNT.inc();

// In an error flow:
ERROR_COUNT.with_label_values(&["conflict"]).inc();
```

We serve metrics using `axum`.

```rust
use axum::{routing::get, Router};

pub fn metrics_router() -> Router {
    Router::new().route("/metrics", get(metrics))
}

async fn metrics() -> String {
    prometheus::TextEncoder::new()
        .encode_to_string(&prometheus::gather())
        .unwrap()
}
```

Either running on a dedicated HTTP server, or merged to the main router (if the service is an internal HTTP server).

```rust
let app = Router::new()
    .route("/", get(root))
    .route("/users", post(create_user))
    /* .. snip .. */
    .merge(metrics::metrics_router());
```

```bash
$ curl -H "Authorization: Bearer hello2025" http://localhost:3000/metrics
# HELP error_count Number of errors by type
# TYPE error_count counter
error_count{type="conflict"} 1
# HELP request_count Number of requests received
# TYPE request_count counter
request_count 3
```

## Vectors, Arrays, ML

For us, most of the ML workloads use NVIDIA's Triton server (using `tonic` for the gRPC client and `half` for creating `f16`-type inputs), for which we compile ONNX models to TensorRT engines.

When testing with floats, `approx` is useful for quick comparisons:

```rust
use approx::assert_abs_diff_eq;
use half::f16;

let res = f16::from_f32(0.1) + f16::from_f32(0.2);

assert_abs_diff_eq!(f32::from(res), 0.3, epsilon = 1e-3);
assert_eq!(res.to_le_bytes(), [0xcc, 0x34]);
```

There are a few cases where running a model on the CPU makes more sense, and this is where we use `ort` crate to run the ONNX models directly (currently on version `2.0.0-rc.9`).

For actual input/output related work, we use `ndarray` which is the `numpy` equivalent for Rust ([ndarray::doc::ndarray_for_numpy_users](https://docs.rs/ndarray/latest/ndarray/doc/ndarray_for_numpy_users/index.html)).

```rust
use ndarray::{array, s, Array};
let a = array![[1, 2], [3, 4]];

assert_eq!(a.slice(s![.., 1]), Array::from_vec(vec![2, 4]));
```

For the more geometric code, the `nalgebra` crate has all the needed `Matrix` and `Vector` operations.

```rust
use nalgebra::Matrix2;

let m = Matrix2::new(1., 2.,
                     3., 4.);

assert_eq!(m.determinant(), -2.);
assert_eq!(m.is_invertible(), true);
```

## Editions, nightly vs stable

We use the 2021 edition for all our code, and the transition from 2018 was completely smooth.

We only use the `stable` compiler, which wasn't true when I started using Rust at $work^1.

Both of these seem obvious, which only makes it an even bigger success of the Rust project.

We do use `Bazel` as our build system (primarily because we mix Rust and Python), but that's a topic for a different post.

## Wrapping up

I hope you found this helpful! I wonder how much this list will change next year? In five? \
And of course, there are so many other awesome crates. \
What are some of yours? Discuss on [r/rust](https://www.reddit.com/r/rust/comments/1hafdai/state_of_the_crates_2025/) 🦀.

## TL;DR

Even though we end up building almost 500 crates, a clean `cargo build` on my M3 MacBook Pro takes less than 40 seconds (with a prebuilt jemalloc),
and `cargo check` is instant.

The rest of the code is also available on GitHub at [ohadravid/state-of-the-crates](https://github.com/ohadravid/state-of-the-crates).

Special thanks to [Omer](https://github.com/omerbenamram) and [Yuval](https://github.com/yogevyuval) for reviewing earlier drafts of this article.
