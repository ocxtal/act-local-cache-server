mod artifact; // actions/upload-artifact@v3 and actions/download-artifact@v3
mod cache; // actions/cache@v3
mod file;
mod utils;

use crate::artifact::*;
use crate::cache::*;
use clap::Parser;
use once_cell::sync::OnceCell;
use std::net::Ipv4Addr;
use warp::Filter;

#[derive(Parser, Clone, Debug)]
#[command(version, about = "Local artifact/cache server for use with nektos/act", long_about = None)]
struct Args {
    #[clap(short, long, help = "Server address", default_value = "127.0.0.1")]
    address: Ipv4Addr,

    #[clap(short, long, help = "Server port", default_value = "8000")]
    port: u16,

    #[clap(short, long, help = "Authentication token")]
    token: Option<String>,
}

#[derive(Debug)]
struct ServerParams {
    host: String, // http://{address}:{port}
    auth: String, // Bearer {token}
}

static SERVER_PARAMS: OnceCell<ServerParams> = OnceCell::new();

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args = Args::parse();
    let token = args.token.unwrap_or_else(|| {
        std::env::var("ACT_LOCAL_CACHE_SERVER_TOKEN").unwrap_or_else(|_| "token".to_string())
    });

    // format host and auth header
    let params = ServerParams {
        host: format!("http://{}:{}", args.address, args.port),
        auth: format!("Bearer {}", token),
    };
    SERVER_PARAMS.set(params).unwrap();

    // POST "/<run_id>/artifacts?api-version"
    let path_get_artifact_upload_url =
        warp::path!("_apis" / "pipelines" / "workflows" / String / "artifacts")
            .and(warp::post())
            .and(warp::header::exact_ignore_case(
                "Authorization",
                &SERVER_PARAMS.get().unwrap().auth,
            ))
            .and(warp::query::<VersionQuery>())
            .map(|x, y| get_artifact_upload_url(&SERVER_PARAMS.get().unwrap().host, x, y));

    // GET "/<run_id>/artifacts?api-version"
    let path_get_artifact_download_url =
        warp::path!("_apis" / "pipelines" / "workflows" / String / "artifacts")
            .and(warp::get())
            .and(warp::header::exact_ignore_case(
                "Authorization",
                &SERVER_PARAMS.get().unwrap().auth,
            ))
            .and(warp::query::<VersionQuery>())
            .map(|x, y| get_artifact_download_url(&SERVER_PARAMS.get().unwrap().host, x, y));

    // PATCH "/<run_id>/artifacts?api-version"
    let path_finalize_artifact =
        warp::path!("_apis" / "pipelines" / "workflows" / String / "artifacts")
            .and(warp::patch())
            .and(warp::header::exact_ignore_case(
                "Authorization",
                &SERVER_PARAMS.get().unwrap().auth,
            ))
            .and(warp::query::<VersionQuery>())
            .and(warp::body::content_length_limit(1024))
            .and(warp::body::json())
            .map(finalize_artifact);

    // GET "/download/<run_id>"
    let path_enumerate_artifacts = warp::path::param::<String>()
        .and(warp::path::end())
        .map(|x| enumerate_artifacts(&SERVER_PARAMS.get().unwrap().host, x));

    // GET "/download/<run_id>/<path>"
    let path_download_artifact = warp::path::param::<String>()
        .and(warp::path::tail())
        .and(warp::header::optional::<String>("Content-Range"))
        .map(download_artifact);

    // either of two above
    let path_download_or_enumerate_artifact = warp::path("download")
        .and(warp::get())
        .and(warp::header::exact_ignore_case(
            "Authorization",
            &SERVER_PARAMS.get().unwrap().auth,
        ))
        .and(path_enumerate_artifacts.or(path_download_artifact));

    // PUT "/upload/<run_id>"
    let path_upload_artifact = warp::path!("upload" / String)
        .and(warp::put())
        .and(warp::query::<ItemPathQuery>())
        .and(warp::header::optional::<String>("Content-Encoding"))
        .and(warp::header::optional::<String>("Content-Range"))
        .and(warp::body::content_length_limit(64 * 1024 * 1024))
        .and(warp::body::bytes())
        .map(upload_artifact);

    // POST _apis/artifactcache/caches/
    let path_reserve_cache = warp::path!("_apis" / "artifactcache" / "caches")
        .and(warp::post())
        .and(warp::header::exact_ignore_case(
            "Authorization",
            &SERVER_PARAMS.get().unwrap().auth,
        ))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .map(reserve_cache);

    // PATCH _apis/artifactcache/caches/:cacheId
    let path_upload_cache = warp::path!("_apis" / "artifactcache" / "caches" / String / String)
        .and(warp::patch())
        .and(warp::header::exact_ignore_case(
            "Authorization",
            &SERVER_PARAMS.get().unwrap().auth,
        ))
        .and(warp::header::optional::<String>("Content-Encoding"))
        .and(warp::header::optional::<String>("Content-Range"))
        .and(warp::body::bytes())
        .map(upload_cache);

    // POST _apis/artifactcache/caches/:cacheId body = { size: filesize }
    let path_finalize_cache = warp::path!("_apis" / "artifactcache" / "caches" / String / String)
        .and(warp::post())
        .and(warp::header::exact_ignore_case(
            "Authorization",
            &SERVER_PARAMS.get().unwrap().auth,
        ))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .map(finalize_cache);

    // GET _apis/artifactcache/cache?keys=${encodeURIComponent(keys.join(','))}&version=${version}`;
    // -> { archiveLocation }
    let path_enumerate_cache = warp::path!("_apis" / "artifactcache" / "cache")
        .and(warp::get())
        .and(warp::header::exact_ignore_case(
            "Authorization",
            &SERVER_PARAMS.get().unwrap().auth,
        ))
        .and(warp::query::<EnumerateQuery>())
        .map(|x| enumerate_caches(&SERVER_PARAMS.get().unwrap().host, x));

    // GET _apis/artifactcache/cache/:cacheId
    let path_download_cache =
        warp::path!("_apis" / "artifactcache" / "cache" / "download" / String / String)
            .and(warp::get())
            .and(warp::header::optional::<String>("Content-Range"))
            .map(download_cache);

    let routes = warp::any().and(
        path_get_artifact_upload_url
            .or(path_get_artifact_download_url)
            .or(path_finalize_artifact)
            .or(path_download_or_enumerate_artifact)
            .or(path_upload_artifact)
            .or(path_reserve_cache)
            .or(path_upload_cache)
            .or(path_finalize_cache)
            .or(path_enumerate_cache)
            .or(path_download_cache),
    );

    warp::serve(routes).run((args.address, args.port)).await;
}
