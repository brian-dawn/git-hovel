use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use dotenv::dotenv;

use errors::HovelError;
use serde::{Deserialize, Serialize};

use std::{error::Error, net::SocketAddr};

pub mod crud;
pub mod errors;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());

    let pool = sqlx::sqlite::SqlitePool::connect(&database_url).await?;

    // Embed the migrations directory into the binary.
    sqlx::migrate!().run(&pool).await?;

    crate::crud::create_repository(&pool, "Test", None, "some-test-url").await?;

    // Insert an example repo.

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/api/repositories", get(repositories_json))
        .route("/api/repositories/:id", get(repositories_json_fetch))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct RepositoryResponse {
    repositories: Vec<crud::Repository>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    repositories: Vec<crud::Repository>,
}

async fn root(pool: State<sqlx::SqlitePool>) -> Result<Html<String>, HovelError> {
    // Load the askama template...
    let index_template = IndexTemplate {
        repositories: crud::list_repositories(&pool).await?,
    };

    let rendered = index_template.render()?;

    Ok(Html(rendered))
}

async fn repositories_json(
    pool: State<sqlx::SqlitePool>,
) -> Result<axum::response::Json<RepositoryResponse>, HovelError> {
    let repos = crud::list_repositories(&pool).await?;

    let response = RepositoryResponse {
        repositories: repos,
    };

    // Return json response
    Ok(axum::response::Json(response))
}

async fn repositories_json_fetch(
    pool: State<sqlx::SqlitePool>,
    Path(id): Path<sqlx::types::Uuid>,
) -> Result<axum::response::Json<crud::Repository>, HovelError> {
    let repository = crud::fetch_repository(&pool, &id).await?;

    // Return json response
    Ok(axum::response::Json(repository))
}
