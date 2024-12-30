#![allow(warnings)]
use std::collections::HashMap;
use serde::{
    Serialize,
    Deserialize
};

use super::super::models::scraper::ProductData;
use super::super::api::error::ApiError;
use super::super::utils::{
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
    #[serde(rename="iLimit")]
    pub ilimit: u64,
    #[serde(rename="cLimit")]
    pub climit: u64
}

impl Token {
    pub fn new(ttl: u64, ilimit: u64, climit: u64) -> Self  {
        Self {
            id: create_token_id(),
            created_at: timestamp_now(),
            ttl,
            ilimit,
            climit
        }
    }

    pub fn is_expired(&self) -> bool {
        (self.created_at + self.ttl) - timestamp_now() < 0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all="lowercase")]
pub enum TaskStatus {
    Waiting,
    Processing,
    Completed,
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

	#[serde(rename="proxyList")]
    pub proxy_list: Vec<String>,
    #[serde(rename="proxyMap")]
    pub proxy_map: HashMap<String, String>,
	#[serde(rename="cookieList")]
	pub cookie_list: Vec<OrderCookiesParam>,
}

impl Order {
    fn sha1_hash(&self) -> OrderHash {
        let order_hash_data = format!(
            "{} {}",
            self.token_id,
            self.products.join(",")
        );

        sha1_hash(
            order_hash_data.as_bytes()
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct OrderExtractData {
    pub products: Vec<String>,
    pub proxy_list: Vec<String>,
	pub cookie_list: Vec<OrderCookiesParam>,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    pub fn extract_order_data(&mut self) -> OrderExtractData {
        let extract_data = OrderExtractData {
            products: std::mem::take(&mut self.order.products),
            proxy_list: std::mem::take(&mut self.order.proxy_list),
            cookie_list: std::mem::take(&mut self.order.cookie_list),
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

    pub fn is_done(&self) -> bool {
        if let Some(progress) = &self.progress {
            return progress.0 == progress.1;
        }

        false
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderCookiesParam {
    name: String,
    value: String,
    url: Option<String>,
    domain: Option<String>,
    path: Option<String>,
    #[serde(rename="httpOnly")]
    http_only: Option<bool>,
    #[serde(rename="sameSite")]
    same_site: Option<String>,
    secure: Option<bool>
}
