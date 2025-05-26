use crate::*;

use axum::{
    self,
    extract::{Path, State},
    http,
    response::{self, IntoResponse},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "recipe-server", description = "Recipes API")
    )
)]
pub struct ApiDoc;

pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new()
        .routes(routes!(recipe_get_by_id))
        .routes(routes!(recipe_get_random))
        .routes(routes!(recipe_get_random_cuisine))
}

#[utoipa::path(
    get,
    path = "/recipe/id/{recipe_id}",
    responses(
        (status = 200, description = "Get a recipe by id", body = [Recipe]),
        (status = 404, description = "No matching recipe"),
    )
)]
async fn recipe_get_by_id(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(recipe_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    match recipe_id.parse::<i64>() {
        Ok(id) => {
            let app_reader = app_state.read().await;
            let db = &app_reader.db;
            Recipe::get_by_id(db, id)
                .await
                .map(|recipe| recipe.into_response())
                .map_err(|e| {
                    log::warn!("recipe fetch failed: {e}");
                    http::StatusCode::NOT_FOUND
                })
        }
        Err(e) => {
            log::warn!("malformed id: {e}");
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    get,
    path = "/recipe/random",
    responses(
        (status = 200, description = "Get a recipe by random", body = [Recipe]),
        (status = 404, description = "No recipe"),
    )
)]
async fn recipe_get_random(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    Recipe::get_random(db)
        .await
        .map(|recipe| recipe.into_response())
        .map_err(|e| {
            log::warn!("recipe fetch failed: {e}");
            http::StatusCode::NOT_FOUND
        })
}

#[utoipa::path(
    get,
    path = "/recipe/cuisine/{cuisine}",
    responses(
        (status = 200, description = "Get a recipe by cuisine", body = [Recipe]),
        (status = 404, description = "No matching recipe"),
    )
)]
async fn recipe_get_random_cuisine(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(cuisine): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    Recipe::get_random_cuisine(db, &cuisine)
        .await
        .map(|recipe| recipe.into_response())
        .map_err(|e| {
            log::warn!("recipe fetch failed: {e}");
            http::StatusCode::NOT_FOUND
        })
}
