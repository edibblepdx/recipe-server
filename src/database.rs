use crate::DatabaseError;
use crate::read_recipes;

use sqlx::SqlitePool;

pub fn get_db_uri(db_uri: Option<&str>) -> String {
    if let Some(db_uri) = db_uri {
        db_uri.to_string()
    } else if let Ok(db_uri) = std::env::var("RECIPE_DB_URI") {
        db_uri
    } else {
        "sqlite://db/recipe.db".to_string()
    }
}

pub fn extract_db_dir(db_uri: &str) -> Result<&str, DatabaseError> {
    if db_uri.starts_with("sqlite://") && db_uri.ends_with(".db") {
        let start = db_uri.find(':').unwrap() + 3;
        let mut path = &db_uri[start..];
        if let Some(end) = path.rfind('/') {
            path = &path[..end];
        } else {
            path = "";
        }
        Ok(path)
    } else {
        Err(DatabaseError::InvalidDbUri(db_uri.to_string()))
    }
}

pub async fn init_db_from_csv(
    db: &SqlitePool,
    path: &std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let recipes = read_recipes(path)?;
    'next_recipe: for mut rr in recipes {
        let mut rtx = db.begin().await?;
        match sqlx::query_scalar!(
            "INSERT INTO recipes (name, cuisine, cooking_time_minutes, prep_time_minutes, servings, calories_per_serving) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id;",
            rr.name,
            rr.cuisine,
            rr.cooking_time_minutes,
            rr.prep_time_minutes,
            rr.servings,
            rr.calories_per_serving,
        )
        .fetch_one(&mut *rtx)
        .await
        {
            Ok(id) => rr.id = id,
            Err(e) => {
                eprintln!("error: joke insert: {}: {}", rr.id, e);
                rtx.rollback().await?;
                continue;
            }
        }

        for ii in rr.ingredients {
            if let Err(e) = sqlx::query!(
                "INSERT INTO ingredients (recipe_id, ingredient) VALUES ($1, $2);",
                rr.id,
                ii,
            )
            .execute(&mut *rtx)
            .await
            {
                eprintln!("error: ingredient insert: recipe {}; {}: {}", rr.id, ii, e);
                rtx.rollback().await?;
                continue 'next_recipe;
            };
        }

        for dd in rr.dietary_restrictions {
            if let Err(e) = sqlx::query!(
                "INSERT INTO dietary_restrictions (recipe_id, dietary_restriction) VALUES ($1, $2);",
                rr.id,
                dd,
            )
                .execute(&mut *rtx)
                .await
            {
                eprintln!(
                    "error: dietary_restriction insert: recipe {}; {}: {}",
                    rr.id, dd, e
                );
                rtx.rollback().await?;
                continue 'next_recipe;
            };
        }
        rtx.commit().await?;
    }
    Ok(())
}
