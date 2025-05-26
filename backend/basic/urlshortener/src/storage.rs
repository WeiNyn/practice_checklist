use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::Sqlite;
use sqlx::migrate::MigrateDatabase;
use sqlx::{FromRow, SqlitePool};

pub const DB_URL: &str = "sqlite://url.db";

pub async fn init_db(db_url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    if !Sqlite::database_exists(db_url).await? {
        Sqlite::create_database(db_url).await?;
    }
    let pool = sqlx::SqlitePool::connect(db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Url {
    pub id: Option<i64>,
    pub original_url: String,
    pub short_url: String,
    pub click_count: i64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

pub async fn create_url(pool: &SqlitePool, original_url: String) -> Result<Url, sqlx::Error> {
    let now = chrono::Utc::now().naive_utc();
    let url = sqlx::query_as!(
        Url,
        r#"
        INSERT INTO url (original_url, short_url, click_count, created_at, updated_at)
        VALUES (?, '', ?, ?, ?)
        RETURNING id, original_url, short_url, click_count, created_at, updated_at
        "#,
        original_url,
        0,
        now,
        now
    )
    .fetch_one(pool)
    .await?;
    Ok(url)
}

pub async fn get_urls(pool: &SqlitePool, limit: i64, offset: i64) -> Result<Vec<Url>, sqlx::Error> {
    let urls = sqlx::query_as!(Url, "SELECT * FROM url LIMIT ? OFFSET ?", limit, offset)
        .fetch_all(pool)
        .await?;
    Ok(urls)
}

pub async fn get_url_by_short(
    pool: &SqlitePool,
    short_url: &str,
) -> Result<Option<Url>, sqlx::Error> {
    let url = sqlx::query_as!(Url, "SELECT * FROM url WHERE short_url = ?", short_url)
        .fetch_optional(pool)
        .await?;
    Ok(url)
}

#[allow(dead_code)]
pub async fn get_url_by_long(
    pool: &SqlitePool,
    original_url: &str,
) -> Result<Option<Url>, sqlx::Error> {
    let url = sqlx::query_as!(
        Url,
        "SELECT * FROM url WHERE original_url = ?",
        original_url
    )
    .fetch_optional(pool)
    .await?;
    Ok(url)
}

pub async fn increment_click_count(
    pool: &SqlitePool,
    short_url: &str,
) -> Result<Option<Url>, sqlx::Error> {
    let url = sqlx::query_as!(
        Url,
        r#"
        UPDATE url
        SET click_count = click_count + 1
        WHERE short_url = ?
        RETURNING id, original_url, short_url, click_count, created_at, updated_at
        "#,
        short_url
    )
    .fetch_optional(pool)
    .await?;
    Ok(url)
}

pub async fn cleanup_not_used_urls(pool: &SqlitePool, days: i64) -> Result<u64, sqlx::Error> {
    let threshold = chrono::Utc::now().naive_utc() - chrono::Duration::days(days);
    let result = sqlx::query!("DELETE FROM url WHERE updated_at < ?", threshold)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

#[allow(dead_code)]
pub async fn delete_url(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM url WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn update_short_url(
    pool: &SqlitePool,
    id: i64,
    short_url: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!("UPDATE url SET short_url = ? WHERE id = ?", short_url, id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
