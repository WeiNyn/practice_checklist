use chrono::NaiveDateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Sqlite, SqlitePool, migrate::MigrateDatabase};

pub const DB_URL: &str = "sqlite://todoapp.db";

pub async fn init_db(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    if !Sqlite::database_exists(db_url).await? {
        Sqlite::create_database(db_url).await?;
    }
    let pool = SqlitePool::connect(db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Todo {
    pub id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

pub async fn create_todo(
    pool: &SqlitePool,
    title: String,
    description: Option<String>,
) -> Result<Todo, sqlx::Error> {
    let now = Utc::now();
    let todo = sqlx::query_as!(
        Todo,
        r#"
        INSERT INTO todo (title, description, completed, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING id, title, description, completed, created_at, updated_at
        "#,
        title,
        description,
        false,
        now,
        now
    )
    .fetch_one(pool)
    .await?;
    Ok(todo)
}

pub async fn get_todos(pool: &SqlitePool) -> Result<Vec<Todo>, sqlx::Error> {
    let todos = sqlx::query_as!(Todo, "SELECT * FROM todo")
        .fetch_all(pool)
        .await?;
    Ok(todos)
}

pub async fn update_todo(
    pool: &SqlitePool,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
) -> Result<Todo, sqlx::Error> {
    let now = Utc::now();
    let todo = sqlx::query_as!(
        Todo,
        r#"
        UPDATE todo
        SET title = COALESCE(?, title),
            description = COALESCE(?, description),
            completed = COALESCE(?, completed),
            updated_at = ?
        WHERE id = ?
        RETURNING id, title, description, completed, created_at, updated_at
        "#,
        title,
        description,
        completed,
        now,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(todo)
}

pub async fn delete_todo(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM todo WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_todo_by_id(pool: &SqlitePool, id: i64) -> Result<Todo, sqlx::Error> {
    let todo = sqlx::query_as!(Todo, "SELECT * FROM todo WHERE id = ?", id)
        .fetch_one(pool)
        .await?;
    Ok(todo)
}

pub async fn get_todos_by_completion(
    pool: &SqlitePool,
    completed: bool,
) -> Result<Vec<Todo>, sqlx::Error> {
    let todos = sqlx::query_as!(Todo, "SELECT * FROM todo WHERE completed = ?", completed)
        .fetch_all(pool)
        .await?;
    Ok(todos)
}

pub async fn get_todos_by_time_range(
    pool: &SqlitePool,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<Vec<Todo>, sqlx::Error> {
    let todos = sqlx::query_as!(
        Todo,
        "SELECT * FROM todo WHERE created_at BETWEEN ? AND ?",
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;
    Ok(todos)
}

#[cfg(test)]
mod tests {
    use chrono::Days;

    use super::*;

    async fn init_test_db() -> Result<SqlitePool, sqlx::Error> {
        let db_url = "sqlite://test.db";
        init_db(db_url).await
    }

    async fn cleanup_test_db() -> Result<(), sqlx::Error> {
        let db_url = "sqlite://test.db";
        if Sqlite::database_exists(db_url).await? {
            Sqlite::drop_database(db_url).await?;
        }
        Ok(())
    }

    async fn test_get_todos_empty(pool: &SqlitePool) {
        let todos = get_todos(&pool).await;
        assert!(todos.is_ok());
        let todos = todos.unwrap();
        assert!(todos.is_empty()); // Initially, the database should be empty
    }

    async fn test_create_todo(pool: &SqlitePool) {
        let todo = create_todo(&pool, "Test Todo".to_string(), None).await;
        assert!(todo.is_ok());
        let todo = todo.unwrap();
        assert_eq!(todo.title, "Test Todo");
        assert!(!todo.completed);
    }

    async fn test_get_todos(pool: &SqlitePool) {
        let todos = get_todos(&pool).await;
        assert!(todos.is_ok());
        let todos = todos.unwrap();
        assert!(!todos.is_empty()); // There should be at least one todo
        assert_eq!(todos[0].title, "Test Todo"); // Check the title of the created todo
    }

    async fn test_update_todo(pool: &SqlitePool) {
        let todo = create_todo(&pool, "Update Test".to_string(), None)
            .await
            .unwrap();
        let updated_todo = update_todo(
            &pool,
            todo.id.unwrap(),
            Some("Updated Title".to_string()),
            None,
            None,
        )
        .await;
        assert!(updated_todo.is_ok());
        let updated_todo = updated_todo.unwrap();
        assert_eq!(updated_todo.title, "Updated Title");
    }

    async fn test_delete_todo(pool: &SqlitePool) {
        let todo = create_todo(&pool, "Delete Test".to_string(), None)
            .await
            .unwrap();
        let delete_result = delete_todo(&pool, todo.id.unwrap()).await;
        assert!(delete_result.is_ok());
        let todos = get_todos(&pool).await.unwrap();
        assert!(todos.iter().all(|t| t.id != todo.id)); // The todo should be deleted
    }

    async fn test_get_todo_by_id(pool: &SqlitePool) {
        let todo = create_todo(&pool, "Get by ID Test".to_string(), None)
            .await
            .unwrap();
        let fetched_todo = get_todo_by_id(&pool, todo.id.unwrap()).await;
        assert!(fetched_todo.is_ok());
        let fetched_todo = fetched_todo.unwrap();
        assert_eq!(fetched_todo.title, "Get by ID Test");
    }

    async fn test_get_todos_by_completion(pool: &SqlitePool) {
        let _ = create_todo(&pool, "Get by Completion Test".to_string(), None)
            .await
            .unwrap();
        let fetched_todos = get_todos_by_completion(&pool, false).await;
        assert!(fetched_todos.is_ok());
        let fetched_todos = fetched_todos.unwrap();
        assert_eq!(fetched_todos.len(), 4);
        assert_eq!(fetched_todos[3].title, "Get by Completion Test");
    }

    async fn test_get_todos_by_date_range(pool: &SqlitePool) {
        let start_date = Utc::now()
            .naive_utc()
            .checked_sub_days(Days::new(1))
            .unwrap();
        let _ = create_todo(&pool, "Get by Date Range Test".to_string(), None)
            .await
            .unwrap();
        let end_date = Utc::now()
            .naive_utc()
            .checked_add_days(Days::new(1))
            .unwrap();
        let fetched_todos = get_todos_by_time_range(&pool, start_date, end_date).await;
        assert!(fetched_todos.is_ok());
        let fetched_todos = fetched_todos.unwrap();
        assert_eq!(fetched_todos.len(), 5);
        assert_eq!(fetched_todos[4].title, "Get by Date Range Test");
    }

    #[tokio::test]
    async fn run_tests() {
        let pool = init_test_db().await;

        assert!(pool.is_ok());

        let pool = pool.unwrap();

        test_get_todos_empty(&pool).await;
        test_create_todo(&pool).await;
        test_get_todos(&pool).await;
        test_update_todo(&pool).await;
        test_delete_todo(&pool).await;
        test_get_todo_by_id(&pool).await;
        test_get_todos_by_completion(&pool).await;
        test_get_todos_by_date_range(&pool).await;

        cleanup_test_db()
            .await
            .expect("Failed to clean up test database");
    }
}
