use std::{collections::BTreeMap, fmt, sync::Arc, time::Duration};

use axum::{
    extract::{Host, Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    routing::{self, IntoMakeService},
    Router,
};
use reqwest::{Client, ClientBuilder, StatusCode};
use serde::Deserialize;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::Level;

use crate::{
    text::StrIterator,
    web::{env::Env, lines::Lines, worker::Worker},
};

#[derive(Clone)]
pub struct AppState {
    client: Client,
    worker: Worker,
    env: Arc<Env>,
}

pub fn app(env: Env) -> IntoMakeService<Router> {
    let state = AppState {
        client: ClientBuilder::new()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap(),
        worker: Worker::new(env.clone()),
        env: Arc::new(env),
    };
    Router::new()
        .route("/:owner/:repo/blob/:branch/*path", routing::get(github))
        .with_state(state)
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::INFO)))
        .into_make_service()
}

#[derive(Deserialize)]
struct GithubQuery {
    font: Option<String>,
    theme: Option<String>,
    tab_width: Option<usize>,
    #[serde(default, flatten)]
    rest: BTreeMap<String, String>,
}

async fn github(
    State(state): State<AppState>,
    Host(host): Host,
    Path((owner, repo, branch, path)): Path<(String, String, String, String)>,
    Query(query): Query<GithubQuery>,
) -> Response {
    let host: Vec<&str> = host.rsplitn(3, '.').collect();
    let subdomain = match host.as_slice() {
        [_, _, subdomain] => *subdomain,
        [tld, domain] if tld.starts_with("localhost") => *domain,
        [..] => "",
    };

    let lines = query.rest.keys().find_map(|s| s.parse::<Lines>().ok());

    if subdomain == state.env.image_subdomain {
        image(
            state,
            ImageQuery {
                owner,
                repo,
                branch,
                path,
                lines,
                font: query.font,
                theme: query.theme,
                tab_width: query.tab_width,
            },
        )
        .await
    } else {
        todo!("html with og:image and meta refresh")
    }
}

fn status(s: StatusCode) -> Response {
    (
        s,
        [(header::CONTENT_TYPE, "text/plain")],
        s.canonical_reason().unwrap_or("error"),
    )
        .into_response()
}
fn error(e: impl fmt::Display, s: StatusCode) -> Response {
    tracing::warn!("{e}");
    status(s)
}

#[derive(Debug)]
struct ImageQuery {
    owner: String,
    repo: String,
    branch: String,
    path: String,
    lines: Option<Lines>,
    font: Option<String>,
    theme: Option<String>,
    tab_width: Option<usize>,
}

#[tracing::instrument(level = "debug", skip(state))]
async fn image(state: AppState, query: ImageQuery) -> Response {
    let extension = query
        .path
        .rsplit_once('.')
        .or_else(|| query.path.rsplit_once('/'))
        .unwrap_or(("", &query.path))
        .1
        .to_string();

    let url = format!(
        "https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}",
        owner = query.owner,
        repo = query.repo,
        branch = query.branch,
        path = query.path,
    );
    let res = match state.client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => return error(e, StatusCode::BAD_REQUEST),
    };
    let mut text = match res.text().await {
        Ok(t) => t,
        Err(e) => return error(e, StatusCode::BAD_REQUEST),
    };

    if let Some(lines) = query.lines {
        text = text
            .lines()
            .skip(lines.start - 1)
            .take(lines.end - lines.start + 1)
            .join_newline();
    }

    let image = state
        .worker
        .render(text, extension, query.font, query.theme, query.tab_width)
        .await;
    match image {
        Some(image) => {
            (StatusCode::OK, [(header::CONTENT_TYPE, "image/png")], image).into_response()
        }
        None => status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
