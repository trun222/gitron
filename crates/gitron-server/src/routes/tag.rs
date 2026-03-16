use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{error::GitError, remote, repository, types::*};

#[derive(Deserialize)]
pub struct CreateTagRequest {
    path: String,
    name: String,
    #[serde(rename = "targetOid")]
    target_oid: String,
    message: Option<String>,
}

pub async fn create_tag(
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<Tag>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let tag = repository::create_tag(&repo, &req.name, &req.target_oid, req.message.as_deref())
        .map_err(err)?;
    Ok(Json(tag))
}

#[derive(Deserialize)]
pub struct DeleteTagRequest {
    path: String,
    name: String,
}

pub async fn delete_tag(
    Json(req): Json<DeleteTagRequest>,
) -> Result<Json<()>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::delete_tag(&repo, &req.name).map_err(err)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct MoveTagRequest {
    path: String,
    name: String,
    #[serde(rename = "targetOid")]
    target_oid: String,
}

pub async fn move_tag(
    Json(req): Json<MoveTagRequest>,
) -> Result<Json<Tag>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let tag = repository::move_tag(&repo, &req.name, &req.target_oid).map_err(err)?;
    Ok(Json(tag))
}

#[derive(Deserialize)]
pub struct PushTagRequest {
    path: String,
    #[serde(rename = "remoteName")]
    remote_name: String,
    #[serde(rename = "tagName")]
    tag_name: String,
    #[serde(default)]
    force: bool,
}

pub async fn push_tag(
    Json(req): Json<PushTagRequest>,
) -> Result<Json<PushResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = remote::push_tag(&workdir, &req.remote_name, &req.tag_name, req.force)
        .await
        .map_err(err)?;
    Ok(Json(result))
}

pub async fn delete_remote_tag(
    Json(req): Json<PushTagRequest>,
) -> Result<Json<()>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    remote::delete_remote_tag(&workdir, &req.remote_name, &req.tag_name)
        .await
        .map_err(err)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct ListRemoteTagsRequest {
    path: String,
    #[serde(rename = "remoteName")]
    remote_name: String,
}

pub async fn list_remote_tags(
    Json(req): Json<ListRemoteTagsRequest>,
) -> Result<Json<Vec<RemoteTagInfo>>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let tags = remote::list_remote_tags(&workdir, &req.remote_name)
        .await
        .map_err(err)?;
    Ok(Json(tags))
}

fn get_workdir(path: &str) -> Result<String, (StatusCode, String)> {
    let repo = repository::open(path).map_err(err)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Other("Bare repository".into()))
        .map_err(err)?
        .to_string_lossy()
        .to_string();
    Ok(workdir)
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
