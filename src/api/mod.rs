pub mod database;
pub mod routers;
pub mod stream;
pub mod config;
pub mod models;
pub mod states;
pub mod utils;
pub mod error;


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_api() {
        println!("Its a API mod test!");

        let db_pool = Arc::new(
            database::init().await.unwrap()
        );
        let app_state = states::AppState::new(db_pool, 2).await;

        let mut task_id = String::new();

        for n in 0..6 {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            println!("Task №{n}");
            let order = models::Order {
                token_id: utils::gen_token_id(),
                type_: models::OrderTypes::Products,
                items: vec![
                    "oz/1234567890".into(), 
                    "oz/1234567891".into(),
                    "oz/9999967890".into(), 
                    "oz/7777767891".into(),
                    format!("oz/{}", utils::timestamp_now())
                    ],
                proxy_list: vec![
                    "EyPrWhn4uZ:wN1qqx1gPH@178.255.30.223:11223".into(),
                    "DF3fdv4uZ:w3ER56bi1gRp@185.255.30.168:11223".into(),
                    format!("oz/{}", utils::gen_uuid())
                    ]
            };
            println!("Order token_id: {}", order.token_id);
            match app_state.insert_order(order).await {
                Ok(ti) => { task_id = ti; },
                Err(e) => { 
                    println!("{:?}", e); 
                    panic!()
                }
            } 
        }

        
        loop {
            match app_state.get_task_state(&task_id).await {
                Ok(ts) => println!("{ts}"),
                Err(_) => break
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(700)).await;
        }
        assert_eq!(true, true);
    }
}