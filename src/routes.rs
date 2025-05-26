use crate::AppState;
use crate::Recipe;
use crate::api;
use crate::templates::*;

use axum::Router;
use axum::{
    self,
    extract::{Query, State},
    http,
    response::{self, IntoResponse},
    routing,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use tower_http::services;
use tower_http::trace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use std::sync::Arc;

#[derive(Deserialize)]
struct GetRecipeParams {
    id: Option<String>,
    cuisine: Option<String>,
}

pub fn init_router() -> Router<Arc<RwLock<AppState>>> {
    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();

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

    // Cors layer
    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([http::Method::GET])
        .allow_origin(tower_http::cors::Any);

    // 404 handler
    async fn handler_404() -> axum::response::Response {
        (http::StatusCode::NOT_FOUND, "404 Not Found").into_response()
    }
    // API router
    let (api_router, api) = OpenApiRouter::with_openapi(api::ApiDoc::openapi())
        .nest("/api/v1", api::router())
        .split_for_parts();

    // API Docs
    let swagger_ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone());
    let redoc_ui = Redoc::with_url("/redoc", api);
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    // init router
    Router::new()
        .route("/", routing::get(root))
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime("assets/static/recipe.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .merge(api_router)
        .fallback(handler_404)
        .layer(cors)
        .layer(trace_layer)
}

async fn root(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetRecipeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_state = app_state.write().await;
    let db = &app_state.db;

    if let GetRecipeParams { id: Some(id), .. } = params {
        app_state.current_recipe = match id.parse::<i64>() {
            Ok(id) => Recipe::get_by_id(db, id).await.unwrap_or_default(),
            Err(e) => {
                log::warn!("malformed id: {e}");
                Recipe::default()
            }
        };
        let recipe = IndexTemplate::new(&app_state.current_recipe);
        Ok(response::Html(recipe.to_string()).into_response())
    } else if let GetRecipeParams {
        cuisine: Some(cuisine),
        ..
    } = params
    {
        if cuisine.trim().is_empty() {
            // redirect
            return Ok(response::Redirect::to("/").into_response());
        }

        app_state.current_recipe = Recipe::get_random_cuisine(db, &cuisine)
            .await
            .unwrap_or_default();
        let recipe = IndexTemplate::new(&app_state.current_recipe);
        Ok(response::Html(recipe.to_string()).into_response())
    } else {
        /*
        Ok(id) => {
            // redirect
            let uri = format!("/?id={}", id);
            Ok(response::Redirect::to(&uri).into_response())
        }
        */
        app_state.current_recipe = Recipe::get_random(db).await.unwrap_or_default();
        let recipe = IndexTemplate::new(&app_state.current_recipe);
        Ok(response::Html(recipe.to_string()).into_response())
    }
}

//fn recipe_by_random() {}
//fn recipe_by_cuisuine() {}
//fn recipe_by_id() {}
