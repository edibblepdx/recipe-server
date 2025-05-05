use crate::AppState;
use crate::Recipe;
use crate::templates::*;

use axum::{Router, extract::State, response, routing};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services;

pub fn init_router() -> Router<Arc<RwLock<AppState>>> {
    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();

    Router::new()
        .route("/", routing::get(get_recipe))
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime("assets/static/recipe.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
}

async fn get_recipe(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let mut app_state = app_state.write().await;
    let db = &app_state.db;

    match sqlx::query_as!(Recipe, "SELECT * FROM recipes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await
    {
        Ok(recipe) => app_state.current_recipe = recipe,
        Err(e) => log::warn!("recipe fetch failed: {}", e),
    }
    let recipe = IndexTemplate::recipe(&app_state.current_recipe);
    response::Html(recipe.to_string())
}
