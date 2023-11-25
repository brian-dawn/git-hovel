use std::fmt::DebugStruct;

use crate::errors::HovelError;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, types::Uuid, FromRow};

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
}

/// Generate a random UUID v4 as a hex string.
fn uuid_v4_hex() -> String {
    let uuid = uuid::Uuid::new_v4();
    let hex = uuid.to_string();
    hex
}

pub async fn create_repository(
    pool: &SqlitePool,
    name: &str,
    description: Option<&str>,
    slug: &str,
) -> Result<Repository, HovelError> {
    let id = uuid_v4_hex();

    sqlx::query!(
        r#"
        INSERT INTO repository (id, name, description, slug)
        VALUES (?, ?, ?, ?)
        "#,
        id,
        name,
        description,
        slug
    )
    .execute(pool)
    .await
    .map_err(|_| HovelError::InternalServerError)?;

    let found = sqlx::query_as!(
        Repository,
        r#"
        SELECT id as "id: Uuid", name, description, slug
        FROM repository
        WHERE id = ?
        "#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|_| HovelError::InternalServerError)?;

    Ok(found)
}


pub async fn list_repositories(pool: &SqlitePool) -> Result<Vec<Repository>, HovelError> {
    let repos = sqlx::query_as!(
        Repository,
        r#"
        SELECT id as "id: Uuid", name, description, slug
        FROM repository
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|_| HovelError::InternalServerError)?;

    Ok(repos)
}
