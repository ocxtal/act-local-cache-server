mod artifact; // actions/upload-artifact@v3 and actions/download-artifact@v3
mod cache; // actions/cache@v3
mod file;
mod utils;

use crate::artifact::*;
use crate::cache::*;
use pretty_env_logger;
use warp::Filter;

fn root() -> String {
    eprintln!("[root] request");
    "act-local-cache-server".to_string()
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let address = [127, 0, 0, 1];
    let port = 8000;

    // "/" -> root
    let path_root = warp::path::end().map(root);

    // POST "/<run_id>/artifacts?api-version"
    let path_get_upload_url =
        warp::path!("_apis" / "pipelines" / "workflows" / String / "artifacts")
            .and(warp::post())
            .and(warp::query::<VersionQuery>())
            .map(get_upload_url);

    // GET "/<run_id>/artifacts?api-version"
    let path_get_download_url =
        warp::path!("_apis" / "pipelines" / "workflows" / String / "artifacts")
            .and(warp::get())
            .and(warp::query::<VersionQuery>())
            .map(get_download_url);

    // PATCH "/<run_id>/artifacts?api-version"
    let path_finalize_artifact =
        warp::path!("_apis" / "pipelines" / "workflows" / String / "artifacts")
            .and(warp::patch())
            .and(warp::query::<VersionQuery>())
            .and(warp::body::content_length_limit(1024))
            .and(warp::body::json())
            .map(finalize_artifact);

    // GET "/download/<run_id>"
    let path_enumerate_files = warp::path::param::<String>()
        .and(warp::path::end())
        .map(enumerate_files);

    // GET "/download/<run_id>/<path>"
    let path_download_file = warp::path::param::<String>()
        .and(warp::path::tail())
        .and(warp::header::optional::<String>("Content-Range"))
        .map(download_file);

    // either of two above
    let path_download_or_enumerate = warp::path("download")
        .and(warp::get())
        .and(path_enumerate_files.or(path_download_file));

    // PUT "/upload/<run_id>"
    let path_upload_file = warp::path!("upload" / String)
        .and(warp::put())
        .and(warp::query::<ItemPathQuery>())
        .and(warp::header::optional::<String>("Content-Encoding"))
        .and(warp::header::optional::<String>("Content-Range"))
        .and(warp::body::content_length_limit(32 * 1024 * 1024))
        .and(warp::body::bytes())
        .map(upload_file);

    let path_cache = warp::path!("_apis" / "artifactcache" / "caches")
        .and(warp::path::tail())
        .and(warp::body::bytes())
        .map(print_cache);

    let routes = warp::any().and(
        path_root
            .or(path_get_upload_url)
            .or(path_get_download_url)
            .or(path_finalize_artifact)
            .or(path_download_or_enumerate)
            .or(path_upload_file)
            .or(path_cache),
    );

    warp::serve(routes).run((address, port)).await;
}
