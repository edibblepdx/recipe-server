use axum::{self, http, response::IntoResponse};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::{fs::File, path::Path};
use utoipa::ToSchema;

use crate::DatabaseError;

/// Common Data View
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    pub cuisine: String,
    pub ingredients: Vec<String>,
    pub cooking_time_minutes: i64,
    pub prep_time_minutes: i64,
    pub servings: i64,
    pub calories_per_serving: i64,
    pub dietary_restrictions: Vec<String>,
}

/// Intermediate Data View as read from CSV file
#[derive(Debug, Deserialize)]
pub struct CsvRecipe {
    pub recipe_name: String,
    pub cuisine: String,
    pub ingredients: String,
    pub cooking_time_minutes: i64,
    pub prep_time_minutes: i64,
    pub servings: i64,
    pub calories_per_serving: i64,
    pub dietary_restrictions: String,
}

/// Recipe db interface
impl Recipe {
    pub async fn get_by_id(db: &SqlitePool, id: i64) -> Result<Self, DatabaseError> {
        match sqlx::query!("SELECT * FROM recipes WHERE id = $1;", id)
            .fetch_one(db)
            .await
        {
            Ok(recipe) => {
                let mut recipe = Recipe {
                    id: recipe.id,
                    name: recipe.name,
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

                Ok(recipe)
            }
            Err(e) => Err(DatabaseError::FailedDbFetch(e)),
        }
    }

    pub async fn get_random(db: &SqlitePool) -> Result<Self, DatabaseError> {
        match sqlx::query_scalar!("SELECT id FROM recipes ORDER BY RANDOM() LIMIT 1;")
            .fetch_one(db)
            .await
        {
            Ok(id) => Recipe::get_by_id(db, id).await,
            Err(e) => Err(DatabaseError::FailedDbFetch(e)),
        }
    }

    pub async fn get_random_cuisine(db: &SqlitePool, cuisine: &str) -> Result<Self, DatabaseError> {
        match sqlx::query_scalar!(
            "SELECT id FROM recipes WHERE cuisine = $1 COLLATE NOCASE ORDER BY RANDOM() LIMIT 1;",
            cuisine
        )
        .fetch_one(db)
        .await
        {
            Ok(id) => Recipe::get_by_id(db, id).await,
            Err(e) => Err(DatabaseError::FailedDbFetch(e)),
        }
    }
}

/// Default Recipe Stub
impl Default for Recipe {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Not Found".to_string(),
            cuisine: "None".to_string(),
            ingredients: vec!["None".to_string()],
            cooking_time_minutes: 0,
            prep_time_minutes: 0,
            servings: 0,
            calories_per_serving: 0,
            dietary_restrictions: vec!["None".to_string()],
        }
    }
}

impl IntoResponse for &Recipe {
    fn into_response(self) -> axum::response::Response {
        (http::StatusCode::OK, axum::Json(&self)).into_response()
    }
}

impl From<CsvRecipe> for Recipe {
    fn from(v: CsvRecipe) -> Self {
        let re = Regex::new(r"[\[\]']").unwrap();

        let ingredients_vec: Vec<String> = re
            .replace_all(&v.ingredients, "")
            .split(",")
            .map(|s| s.trim().to_string())
            .collect();
        let dietary_restrictions_vec: Vec<String> = re
            .replace_all(&v.dietary_restrictions, "")
            .split(",")
            .map(|s| s.trim().to_string())
            .collect();

        Self {
            id: 0,
            name: v.recipe_name,
            cuisine: v.cuisine,
            ingredients: ingredients_vec,
            cooking_time_minutes: v.cooking_time_minutes,
            prep_time_minutes: v.prep_time_minutes,
            servings: v.servings,
            calories_per_serving: v.calories_per_serving,
            dietary_restrictions: dietary_restrictions_vec,
        }
    }
}

/// Read recipes as CsvRecipe IR then immediately convert to Recipe format
pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<Recipe>, DatabaseError> {
    let mut recipes: Vec<Recipe> = Vec::new();

    let file = File::open(recipes_path.as_ref())?;
    let mut reader = csv::Reader::from_reader(file);

    for result in reader.deserialize() {
        let rr: Result<CsvRecipe, csv::Error> = result;
        match rr {
            Ok(rr) => recipes.push(rr.into()),
            Err(e) => eprintln!("CSV reader error: {e}"),
        }
    }

    Ok(recipes)
}
