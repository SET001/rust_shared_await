use rand::Rng;
use tower::Layer;

use axum::{
  extract::{Path, Request, State},
  middleware::map_request,
  routing::get,
  Router, ServiceExt,
};
use futures::{future::Shared, FutureExt};
use std::{collections::HashMap, future::Future, net::Ipv4Addr, pin::Pin, sync::Mutex};
mod db;

pub use db::DB;
use std::sync::Arc;

type QueryCache =
  Arc<Mutex<HashMap<u32, (Shared<Pin<Box<dyn Future<Output = String> + Send>>>, i32)>>>;

#[derive(Clone)]
struct AppState {
  db: DB,
  query_cache: QueryCache,
}
impl AppState {
  fn new() -> Self {
    Self {
      db: DB {},
      query_cache: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

//	middleware for benchmark testing.
async fn random_ip(mut request: Request) -> Request {
  if regex::Regex::new(r"lookup_bench/.*")
    .unwrap()
    .is_match(request.uri().path())
  {
    let mut rng = rand::rng();
    let new_path = format!(
      "/lookup_bench/{}.{}.{}.{}",
      rng.random_range(0..255),
      rng.random_range(0..255),
      rng.random_range(0..255),
      rng.random_range(0..255)
    );
    *request.uri_mut() = new_path.parse().unwrap();
  };

  request
}

#[tokio::main]
async fn main() {
  /*
     * Lets imagine:
     * You have a system where you are getting a lot of concurrent http requests
     * On each request you need to do DB query with the requested ip to get country from which is user
     * what u will do and which bottlenecks do u have?
     * how u will implement this ?
     ! IPs are only in range of ipv4
     ! IPs can be repeated
     ! Lets assume that each query execution is 1.5s
  */
  let app_state = AppState::new();

  let app = Router::new()
    .route("/lookup/{ip}", get(ip))
    .route("/lookup_bench/{ip}", get(ip))
    .route("/stats", get(stats))
    .with_state(app_state);

  let middleware = map_request(random_ip);
  let app_with_middleware = middleware.layer(app);
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, app_with_middleware.into_make_service())
    .await
    .unwrap();
}

async fn stats(
  State(AppState {
    query_cache: cache, ..
  }): State<AppState>,
) -> String {
  let cache = cache.lock().unwrap();
  format!("Cache size: {}", cache.len())
}

async fn ip(
  State(AppState {
    query_cache, db, ..
  }): State<AppState>,
  Path(ip): Path<Ipv4Addr>,
) -> String {
  println!("ip: {:?}", ip);
  let mut future = query_cache
    .lock()
    .unwrap()
    .entry(ip.to_bits())
    .and_modify(|future| future.1 += 1)
    .or_insert_with(move || (async move { db.get_country(ip).await }.boxed().shared(), 1))
    .clone();
  let res = future.0.await;
  future.1 -= 1;
  if future.1 == 0 {
    query_cache.lock().unwrap().remove(&ip.to_bits());
  }
  res
}
