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

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,

    pub pubkeys: Vec<String>,
}

/// Generate a random UUID v4 as a hex string.
fn uuid_v4_hex() -> Uuid {
    uuid::Uuid::new_v4()
}

#[tracing::instrument]
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
    .await?;

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
    .await?;

    Ok(found)
}

#[tracing::instrument]
pub async fn list_repositories(pool: &SqlitePool) -> Result<Vec<Repository>, HovelError> {
    let repos = sqlx::query_as!(
        Repository,
        r#"
        SELECT id as "id: Uuid", name, description, slug
        FROM repository
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(repos)
}

#[tracing::instrument]
pub async fn fetch_repository(pool: &SqlitePool, id: &Uuid) -> Result<Repository, HovelError> {
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
    .await?;

    Ok(found)
}

#[tracing::instrument]
pub async fn create_user(pool: &SqlitePool, name: &str, email: &str) -> Result<User, HovelError> {
    let id = uuid_v4_hex();

    sqlx::query!(
        r#"
        INSERT INTO user (id, name, email)
        VALUES (?, ?, ?)
        "#,
        id,
        name,
        email
    )
    .execute(pool)
    .await?;

    let found = sqlx::query!(
        r#"
        SELECT id as "id: Uuid", name, email
        FROM user
        WHERE id = ?
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(User {
        id: found.id,
        name: found.name,
        email: found.email,
        pubkeys: vec![],
    })
}

pub async fn add_pubkey(pool: &SqlitePool, user_id: &Uuid, key: &str) -> Result<(), HovelError> {

    let key = key.trim();
    // Extract just the key out. Not the method or comment.
    let key = key.split_whitespace().collect::<Vec<&str>>()[1];

    tracing::info!("Adding pubkey: {}", key);

    let id = uuid_v4_hex();
    sqlx::query!(
        r#"
        INSERT INTO pubkey (id, user_id, key)
        VALUES (?, ?, ?)
        "#,
        id,
        user_id,
        key
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument]
pub async fn user_id_from_pubkey(pool: &SqlitePool, pubkey: &str) -> Result<User, HovelError> {
    let user = sqlx::query!(
        r#"
        SELECT user.id as "id: Uuid", email, name
        FROM user
        INNER JOIN pubkey ON user.id = pubkey.user_id
        WHERE pubkey.key = ?
        "#,
        pubkey
    )
    .fetch_one(pool)
    .await?;

    let user_pubkeys = sqlx::query!(
        r#"
        SELECT key
        FROM pubkey
        WHERE user_id = ?
        "#,
        user.id
    )
    .fetch_all(pool)
    .await?;

    Ok(User {
        id: user.id,
        name: user.name,
        email: user.email,
        pubkeys: user_pubkeys.iter().map(|k| k.key.clone()).collect(),
    })
}
