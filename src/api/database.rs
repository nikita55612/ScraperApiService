#![allow(warnings)]
use sqlx::{
    sqlite::SqlitePoolOptions,
    migrate::MigrateDatabase,
    SqlitePool,
    Sqlite,
};

use super::super::config as cfg;
use super::super::models::api::{
    Token,
    Task
};


type Result<T> = core::result::Result<T, sqlx::Error>;
pub type Pool = SqlitePool;


pub async fn init() -> Result<Pool> {
    let db_path = &cfg::get().api.db_path;

    if !Sqlite::database_exists(db_path).await? {
        Sqlite::create_database(db_path).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(
            cfg::get().api.db_max_conn
        )
        .connect(db_path)
        .await?;

    sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tokens (
                id TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL,
                ttl INTEGER NOT NULL,
                op_limit INTEGER NOT NULL,
                tc_limit INTEGER NOT NULL
            );"#
        )
        .execute(&pool)
        .await?;

    sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS completed_tasks (
                order_hash TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );"#
        )
        .execute(&pool)
        .await?;

    sqlx::query(r#"DELETE FROM completed_tasks;"#)
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn insert_token(pool: &Pool, token: &Token) -> Result<()> {
    sqlx::query(
        "INSERT INTO tokens (id, created_at, ttl, op_limit, tc_limit) VALUES (?, ?, ?, ?, ?);"
        )
        .bind(token.id.as_str())
        .bind(token.created_at as i64)
        .bind(token.ttl as i64)
        .bind(token.op_limit as i64)
        .bind(token.tc_limit as i64)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_token(pool: &Pool, token: &Token) -> Result<()> {
    sqlx::query(
            "UPDATE tokens SET ttl = ?, op_limit = ?, tc_limit = ? WHERE id = ?"
        )
        .bind(token.ttl as i64)
        .bind(token.op_limit as i64)
        .bind(token.tc_limit as i64)
        .bind(token.id.clone())
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn token_exists(pool: &Pool, token_id: &str) -> Result<bool> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM tokens WHERE id = ?;"
        )
        .bind(token_id)
        .fetch_optional(pool)
        .await?;

    Ok(row.is_some())
}

pub async fn read_token(pool: &Pool, token_id: &str) -> Result<Option<Token>> {
    let token: Option<Token> = sqlx::query_as(
        "SELECT * FROM tokens WHERE id = ?;"
        )
        .bind(token_id)
        .fetch_optional(pool)
        .await?;

    Ok(token)
}

pub async fn cutout_token(pool: &Pool, token_id: &str) -> Result<Option<Token>> {
    let token: Option<Token> = sqlx::query_as(
        "DELETE FROM tokens WHERE id = ? RETURNING *;"
        )
        .bind(token_id)
        .fetch_optional(pool)
        .await?;

    Ok(token)
}

pub async fn insert_task(
    pool: &Pool,
    task: &Task
) -> Result<()> {
    let task_data = serde_json::to_string(task).unwrap();
    sqlx::query(
        "INSERT INTO completed_tasks (order_hash, data) VALUES (?, ?);"
        )
        .bind(task.order_hash.as_str())
        .bind(task_data)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn task_exists(pool: &Pool, order_hash: &str) -> Result<bool> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM completed_tasks WHERE id = ?;"
        )
        .bind(order_hash)
        .fetch_optional(pool)
        .await?;

    Ok(row.is_some())
}

pub async fn cutout_task(pool: &Pool, order_hash: &str) -> Result<Task> {
    let task_data: (String,) = sqlx::query_as(
        "DELETE FROM completed_tasks WHERE order_hash = ? RETURNING data"
        )
        .bind(order_hash)
        .fetch_one(pool)
        .await?;

    Ok(serde_json::from_str(&task_data.0).unwrap())
}

pub async fn cutout_string_task(pool: &Pool, order_hash: &str) -> Result<String> {
    let task_data: (String,) = sqlx::query_as(
        "DELETE FROM completed_tasks WHERE order_hash = ? RETURNING data"
        )
        .bind(order_hash)
        .fetch_one(pool)
        .await?;

    Ok(task_data.0)
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    use std::collections::HashMap;
    use std::time::Duration;
    use crate::models::api as models;
    use crate::utils;


    #[tokio::test]
    async fn test_db_init() {
        let init_result = init().await;
        assert_eq!(init_result.is_ok(), true);
    }

    #[tokio::test]
    async fn test_db_insert_token() {
        let pool = init().await.unwrap();
        let token = Token::new(2592000, 250, 1);
        println!("{:?}", token);
        let insert_token_result = insert_token(
            &pool,
            &token
        ).await;
        assert_eq!(insert_token_result.is_ok(), true);
    }

    #[tokio::test]
    async fn test_db_read_token() {
        let pool = init().await.unwrap();
        let token = Token::new(2592000, 250, 1);
        println!("Insert token: {:?}", token);
        let _ = insert_token(
            &pool,
            &token
        ).await;
        let read_token_ = read_token(
            &pool,
            &token.id
        ).await
        .unwrap();
        println!("Read toke: {:?}", read_token_);
        assert_eq!(true, read_token_ == Some(token));
    }

    #[tokio::test]
    async fn test_db_cutout_token() {
        let pool = init().await.unwrap();
        let token = Token::new(2592000, 250, 1);
        println!("{:?}", token);
        let _ = insert_token(
            &pool,
            &token
        ).await;
        let cutout_token = cutout_token(
            &pool,
            &token.id
        ).await
        .unwrap();
        println!("Cutout token: {:?}", cutout_token);
        assert_eq!(true, true);
    }

    #[tokio::test]
    async fn test_db_token_exists() {
        let pool = init().await.unwrap();
        let token_id = "ss.81240d5c7b7e461db5c3b9a1c7b9b8f5";
        assert_eq!(
            token_exists(&pool, token_id).await.ok(),
            Some(true)
        );
    }

    fn create_task() -> Task {
        let order = models::Order {
            token_id: utils::create_token_id(),
            products: vec![
                "oz/1234567890".into(),
                "oz/1234567891".into(),
                "oz/9999967890".into(),
                "oz/7777767891".into()
                ],
            proxy_pool: vec![
                "EyPrWhn4uZ:wN1qqx1gPH@178.255.30.223:11223".into(),
                "DF3fdv4uZ:w3ER56bi1gRp@185.255.30.168:11223".into()
                ],
            //proxy_map: HashMap::new(),
            cookies: Vec::new()
        };
        Task::from_order(order)
    }

    #[tokio::test]
    async fn test_db_insert_task() {
        let pool = init().await.unwrap();
        let task = create_task();
        println!("{:?}", task);
        let insert_task_result = insert_task(
            &pool,
            &task
        ).await;
        assert_eq!(insert_task_result.is_ok(), true);
    }

    #[tokio::test]
    async fn test_db_cutout_task() {
        let pool = init().await.unwrap();
        let task = create_task();
        println!("{:?}", task);
        let insert_task_result = insert_task(
            &pool,
            &task
        ).await;
        let cutout_task_result = cutout_task(
            &pool,
            &task.order_hash
        ).await;
        let cutout_task_option = cutout_task_result.as_ref().ok();
        if let Some(task) = cutout_task_option {
            println!("{:?}", task);
        }
        assert_eq!(insert_task_result.is_ok(), true);
    }

    #[tokio::test]
    async fn test_db_task_exists() {
        let pool = init().await.unwrap();
        let task = create_task();
        println!("{:?}", task);
        let _ = insert_task(
            &pool,
            &task
        ).await;
        assert_eq!(
            task_exists(&pool, &task.order_hash).await.unwrap(),
            true
        );
    }
}
