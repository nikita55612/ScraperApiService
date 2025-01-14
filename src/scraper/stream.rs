use std::collections::HashMap;
use once_cell::sync::Lazy;
use async_stream::stream;
use browser_bridge::PageParam;
use tokio_stream::Stream;

use crate::models::scraper::Product;

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


static INTERRUPT_CHECK_STEP: Lazy<u64> = Lazy::new(|| cfg::get().api.interrupt_check_step);


// struct SkipMap {
//     oz: bool,
//     ym: bool,
//     mm: bool
// }

// impl SkipMap {
//     fn new() -> Self {
//         Self {
//             oz: false,
//             ym: false,
//             mm: false
//         }
//     }

//     fn is_skipped(&self, item: &str) -> bool {
//         if let Some((_, s)) = item.split_once('/') {
//             return match s {
//                 "oz" => self.oz,
//                 "ym" => self.ym,
//                 "mm" => self.mm,
//                 _ => false
//             };
//         }

//         false
//     }
// }

pub async fn task_stream(mut task: Task) -> impl Stream<Item = Task> {
    //let mut skip_map = SkipMap::new();
    let intpt_check_step = *INTERRUPT_CHECK_STEP;
    task.init_progress();
    let order_data = task.extract_order_data();
    let req_session_res = ReqSession::new(
        &cfg::get().req_session,
        ReqMethod::Combined,
        &order_data.cookies,
        order_data.proxy_pool
    ).await;

    let stream = stream! {
        match req_session_res {
            Ok(mut req_session) => {
                task.set_status(TaskStatus::Processing);
                task.init_result_data();
                if let Some(p) = order_data.products
                    .iter()
                    .find(|p| p.starts_with("oz"))
                {
                    let product = Product::from_string_without_valid(p);
                    let _ = req_session.browser_open_page(
                        &product.get_parse_url(),
                        &PageParam {
                            duration: 180,
                            ..Default::default()
                        }
                    ).await;
                }
                while !task.is_done_by_status() {
                    let step = task.get_curr_step();
                    let order_item = order_data.products[step as usize].clone();
                    //if skip_map.is_skipped(&order_item) {
                    //    task.insert_result_item(order_item, None)
                    //} else {
                    let product = Product::from_string_without_valid(&order_item);
                    let product_data = req_session
                        .req_product_data(&product)
                        .await;
                    match product_data {
                        Ok(product_result) =>
                            task.insert_result_item(order_item, product_result),
                        Err(_) => task.insert_result_item(order_item, None)
                    }
                    //}
                    task.next_progress_step();
                    let step = task.get_curr_step();
                    if step >= intpt_check_step && step % intpt_check_step == 0 {
                        if let Some(data) = task.extract_result_data() {
                            if data.values().skip(data.len() - intpt_check_step as usize)
                                .all(|v| v.is_none())
                            {
                                task.set_status(TaskStatus::Interrupted);
                            }
                        }
                    }
                    if task.is_done_by_progress() {
                        task.set_status(TaskStatus::Completed);
                    }
                    yield task.clone();
                }
                req_session.close().await;
            },
            Err(e) => {
                task.set_status(TaskStatus::Error);
                task.set_result_error(e.into());
                yield task;
            }
        };
    };

    Box::pin(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_stream() {
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

        let mut status = true;
        let mut progress = 0;
        let end = 22;


        while status {
            progress += 1;
            if progress >= end {
                status = !status;
            }
            println!("status {}", progress);
        }
    }
}
