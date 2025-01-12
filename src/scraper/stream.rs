use std::collections::HashMap;
use async_stream::stream;
use tokio_stream::Stream;

use super::{
    req::{
        ReqSession,
        ReqMethod
    },
    super::{
        config as cfg,
        models::{
            scraper::ProductData,
            api::{
                Task,
                TaskProgress,
                TaskResult,
                TaskStatus
            }
        }
    }
};


//fn task_handler(mut task: Task) -> Task {

//}

pub async fn task_stream(mut task: Task) -> impl Stream<Item = Task> {
    let order_data = task.extract_order_data();
    let req_session_res = ReqSession::new(
        &cfg::get().req_session,
        ReqMethod::Combined,
        &order_data.cookies,
        order_data.proxy_pool
    ).await;
    match req_session_res {
        Ok(rs) => {
            task.init_progress();
            task.set_status(TaskStatus::Processing);
            task.init_result_data();
        },
        Err(e) => {
            task.set_status(TaskStatus::Error);
            task.set_result_error(e.to_string());
        }
    };
    //req_session_res.unwrap().close().await;

    let stream = stream! {
        while !task.is_done_by_status() {

            let order_item = order_data.products.get(
                task.get_curr_step() as usize
            ).unwrap().clone();

            let rand_product_data = ProductData::rand();
            let product_result = Some(rand_product_data);

            task.insert_result_item(order_item, product_result);

            task.next_progress_step();
            if task.is_done_by_progress() {
                task.set_status(TaskStatus::Completed);
            }
            yield task.clone();

        }
        yield task;
    };

    Box::pin(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api() {
        let products = vec![
            "123".to_string(),
            "444".to_string(),
            "12344".to_string(),
            "44411".to_string(),
            "12311".to_string(),
            "44466".to_string(),
        ];
        // Нахождение товара для озон и запрос этого товара для подгрузки cookis
        if let Some(p) = products.iter().find(|p| p.starts_with("4")) {
            println!("{p}")
        }

        let status = false;

        while status {
            println!("status");
            break;
        }

        // Обязательна проверка состояний последних запросов. Если последние 10 запросов закончились неудачей прервать обработку заказа
    }
}
