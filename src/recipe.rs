use std::{fs::File, path::Path};

use crate::DatabaseError;

#[derive(Debug, serde::Deserialize)]
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

pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<Recipe>, DatabaseError> {
    let mut recipes = Vec::new();

    let file = File::open(recipes_path.as_ref())?;
    let mut reader = csv::Reader::from_reader(file);

    for result in reader.deserialize() {
        let rr: Recipe = result?;
        recipes.push(rr);
    }

    Ok(recipes)
}
