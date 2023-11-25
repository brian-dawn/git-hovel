use axum::{extract::State, routing::get, Router};
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

    // Insert an example repo.
    crud::create_repository()

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/repositories.json", get(repositories_json))
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

async fn root(pool: State<sqlx::SqlitePool>) -> Result<String, HovelError> {
    let repos = sqlx::query!(r#"SELECT id, name FROM repository"#,)
        .fetch_all(&*pool)
        .await
        .map_err(|_| HovelError::InternalServerError)?;

    let mut html = String::new();
    for repo in repos {
        html.push_str(&format!("<li>{} - {}</li>", repo.id, repo.name));
    }

    Ok(html)
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
