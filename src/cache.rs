use crate::file::*;
use crate::utils::parse_range;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;
use warp::http::{Response, StatusCode};
use warp::hyper::body::Bytes;
use warp::reply::{json, with_status, Json, WithStatus};

#[derive(Serialize, Clone, Debug)]
struct StatusResponse {
    status: String,
}

// POST http://localhost:8000/_apis/artifactcache/caches
// -> cacheId
#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct ReserveCacheQuery {
    key: String,
    version: String,

    #[serde(rename = "cacheSize")]
    size: usize,
}

#[derive(Serialize, Clone, Debug)]
struct ReserveCacheResponse {
    status: String,

    #[serde(rename = "cacheId")]
    cache_id: String,
}

pub fn reserve_cache(query: ReserveCacheQuery) -> Json {
    eprintln!("[reserve_cache] query = {query:?}");

    let res = ReserveCacheResponse {
        status: "success".to_string(),
        cache_id: format!("{}/{}", query.key, query.version),
    };
    eprintln!("[reserve_cache] response = {res:?}");

    json(&res)
}

pub fn upload_cache(
    key: String,
    version: String,
    encoding: Option<String>,
    range: Option<String>,
    input: Bytes,
) -> Json {
    eprintln!(
        "[upload_cache] version = {version}, key = {key}, encoding = {encoding:?}, range = {range:?}, input = <{} bytes>",
        input.len()
    );

    // format chunk prefix that can be safely sorted into the original chunk order
    // (this assumes total bytes being less than 1TB)
    let path = format!(".act_local_cache/caches/{key}/{version}");

    // workaround for gzipped stream
    let is_gzip = encoding.as_deref() == Some("gzip");

    let range = range.as_deref().map_or(0..input.len(), parse_range);
    save_file(&path, is_gzip, range.start, &input.slice(..));

    let res = StatusResponse {
        status: "success".to_string(),
    };
    eprintln!("[upload_cache] response = {res:?}");

    json(&res)
}

#[derive(Deserialize, Clone, Debug)]
pub struct FinalizeQuery {
    size: usize,
}

pub fn finalize_cache(key: String, version: String, input: FinalizeQuery) -> WithStatus<Json> {
    eprintln!("[finalize_cache] version = {version}, key = {key}, input = {input:?}");

    let size = finalize_files(".act_local_cache/caches", &format!("{key}/{version}*"));
    if size != input.size {
        let expected = input.size;
        eprintln!("[finalize_cache] upload size differs (expected = {expected}, actual = {size})");
    }

    // TODO: check total file size (and concatenate files if needed)
    let res = StatusResponse {
        status: "success".to_string(),
    };
    eprintln!("[finalize_cache] response = {res:?}");

    with_status(json(&res), StatusCode::OK)
}

#[derive(Deserialize, Clone, Debug)]
pub struct EnumerateQuery {
    keys: String,
    version: String,
}

#[derive(Serialize, Clone, Debug)]
struct UrlResponse {
    status: String,

    #[serde(rename = "archiveLocation")]
    url: String,

    #[serde(rename = "cacheKey")]
    key: String,
}

pub fn enumerate_caches(query: EnumerateQuery) -> WithStatus<Json> {
    eprintln!("[enumerate_caches] query = {query:?}");

    let version = query.version;

    let mut array = Vec::new();
    for key in query.keys.split(',') {
        let path = format!(".act_local_cache/caches/{key}/{version}");

        if Path::new(&path).exists() {
            let url =
                format!("http://127.0.0.1:8000/_apis/artifactcache/cache/download/{key}/{version}");
            array.push(UrlResponse {
                status: "success".to_string(),
                url,
                key: key.to_string(),
            });
        }
    }

    if let Some(res) = array.pop() {
        eprintln!("[enumerate_caches] response = {res:?}");
        with_status(json(&res), StatusCode::OK)
    } else {
        let res = StatusResponse {
            status: "not found".to_string(),
        };
        eprintln!("[enumerate_caches] response = {res:?}");

        with_status(json(&res), StatusCode::NOT_FOUND)
    }
}

pub fn download_cache(key: String, version: String, range: Option<String>) -> Response<Vec<u8>> {
    eprintln!("[download_cache] version = {version}, key = {key}, range = {range:?}");

    let (is_gzip, data) = dump_file(
        &format!(".act_local_cache/caches/{key}/{version}"),
        range.as_deref().map(parse_range),
    );

    // workaround for gzipped stream
    let header = Response::builder().header("Content-Type", "application/octet-stream");
    let header = if is_gzip {
        header.header("Content-Encoding", "gzip")
    } else {
        header
    };

    let len = data.len();
    eprintln!("[download_cache] response = <{len} bytes>");

    header.body(data).unwrap()
}
