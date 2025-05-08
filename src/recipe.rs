use regex::Regex;
use std::{fs::File, path::Path};

use crate::DatabaseError;

#[derive(Debug)]
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

#[derive(Debug, serde::Deserialize)]
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

impl Default for Recipe {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Meatballs from Mars".to_string(),
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
