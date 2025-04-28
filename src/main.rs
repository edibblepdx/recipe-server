mod error;
mod recipe;
mod templates;

extern crate log;
extern crate mime;

use error::*;
use recipe::*;
use templates::*;

use axum::{self, Router, extract::State, response, routing};
use clap::Parser;
use sqlx::{SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::Arc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
}

struct AppState {
    db: SqlitePool,
    current_recipe: Recipe,
}

async fn get_recipe(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let mut app_state = app_state.write().await;
    let db = &app_state.db;
    let recipe_result = sqlx::query_as!(Recipe, "SELECT * FROM recipes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await;
    match recipe_result {
        Ok(recipe) => app_state.current_recipe = recipe,
        Err(e) => log::warn!("recipe fetch failed: {}", e),
    }
    let recipe = IndexTemplate::recipe(&app_state.current_recipe);
    response::Html(recipe.to_string())
}

fn get_db_uri(db_uri: Option<&str>) -> String {
    if let Some(db_uri) = db_uri {
        db_uri.to_string()
    } else if let Ok(db_uri) = std::env::var("RECIPE_DB_URI") {
        db_uri
    } else {
        "sqlite://db/recipe.db".to_string()
    }
}

fn extract_db_dir(db_uri: &str) -> Result<&str, DatabaseError> {
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

async fn init_db_from_csv(
    db: &SqlitePool,
    path: &std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let recipes = read_recipes(path)?;
    'next_recipe: for r in recipes {
        let mut rtx = db.begin().await?;
        let recipe_insert = sqlx::query!(
            "INSERT INTO jokes (id, recipe_name, cuisine, \
            ingredients, cooking_time_minutes, prep_time_minutes, \
            servings, calories_per_serving, dietary_restrictions) \
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);",
            r.id,
            r.recipe_name,
            r.cuisine,
            r.ingredients,
            r.cooking_time_minutes,
            r.prep_time_minutes,
            r.servings,
            r.calories_per_serving,
            r.dietary_restrictions,
        )
        .execute(&mut *rtx)
        .await;
        if let Err(e) = recipe_insert {
            eprintln!("error: joke insert: {}: {}", r.id, e);
            rtx.rollback().await?;
            continue;
        };
        // TODO: make a cuisine table
        for t in ts {
            let tag_insert =
                sqlx::query!("INSERT INTO tags (joke_id, tag) VALUES ($1, $2);", r.id, t,)
                    .execute(&mut *rtx)
                    .await;
            if let Err(e) = tag_insert {
                eprintln!("error: tag insert: {} {}: {}", r.id, t, e);
                rtx.rollback().await?;
                continue 'next_recipe;
            };
        }
        rtx.commit().await?;
    }
    Ok(())
}

async fn serve(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let db_uri = get_db_uri(args.db_uri.as_deref());
    if !sqlite::Sqlite::database_exists(&db_uri).await? {
        let db_dir = extract_db_dir(&db_uri)?;
        std::fs::create_dir_all(db_dir)?;
        sqlite::Sqlite::create_database(&db_uri).await?
    }

    let db = SqlitePool::connect(&db_uri).await?;
    sqlx::migrate!().run(&db).await?;

    if let Some(path) = args.init_from {
        init_db_from_csv(&db, &path).await?;
    }

    let current_recipe = Recipe::default();

    let app_state = AppState { db, current_recipe };
    let state = Arc::new(RwLock::new(app_state));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "recipe-server=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    /*
    https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    */
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();
    let app = Router::new()
        .route("/", routing::get(get_recipe))
        .layer(trace_layer)
        .with_state(state);

    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(err) = serve(args).await {
        eprintln!("server error: {}", err);
        std::process::exit(1);
    }
}
