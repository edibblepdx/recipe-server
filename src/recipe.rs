use std::{fs::File, path::Path};

use crate::DatabaseError;

#[derive(Debug)]
pub struct Recipe {
    pub id: String,
    pub cuisine: String,
    //pub ingredients: Vec<String>,
    pub cooking_time_minutes: i64,
    pub prep_time_minutes: i64,
    pub servings: i64,
    pub calories_per_serving: i64,
    //pub dietary_restriction: Vec<String>,
}

#[allow(dead_code)]
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
            id: "Meatballs from Mars".to_string(),
            cuisine: "Martian".to_string(),
            //ingredients: vec![
            //"dried martian gleebles".to_string(),
            //"fish from space".to_string(),
            //],
            cooking_time_minutes: 1_000_000_000,
            prep_time_minutes: 1_000_000_000,
            servings: 99,
            calories_per_serving: 1_000_000_000,
            //dietary_restriction: vec!["humans should not consume".to_string()],
        }
    }
}

impl From<CsvRecipe> for Recipe {
    fn from(v: CsvRecipe) -> Self {
        Self {
            id: v.recipe_name,
            cuisine: v.cuisine,
            //pub ingredients: Vec<String>,
            cooking_time_minutes: v.cooking_time_minutes,
            prep_time_minutes: v.prep_time_minutes,
            servings: v.servings,
            calories_per_serving: v.calories_per_serving,
            //pub dietary_restriction: Vec<String>,
        }
    }
}

pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<CsvRecipe>, DatabaseError> {
    let mut recipes = Vec::new();

    let file = File::open(recipes_path.as_ref())?;
    let mut reader = csv::Reader::from_reader(file);

    for result in reader.deserialize() {
        let rr: Result<CsvRecipe, csv::Error> = result;
        match rr {
            Ok(rr) => recipes.push(rr),
            Err(e) => eprintln!("CSV reader error: {e}"),
        }
    }

    Ok(recipes)
}
