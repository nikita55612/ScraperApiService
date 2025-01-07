use std::collections::HashMap;
use async_stream::stream;
use tokio_stream::Stream;

use super::{
    req::{
        ReqSession,
        ReqMethod
    },
    super::models::{
        scraper::ProductData,
        api::{
            Task,
            TaskProgress,
            TaskResult,
            TaskStatus
        }
    }
};


//fn task_handler(mut task: Task) -> Task {

//}

pub async fn task_stream(mut task: Task) -> impl Stream<Item = Task> {
    let order_data = task.extract_order_data();
    let req_session = ReqSession::new(
        ReqMethod::Combined,
        &order_data.cookies,
        order_data.proxy_pool
    ).await;

    match req_session {
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

    let stream = stream! {
        while !matches!(task.status, TaskStatus::Completed | TaskStatus::Error) {

            let order_item = order_data.products.get(
                task.get_curr_step() as usize
            ).unwrap().clone();

            let rand_product_data = ProductData::rand();
            let product_result = Some(rand_product_data);

            task.insert_result_item(order_item, product_result);

            task.next_progress_step();
            if task.is_done() {
                task.set_status(TaskStatus::Completed);
            }
            yield task.clone();

        }
        yield task;
    };
    Box::pin(stream)
}
