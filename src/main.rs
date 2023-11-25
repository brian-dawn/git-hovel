use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, Router,
};
use dotenv::dotenv;
use errors::HovelError;
use maud::{html, Markup};
use serde::{Deserialize, Serialize};
use sqlx;
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing_subscriber::util::SubscriberInitExt;

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
    sqlx::query!(r#"INSERT INTO repository(name) VALUES (?1)"#, "test")
        .execute(&pool)
        .await?;

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
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
