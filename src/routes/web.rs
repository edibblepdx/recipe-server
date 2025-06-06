use crate::AppState;
use crate::Recipe;
use crate::templates::*;

use axum::{
    self, Router,
    extract::{Query, State},
    response::{self, IntoResponse},
    routing,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use tower_http::services;

use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetRecipeParams {
    id: Option<String>,
    cuisine: Option<String>,
}

pub fn router() -> Router<Arc<RwLock<AppState>>> {
    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();

    Router::new()
        .route("/", routing::get(root))
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime(
                "assets/static/recipe.css", //
                &mime::TEXT_CSS_UTF_8,      //
            ),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime(
                "assets/static/favicon.ico", //
                &mime_favicon,               //
            ),
        )
}

async fn root(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetRecipeParams>,
) -> response::Response {
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
        response::Html(recipe.to_string()).into_response()
    } else if let GetRecipeParams {
        cuisine: Some(cuisine),
        ..
    } = params
    {
        if cuisine.trim().is_empty() {
            return response::Redirect::to("/").into_response();
        }

        let recipe = Recipe::get_random_cuisine(db, &cuisine)
            .await
            .unwrap_or_default();
        let uri = format!("/?id={}&cuisine={}", recipe.id, recipe.cuisine);
        response::Redirect::to(&uri).into_response()
    } else {
        let recipe = Recipe::get_random(db).await.unwrap_or_default();
        let uri = format!("/?id={}", recipe.id);
        response::Redirect::to(&uri).into_response()
    }
}
