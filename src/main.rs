mod recipe;
mod templates;

use axum::{Router, routing::get};
use clap::Parser;
use tokio;

#[derive(Parser)]
struct Args {}

async fn serve(args: Args) -> Result<(), Box<dyn std::error::Error>> {
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
