#![feature(bool_to_result)]

use axum::{Json, body::Bytes, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::{auth::SessionStore, config::Config, db::Db};

pub mod api;
pub mod auth;
pub mod codegen;
pub mod config;
pub mod db;
pub mod ui;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Db,
    pub sessions: SessionStore,
}

pub type ApiResult<T> = Result<T, ApiError>;

pub struct ApiError {
    pub error: String,
    pub code: StatusCode,
}

impl ApiError {
    pub fn new<S>(error: S, code: StatusCode) -> Self
    where
        S: ToString,
    {
        Self {
            error: error.to_string(),
            code,
        }
    }

    pub fn not_found() -> Self {
        Self::new("not found", StatusCode::NOT_FOUND)
    }

    pub fn internal_server_error<S>(error: S) -> Self
    where
        S: ToString,
    {
        Self::new(error, StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn database_error() -> Self {
        Self::internal_server_error("database error")
    }

    pub fn unauthorized() -> Self {
        Self::new("unauthorized", StatusCode::UNAUTHORIZED)
    }

    pub fn bad_request<S>(error: S) -> Self
    where
        S: ToString,
    {
        Self::new(error, StatusCode::BAD_REQUEST)
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.code, Json(ErrorResponse { error: self.error })).into_response()
    }
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> IntoResponse for Either<L, R>
where
    L: IntoResponse,
    R: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Either::Left(l) => l.into_response(),
            Either::Right(r) => r.into_response(),
        }
    }
}

pub struct Favicon<T>(pub T);

impl<T> IntoResponse for Favicon<T>
where
    T: Into<Bytes>,
{
    fn into_response(self) -> axum::response::Response {
        (
            [(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("image/svg+xml"),
            )],
            self.0.into(),
        )
            .into_response()
    }
}
