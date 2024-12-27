#![allow(warnings)]
use async_stream::stream;
use tokio_stream::Stream;
use super::super::models::{
    api::{
        Task,
        TaskProgress,
        TaskStatus,
        TaskResult,
    },
    scraper::ProductData
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


pub fn task_stream(mut task: Task) -> impl Stream<Item = Task> {

    let stream = stream! {

        task.init_progress();
        task.set_status(TaskStatus::Processing);
        task.init_result_data();
        let order_data = task.extract_order_data();

        while !matches!(task.status, TaskStatus::Completed | TaskStatus::Error) {

            tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;

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
    };
    Box::pin(stream)
}
