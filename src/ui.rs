use axum::{
    Form, Router,
    extract::State,
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
};
use axum_extra::{extract::CookieJar, response::Css};
use serde::Deserialize;

use crate::{
    ApiResult, AppState, Either, Favicon,
    auth::{self, SESSION_COOKIE},
};

const LOGIN: &str = include_str!("ui/login.html");
const INDEX: &str = include_str!("ui/index.html");
const STYLE: &str = include_str!("ui/style.css");
const FAVICON: &str = include_str!("ui/favicon.svg");

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_index))
        .route("/login", get(get_login).post(post_login))
        .route("/logout", get(logout))
        .route("/style.css", get(get_css))
        .route("/favicon.ico", get(get_favicon))
}

pub async fn get_css(State(state): State<AppState>) -> Css<String> {
    Css(STYLE.replace("{accent_color}", &state.config.ui.accent_color))
}

pub async fn get_favicon(State(state): State<AppState>) -> Favicon<String> {
    Favicon(FAVICON.replace("{accent_color}", &state.config.ui.accent_color))
}

pub async fn get_index(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Either<Redirect, Html<String>> {
    if auth::authentication_gate(&jar, &state).is_err() {
        Either::Left(Redirect::to("/login"))
    } else {
        Either::Right(Html(
            INDEX.replace("{base_url}", &state.config.server.base_url),
        ))
    }
}

pub async fn get_login(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Either<Redirect, Html<String>> {
    if auth::authentication_gate(&jar, &state).is_ok() {
        Either::Left(Redirect::to("/"))
    } else {
        Either::Right(Html(
            LOGIN
                .replace("{login_error}", "")
                .replace("{kurz_version}", std::env!("CARGO_PKG_VERSION")),
        ))
    }
}

#[derive(Deserialize)]
pub struct LoginForm {
    password: String,
}

pub async fn post_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Either<(CookieJar, Redirect), (StatusCode, Html<String>)> {
    match state.sessions.login(&form.password) {
        Some(token) => {
            let cookie = axum_extra::extract::cookie::Cookie::build((SESSION_COOKIE, token))
                .path("/")
                .http_only(true)
                .build();

            Either::Left((jar.add(cookie), Redirect::to("/")))
        }
        None => Either::Right((
            StatusCode::UNAUTHORIZED,
            Html(
                LOGIN
                    .replace("{login_error}", "incorrect password")
                    .replace("{kurz_version}", std::env!("CARGO_PKG_VERSION")),
            ),
        )),
    }
}

pub async fn logout(
    jar: CookieJar,
    State(state): State<AppState>,
) -> ApiResult<Either<(CookieJar, Redirect), Redirect>> {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        state.sessions.logout(cookie.value())?;
        let removal = axum_extra::extract::cookie::Cookie::build((SESSION_COOKIE, ""))
            .path("/")
            .max_age(time::Duration::seconds(0))
            .build();

        Ok(Either::Left((jar.add(removal), Redirect::to("/login"))))
    } else {
        Ok(Either::Right(Redirect::to("/login")))
    }
}
