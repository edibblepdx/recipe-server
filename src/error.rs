use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("recipe fetch failed: {0}")]
    FailedDbFetch(#[from] sqlx::Error),
    #[error("could not find recipe file: {0}")]
    RecipeNotFound(#[from] std::io::Error),
    #[error("could not read csv file: {0}")]
    RecipeMisformat(#[from] csv::Error),
    #[error("invalid database uri: {0}")]
    InvalidDbUri(String),
}

#[derive(Debug, Error)]
pub enum ApiError {}
