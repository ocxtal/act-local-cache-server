
use serde_derive::{Deserialize, Serialize};
use warp::hyper::body::Bytes;
use warp::path::Tail;
use warp::reply::{json, Json};

#[derive(Serialize, Clone, Debug)]
struct StatusResponse {
    status: String,
}

// POST http://localhost:8000/_apis/artifactcache/caches
// -> cacheId
#[derive(Deserialize, Clone, Debug)]
struct ReserveCacheQuery {
    key: String,
    version: String,
    size: usize,
}

#[derive(Serialize, Clone, Debug)]
struct ReserveCacheResponse {
    status: String,

    #[serde(rename = "cacheId")]
    cache_id: String,
}

fn reserve_cache(query: ReserveCacheQuery) -> Json {
    let res = ReserveCacheResponse {
        status: "success".to_string(),
        cache_id: format!("{}/{}", query.version, query.key),
    };
    json(&res)
}

#[derive(Deserialize, Clone, Debug)]
struct FinalizeQuery {
    #[serde(rename = "Size")]
    size: usize,
}

fn finalize_cache(version: String, key: String, input: FinalizeQuery) -> Json {
    let res = StatusResponse {
        status: "success".to_string(),
    };
    json(&res)
}

fn upload_cache(version: String, key: String, encoding: Option<String>, range: Option<String>, input: Bytes) -> Json {
    let res = StatusResponse {
        status: "success".to_string(),
    };
    json(&res)
}

#[derive(Deserialize, Clone, Debug)]
struct EnumerateQuery {
    keys: Vec<String>,
    version: String,
}

#[derive(Serialize, Clone, Debug)]
struct EnumerateResponse {
    status: String,
    count: usize,

}

// fn enumerate_cache(query: EnumerateQuery) -> Json {

// }

// PUT http://localhost:8000/_apis/artifactcache/caches/:cacheId
// PATCH http://localhost:8000/_apis/artifactcache/caches/:cacheId body = { size: filesize }
// GET http://localhost:8000/_apis/artifactcache/cache?keys=${encodeURIComponent(keys.join(','))}&version=${version}`;
// -> { archiveLocation }

pub fn print_cache(path: Tail, input: Bytes) -> Json {
    eprintln!("[root] request");

    eprintln!("path({:?}), input({:?})", path, input);

    let res = StatusResponse { status: "ok".to_string() };
    json(&res)
}
