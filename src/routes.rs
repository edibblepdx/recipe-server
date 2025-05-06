use crate::AppState;
use crate::Recipe;
use crate::templates::*;

use axum::{
    Router,
    extract::{Query, State},
    http,
    response::{self, IntoResponse},
    routing,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services;

#[derive(Deserialize)]
struct GetRecipeParams {
    id: Option<String>,
}

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

async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetRecipeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_state = app_state.write().await;
    let db = &app_state.db;

    if let GetRecipeParams { id: Some(id), .. } = params {
        let result = match sqlx::query!("SELECT * FROM recipes WHERE id = $1;", id)
            .fetch_one(db)
            .await
        {
            Ok(recipe) => {
                let mut recipe = Recipe {
                    id: recipe.id,
                    cuisine: recipe.cuisine,
                    ingredients: vec![],
                    cooking_time_minutes: recipe.cooking_time_minutes,
                    prep_time_minutes: recipe.prep_time_minutes,
                    servings: recipe.servings,
                    calories_per_serving: recipe.calories_per_serving,
                    dietary_restrictions: vec![],
                };

                recipe.ingredients = match sqlx::query_scalar!(
                    "SELECT ingredient FROM ingredients WHERE recipe_id = $1;",
                    recipe.id
                )
                .fetch_all(db)
                .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("ingredient fetch failed: {}", e);
                        vec![]
                    }
                };

                recipe.dietary_restrictions = match sqlx::query_scalar!(
                    "SELECT dietary_restriction FROM dietary_restrictions WHERE recipe_id = $1;",
                    recipe.id
                )
                .fetch_all(db)
                .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("dietary restriction fetch failed: {}", e);
                        vec![]
                    }
                };

                app_state.current_recipe = recipe;
                let recipe = IndexTemplate::new(&app_state.current_recipe);
                Ok(response::Html(recipe.to_string()).into_response())
            }
            Err(e) => {
                log::warn!("recipe fetch failed: {}", e);
                Err(http::StatusCode::NOT_FOUND)
            }
        };
        return result;
    } else {
        // Random Recipe
        match sqlx::query_scalar!("SELECT id FROM recipes ORDER BY RANDOM() LIMIT 1;")
            .fetch_one(db)
            .await
        {
            Ok(id) => {
                // redirect
                let uri = format!("/?id={}", id);
                Ok(response::Redirect::to(&uri).into_response())
            }
            Err(e) => {
                log::error!("recipe selection failed: {e}");
                panic!("recipe selection failed")
            }
        }
    }
}
