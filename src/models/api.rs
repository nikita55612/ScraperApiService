#![allow(warnings)]
use std::collections::HashMap;
use serde::{
    Serialize,
    Deserialize
};

use super::super::models::scraper::ProductData;
use super::super::api::error::ApiError;
use super::super::utils::{
    remove_duplicates,
    create_token_id,
    timestamp_now,
    sha1_hash
};
use super::validation::Validation;


type OrderHash = String;

#[derive(Clone, Debug, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub struct Token {
    pub id: String,
	#[serde(rename="createdAt")]
    pub created_at: u64,
    pub ttl: u64,
    #[serde(rename="orderProductsLimit")]
    pub op_limit: u64,
    #[serde(rename="taskCountLimit")]
    pub tc_limit: u64
}

impl Token {
    pub fn new(ttl: u64, op_limit: u64, tc_limit: u64) -> Self  {
        Self {
            id: create_token_id(),
            created_at: timestamp_now(),
            ttl,
            op_limit,
            tc_limit
        }
    }

    pub fn is_expired(&self) -> bool {
        (self.created_at + self.ttl) - timestamp_now() < 0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all="lowercase")]
pub enum TaskStatus {
    Waiting,
    Processing,
    Completed,
    Interrupted,
    Error
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all="lowercase")]
pub enum TaskResult {
    Data(HashMap<String, Option<ProductData>>),
    Error(String)
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct Order {
	#[serde(skip)]
    pub token_id: String,
    pub products: Vec<String>,
	#[serde(rename="proxyPool")]
    pub proxy_pool: Vec<String>,
    //#[serde(rename="proxyMap")]
    //pub proxy_map: HashMap<String, Vec<String>>,
	#[serde(rename="cookies")]
	pub cookies: Vec<OrderCookieParam>,
}

impl Order {
    fn sha1_hash(&self) -> OrderHash {
        let mut sort_data = self.products.clone();
        sort_data.sort();
        let order_hash_data = format!(
            "{}.{}",
            self.token_id,
            sort_data.join(",")
        );

        sha1_hash(
            order_hash_data.as_bytes()
        )
    }

    pub fn remove_duplicates(&mut self) {
        if !self.products.is_empty() {
            remove_duplicates(&mut self.products);
        }
        if !self.proxy_pool.is_empty() {
            remove_duplicates(&mut self.proxy_pool);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderCookieParam {
    pub name: String,
    pub value: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename="httpOnly", skip_serializing_if = "Option::is_none")]
    pub http_only: Option<bool>,
    #[serde(rename="sameSite", skip_serializing_if = "Option::is_none")]
    pub same_site: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure: Option<bool>
}

#[derive(Clone, Debug, Default)]
pub struct OrderExtractData {
    pub products: Vec<String>,
    pub proxy_pool: Vec<String>,
    //pub proxy_map: HashMap<String, Vec<String>>,
	pub cookies: Vec<OrderCookieParam>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    #[serde(skip)]
    pub order: Order,
    #[serde(skip)]
    pub order_hash: OrderHash,

    #[serde(rename="queueNum")]
    pub queue_num: u64,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<TaskProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<TaskResult>,
    #[serde(rename="createdAt")]
    pub created_at: u64,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.order_hash == other.order_hash &&
        self.queue_num == other.queue_num &&
        self.status == other.status &&
        self.progress == other.progress &&
        self.queue_num == other.queue_num
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TaskProgress(u64, u64);

impl TaskProgress {
    pub fn new(done: u64, total: u64) -> Self {
        Self (done, total)
    }

    pub fn next_step(&mut self) {
        self.0 += 1;
    }
}

impl Task {
    pub fn from_order(order: Order) -> Self {
        let order_hash = order.sha1_hash();
        Self {
            order: order,
            order_hash,
            queue_num: 0,
            status: TaskStatus::Waiting,
            progress: None,
            result: None,
            created_at: timestamp_now(),
        }
    }

    pub fn is_done_by_status(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Completed
            | TaskStatus::Error
            | TaskStatus::Interrupted
        )
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status
    }

    pub fn init_result_data(&mut self) {
        self.result = Some(
            TaskResult::Data(
                HashMap::new()
            )
        )
    }

    pub fn set_result_error(&mut self, error: String) {
        self.result = Some(
            TaskResult::Error(error)
        )
    }

    pub fn extract_order_data(&mut self) -> OrderExtractData {
        let extract_data = OrderExtractData {
            products: std::mem::take(&mut self.order.products),
            proxy_pool: std::mem::take(&mut self.order.proxy_pool),
            //proxy_map: std::mem::take(&mut self.order.proxy_map),
            cookies: std::mem::take(&mut self.order.cookies),
        };

        extract_data
    }

    pub fn insert_result_item(&mut self, k: String, v: Option<ProductData>) {
        if let Some(
            TaskResult::Data(items_map)
        ) = &mut self.result {
            items_map.insert(k, v);
        }
    }

    pub fn set_progress(&mut self, done: u64, total: u64) {
        self.progress = Some(TaskProgress::new(done, total));
    }

    pub fn init_progress(&mut self) {
        let total = self.order.products.len() as u64;
        self.set_progress(0, total);
    }

    pub fn next_progress_step(&mut self) {
        if let Some(progress) = self.progress.as_mut() {
            progress.next_step();
        }
    }

    pub fn get_curr_step(&self) -> u64 {
        if let Some(TaskProgress(done, _)) = &self.progress {
            return *done;
        }

        0
    }

    pub fn is_done_by_progress(&self) -> bool {
        if let Some(progress) = &self.progress {
            return progress.0 == progress.1;
        }

        false
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiState {
    pub handlers_count: usize,
    pub tasks_queue_limit: usize,
    pub curr_task_queue: usize,
    pub open_ws_limit: u32,
    pub curr_open_ws: u32
}
