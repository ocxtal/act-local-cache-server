use crate::file::*;
use crate::utils::{glob_in, parse_range};
use log::info;
use serde_derive::{Deserialize, Serialize};
use warp::http::{Response, StatusCode};
use warp::hyper::body::Bytes;
use warp::path::Tail;
use warp::reply::{json, with_status, Json, WithStatus};

#[derive(Deserialize, Clone, Debug)]
pub struct VersionQuery {
    #[serde(rename = "api-version")]
    api_version: String,
}

#[derive(Serialize, Clone, Debug)]
struct StatusResponse {
    status: String,
}

fn unsupported_version() -> WithStatus<Json> {
    info!("[unsupported_version] unsupported api-version");

    with_status(
        json(&StatusResponse {
            status: "unsupported api-version".to_string(),
        }),
        StatusCode::BAD_REQUEST,
    )
}

#[derive(Serialize, Clone, Debug)]
struct UrlResponse {
    status: String,

    #[serde(rename = "fileContainerResourceUrl")]
    url: String,
}

pub fn get_artifact_upload_url(
    host: &str,
    run_id: String,
    version: VersionQuery,
) -> WithStatus<Json> {
    info!("[get_artifact_upload_url] run_id = {run_id}, version = {version:?}");

    // TODO: unsupported version response
    if version.api_version != "6.0-preview" {
        return unsupported_version();
    }

    let res = UrlResponse {
        status: "success".to_string(),
        url: format!("{host}/upload/{run_id}"),
    };
    info!("[get_artifact_upload_url] response = {res:?}");

    with_status(json(&res), StatusCode::OK)
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemPathQuery {
    #[serde(rename = "itemPath")]
    path: String,
}

pub fn upload_artifact(
    run_id: String,
    path: ItemPathQuery,
    encoding: Option<String>,
    range: Option<String>,
    input: Bytes,
) -> Json {
    info!(
        "[upload_artifact] run_id = {run_id}, path = {path:?}, range = {range:?}, input = <{} bytes>",
        input.len()
    );

    // format chunk prefix that can be safely sorted into the original chunk order
    // (this assumes total bytes being less than 1TB)
    let path = path.path;
    let path = format!(".act_local_cache/artifacts/{run_id}/{path}");

    // workaround for gzipped stream
    let is_gzip = encoding.as_deref() == Some("gzip");

    let range = range.as_deref().map_or(0..input.len(), parse_range);
    save_file(&path, is_gzip, range.start, &input.slice(..));

    let res = StatusResponse {
        status: "success".to_string(),
    };
    info!("[upload_artifact] response = {res:?}");

    json(&res)
}

#[derive(Deserialize, Clone, Debug)]
pub struct FinalizeQuery {
    #[serde(rename = "Size")]
    size: usize,
}

pub fn finalize_artifact(
    run_id: String,
    version: VersionQuery,
    input: FinalizeQuery,
) -> WithStatus<Json> {
    info!("[finalize_artifact] run_id = {run_id}, version = {version:?}, input = {input:?}");

    if version.api_version != "6.0-preview" {
        return unsupported_version();
    }

    let size = finalize_files(&format!(".act_local_cache/artifacts/{run_id}"), "**/*");
    if size != input.size {
        let expected = input.size;
        info!("[finalize_artifact] upload size differs (expected = {expected}, actual = {size})");
    }

    // TODO: check total file size (and concatenate files if needed)
    let res = StatusResponse {
        status: "success".to_string(),
    };
    info!("[finalize_artifact] response = {res:?}");

    with_status(json(&res), StatusCode::OK)
}

#[derive(Serialize, Clone, Debug)]
struct UrlArrayElement {
    name: String,

    #[serde(rename = "fileContainerResourceUrl")]
    url: String,
}

#[derive(Serialize, Clone, Debug)]
struct UrlArrayResponse {
    status: String,
    count: usize,
    value: Vec<UrlArrayElement>,
}

pub fn get_artifact_download_url(
    host: &str,
    run_id: String,
    version: VersionQuery,
) -> WithStatus<Json> {
    info!("[get_artifact_download_url] run_id = {run_id}, version = {version:?}");

    // TODO: unsupported version response
    if version.api_version != "6.0-preview" {
        return unsupported_version();
    }

    let dir = format!(".act_local_cache/artifacts/{run_id}");
    let paths = glob_in(&dir, "*").unwrap();

    let mut array = Vec::new();
    for path in &paths {
        let path = path.to_str().unwrap();
        array.push(UrlArrayElement {
            name: path.to_string(),
            url: format!("{host}/download/{run_id}"),
        });
    }

    let count = array.len();
    let res = UrlArrayResponse {
        status: "success".to_string(),
        count,
        value: array,
    };
    info!("[get_artifact_download_url] response = PathArrayResponse {{ status: \"success\", count: {count}, value: <{count} items> }}");

    with_status(json(&res), StatusCode::OK)
}

#[derive(Serialize, Clone, Debug)]
struct PathArrayElement {
    path: String,

    #[serde(rename = "itemType")]
    item_type: String,

    #[serde(rename = "contentLocation")]
    url: String,
}

#[derive(Serialize, Clone, Debug)]
struct PathArrayResponse {
    status: String,
    count: usize,
    value: Vec<PathArrayElement>,
}

pub fn enumerate_artifacts(host: &str, run_id: String) -> WithStatus<Json> {
    info!("[enumerate_artifacts] run_id = {run_id}");

    let files = list_all_files(&format!(".act_local_cache/artifacts/{run_id}"));

    let mut array = Vec::new();
    for file in files {
        let url = format!("{host}/download/{run_id}/{file}");
        array.push(PathArrayElement {
            path: file,
            item_type: "file".to_string(),
            url,
        });
    }

    let count = array.len();
    let res = PathArrayResponse {
        status: "success".to_string(),
        count,
        value: array,
    };
    info!("[enumerate_artifacts] response = PathArrayResponse {{ status: \"success\", count: {count}, value: <{count} items> }}");

    with_status(json(&res), StatusCode::OK)
}

pub fn download_artifact(run_id: String, path: Tail, range: Option<String>) -> Response<Vec<u8>> {
    info!("[download_artifact] run_id = {run_id}, path = {path:?}, range = {range:?}");

    let path = path.as_str();
    let (is_gzip, data) = dump_file(
        &format!(".act_local_cache/artifacts/{run_id}/{path}"),
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
    info!("[download_artifact] response = <{len} bytes>");

    header.body(data).unwrap()
}
