use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;

use crate::DatabaseError;

use serde::Deserialize;

pub struct Recipe {
    pub id: u64,
    pub recipe_name: String,
    pub cuisine: String,
    pub ingredients: Vec<String>,
    pub cooking_time_minutes: u16,
    pub prep_time_minutes: u16,
    pub servings: u16,
    pub calories_per_serving: u16,
    pub dietary_restrictions: Vec<String>,
}

// recipe_name,cuisine,ingredients,cooking_time_minutes,prep_time_minutes,servings,calories_per_serving,dietary_restrictions

impl Default for Recipe {
    fn default() -> Self {
        Self {
            id: 0,
            recipe_name: "Meatballs from Mars".to_string(),
            cuisine: "Martian".to_string(),
            ingredients: vec![
                "dried martian gleebles".to_string(),
                "fish from space".to_string(),
            ],
            cooking_time_minutes: 1_000_000_000,
            prep_time_minutes: 1_000_000_000,
            servings: 99,
            calories_per_serving: 1_000_000_000,
            dietary_restrictions: vec!["humans should not consume".to_string()],
        }
    }
}

pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<Recipe>, DatabaseError> {
    let recipes = vec![];
    Ok(recipes)
}
