use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
};
use kurz::{AppState, Either, api, auth::SessionStore, config::Config, db::Db, ui};
use std::net::SocketAddr;
use tracing::info;

const P404: &str = include_str!("ui/404.html");

async fn redirect(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Either<Redirect, (StatusCode, Html<String>)> {
    match state.db.get_url_by_slug(&slug) {
        Ok(Some(entry)) => {
            let _ = state.db.increment_url_hits(&slug);
            Either::Left(Redirect::temporary(&entry.original_url))
        }
        _ => Either::Right((
            StatusCode::NOT_FOUND,
            Html(
                P404.replace("{slug}", &slug)
                    .replace("{kurz_version}", std::env!("CARGO_PKG_VERSION")),
            ),
        )),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kurz=info,tower_http=info".into()),
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());

    let config = Config::load(&config_path)?;
    info!(
        "Loaded config from '{}' — base_url: {}",
        config_path, config.server.base_url
    );

    let db = Db::open(&config.database.path)?;
    info!("Database opened at '{}'", config.database.path);

    let sessions = SessionStore::new(&config.auth.password);

    let state = AppState {
        config: config.clone(),
        db,
        sessions,
    };

    let app = Router::new()
        .merge(ui::router())
        .merge(api::router())
        .route("/{slug}", get(redirect));

    #[cfg(feature = "livereload")]
    let app = app.layer(tower_livereload::LiveReloadLayer::new());

    let app = app.with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    info!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
