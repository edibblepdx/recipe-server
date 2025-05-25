extern crate fastrand;
extern crate log;
extern crate mime;

mod api;
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
use tower_http::trace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

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

    let current_recipe = Recipe::default();

    let app_state = AppState { db, current_recipe };
    let state = Arc::new(RwLock::new(app_state));
    let app = init_router().layer(trace_layer).with_state(state);

    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    eprintln!("Listening on 127.0.0.1:3000");

    axum::serve(listener, app).await?;
    Ok(())
}
