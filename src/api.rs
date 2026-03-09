use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{ApiError, ApiResult, auth};
use crate::{AppState, codegen, db::ShortUrl};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/shorten", post(shorten))
        .route("/api/urls", get(get_urls))
        .route("/api/urls/{slug}", delete(delete_url))
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ShorteningMode {
    #[default]
    RandomString,
    RandomWords,
    Custom,
}

#[derive(Deserialize)]
pub struct ShortenRequest {
    pub url: String,
    pub mode: Option<ShorteningMode>,
    pub custom_slug: Option<String>,
}

pub async fn shorten(
    jar: CookieJar,
    State(state): State<AppState>,
    Json(body): Json<ShortenRequest>,
) -> ApiResult<(StatusCode, Json<ShortUrl>)> {
    auth::authentication_gate(&jar, &state)?;

    let url = body.url.trim().to_string();
    if url.is_empty() {
        return Err(ApiError::bad_request("url is required"));
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ApiError::bad_request(
            "url must start with http:// or https://",
        ));
    }

    let slug = match body.mode.unwrap_or_default() {
        ShorteningMode::Custom => body
            .custom_slug
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or(ApiError::bad_request(
                "custom_slug is required for mode=custom",
            ))
            .and_then(codegen::validate_slug)?,
        ShorteningMode::RandomWords => loop {
            let candidate = codegen::random_words(
                state.config.shortener.random_word_count,
                &state.config.shortener.word_separator,
            );
            if state.db.url_exists(&candidate).is_ok_and(|r| !r) {
                break candidate;
            }
        },
        ShorteningMode::RandomString => loop {
            let candidate = codegen::random_string(state.config.shortener.random_string_length);
            if state.db.url_exists(&candidate).is_ok_and(|r| !r) {
                break candidate;
            }
        },
    };

    state
        .db
        .insert_url(&slug, &url)
        .map(|entry| (StatusCode::CREATED, Json(entry)))
        .map_err(|_| ApiError::database_error())
}

pub async fn get_urls(
    jar: CookieJar,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<ShortUrl>>> {
    auth::authentication_gate(&jar, &state)?;

    state
        .db
        .list_all_urls()
        .map(Json)
        .map_err(|_| ApiError::database_error())
}

pub async fn delete_url(
    jar: CookieJar,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<StatusCode> {
    auth::authentication_gate(&jar, &state)?;

    match state.db.delete_url_by_slug(&slug) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(ApiError::not_found()),
        Err(_) => Err(ApiError::database_error()),
    }
}
