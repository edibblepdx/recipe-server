use crate::AppState;

mod api;
mod web;

use axum::Router;
use axum::{self, http, response::IntoResponse};
use tokio::sync::RwLock;
use tower_http::trace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::Arc;

pub fn init_router() -> Router<Arc<RwLock<AppState>>> {
    // tracing layer
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "recipe-server=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    /*
    https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    */
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    // CORS layer
    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([http::Method::GET])
        .allow_origin(tower_http::cors::Any);

    // 404 handler
    async fn handler_404() -> axum::response::Response {
        (http::StatusCode::NOT_FOUND, "404 Not Found").into_response()
    }

    // Init router
    Router::new()
        .merge(api::router())
        .merge(web::router())
        .fallback(handler_404)
        .layer(cors)
        .layer(trace_layer)
}
