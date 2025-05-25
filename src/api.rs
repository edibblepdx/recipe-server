use crate::*;

use axum::{
    self,
    extract::{Path, State},
    http,
    response::{self, IntoResponse},
};

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "recipe-server", description = "Recipes API")
    )
)]
pub struct ApiDoc;

pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new()
        .routes(routes!(get_recipe_by_id))
        .routes(routes!(get_recipe_by_cuisine))
        .routes(routes!(get_recipe_by_random))
}

/// helper function that grabs the recipe from the database by id
async fn fetch(db: &SqlitePool, recipe_id: i64) -> Result<response::Response, http::StatusCode> {
    Recipe::get(db, recipe_id)
        .await
        .map(|recipe| recipe.into_response())
        .map_err(|e| {
            log::warn!("recipe fetch failed: {e}");
            http::StatusCode::NOT_FOUND
        })
}

#[utoipa::path(
    get,
    path = "/api/recipe/id/{recipe_id}",
    responses(
        (status = 200, description = "Get a recipe by id", body = [Recipe]),
        (status = 404, description = "No matching recipe"),
    )
)]
async fn get_recipe_by_id(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(recipe_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    match recipe_id.parse::<i64>() {
        Ok(id) => {
            let app_reader = app_state.read().await;
            let db = &app_reader.db;
            fetch(db, id).await
        }
        Err(e) => {
            log::warn!("malformed id: {e}");
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/recipe/cuisine/{cuisine}",
    responses(
        (status = 200, description = "Get a recipe by cuisine", body = [Recipe]),
        (status = 404, description = "No matching recipe"),
    )
)]
async fn get_recipe_by_cuisine(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(cuisine): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    match sqlx::query_scalar!(
        "SELECT id FROM recipes WHERE cuisine = $1 COLLATE NOCASE ORDER BY RANDOM() LIMIT 1;",
        cuisine
    )
    .fetch_one(db)
    .await
    {
        Ok(id) => fetch(db, id).await,
        Err(e) => {
            log::error!("no recipe found with cuisine: {e}");
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/recipe/random",
    responses(
        (status = 200, description = "Get a recipe by random", body = [Recipe]),
        (status = 404, description = "No recipe"),
    )
)]
async fn get_recipe_by_random(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    match sqlx::query_scalar!("SELECT id FROM recipes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await
    {
        Ok(id) => fetch(db, id).await,
        Err(e) => {
            log::error!("random recipe selection failed: {e}");
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}
