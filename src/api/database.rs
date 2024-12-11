use sqlx::{
    sqlite::SqlitePoolOptions, 
    migrate::MigrateDatabase, 
    SqlitePool,
    Sqlite, 
};
use super::models::{
    Token,
    Task
};
use super::config as cfg;


type Error = Box<dyn std::error::Error>;
type Result<T> = core::result::Result<T, Error>;
pub type Pool = SqlitePool;


pub async fn init() -> Result<Pool> {
    let db = &cfg::get().api.db;

    if !Sqlite::database_exists(db).await? {
        Sqlite::create_database(db).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(
            cfg::get().api.db_max_conn
        )
        .connect(db)
        .await?;

    sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tokens (
                id TEXT PRIMARY KEY,
                master INTEGER NOT NULL CHECK (master IN (0, 1)),
                created_at INTEGER NOT NULL
            );"#
        )
        .execute(&pool)
        .await?;

    sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS completed_tasks (
                id TEXT PRIMARY KEY,
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

pub async fn insert_token(
    pool: &Pool, 
    token: &Token
) -> Result<()> {
    sqlx::query("INSERT INTO tokens (id, master, created_at) VALUES (?, ?, ?);")
      .bind(token.id.as_str())
      .bind(token.master)
      .bind(token.created_at as i64)
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

pub async fn cutout_token(pool: &Pool, token_id: &str) -> Result<Token> {
    let token: Token = sqlx::query_as(
        "DELETE FROM tokens WHERE id = ? RETURNING id, master, created_at"
        )
        .bind(token_id)
        .fetch_one(pool)
        .await?;

    Ok(token)
}

pub async fn insert_task(
    pool: &Pool, 
    task: &Task
) -> Result<()> {
    let task_data = serde_json::to_string(task)?;
    sqlx::query("INSERT INTO completed_tasks (id, data) VALUES (?, ?);")
      .bind(task.order_hash.as_str())
      .bind(task_data)
      .execute(pool)
      .await?;
    
    Ok(())
}

pub async fn task_exists(pool: &Pool, task_id: &str) -> Result<bool> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM completed_tasks WHERE id = ?;"
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

    Ok(row.is_some())
}

pub async fn cutout_task(pool: &Pool, task_id: &str) -> Result<String> {
    let task_data: (String,) = sqlx::query_as(
        "DELETE FROM completed_tasks WHERE id = ? RETURNING data"
        )
        .bind(task_id)
        .fetch_one(pool)
        .await?;

    Ok(task_data.0)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models;
    use crate::api::utils;

    #[tokio::test]
    async fn test_init() {
        let init_result = init().await;
        assert_eq!(init_result.is_ok(), true);
    }

    #[tokio::test]
    async fn test_insert_token() {
        let pool = init().await.unwrap();
        let token = Token::new(false);
        println!("{:?}", token);
        let insert_token_result = insert_token(
            &pool,
            &token
        ).await;
        assert_eq!(insert_token_result.is_ok(), true);
    }

    #[tokio::test]
    async fn test_cutout_token() {
        let pool = init().await.unwrap();
        let token = Token::new(false);
        println!("{:?}", token);
        let insert_token_result = insert_token(
            &pool,
            &token
        ).await;
        let cutout_token_result = cutout_token(
            &pool,
            &token.id
        ).await;
        let cutout_token = cutout_token_result.as_ref().ok();
        println!("Cutout token: {:?}", cutout_token);
        assert_eq!(
            (
                insert_token_result.is_ok(), 
                cutout_token_result.is_ok(),
                cutout_token
            ),
            (
                true, 
                true,
                Some(&token)
            )
        );
    }

    #[tokio::test]
    async fn test_token_exists() {
        let pool = init().await.unwrap();
        let token_id = "ss.81240d5c7b7e461db5c3b9a1c7b9b8f5";
        assert_eq!(
            token_exists(&pool, token_id).await.ok(), 
            Some(true)
        );
    }

    fn create_task() -> Task {
        let order = models::Order {
            token_id: utils::gen_token_id(),
            type_: models::OrderTypes::Products,
            items: vec![
                "oz/1234567890".into(), 
                "oz/1234567891".into(),
                "oz/9999967890".into(), 
                "oz/7777767891".into()
                ],
            proxy_list: vec![
                "EyPrWhn4uZ:wN1qqx1gPH@178.255.30.223:11223".into(),
                "DF3fdv4uZ:w3ER56bi1gRp@185.255.30.168:11223".into()
                ]
        };
        Task::from_order(order)
    }

    #[tokio::test]
    async fn test_insert_task() {
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
    async fn test_cutout_task() {
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
            println!("{task}");
        }
        assert_eq!(insert_task_result.is_ok(), true);
    }

    #[tokio::test]
    async fn test_task_exists() {
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