extern crate fastrand;
extern crate log;
extern crate mime;

mod database;
mod error;
mod recipe;
mod routes;
mod templates;

use database::*;
use error::*;
use recipe::*;
use routes::*;

use clap::Parser;
use sqlx::{SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};

use std::sync::Arc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
    #[arg(long, name = "host", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, name = "port", default_value_t = 3000)]
    port: u16,
}

struct AppState {
    db: SqlitePool,
    current_recipe: Recipe,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(err) = serve(args).await {
        eprintln!("server error: {}", err);
        std::process::exit(1);
    }
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
    let app = init_router().with_state(state);

    let listener = net::TcpListener::bind(format!("{}:{}", args.host, args.port)).await?;
    eprintln!("Listening on {}:{}", args.host, args.port);

    axum::serve(listener, app).await?;
    Ok(())
}
