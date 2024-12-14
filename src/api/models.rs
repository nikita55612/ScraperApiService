#![allow(warnings)]
use serde::{Serialize, Deserialize};
use super::super::utils::{
    gen_token_id,
    timestamp_now,
    sha1_hash
};


type OrderHash = String;


#[derive(Clone, Debug, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub struct Token {
    pub id: String,
    pub created_at: u64,
    pub ttl: u64,
    pub ilimit: u64
}

impl Token {
    pub fn new(ttl: u64, ilimit: u64) -> Self  {
        Self {
            id: gen_token_id(),
            created_at: timestamp_now(),
            ttl,
            ilimit
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
pub struct TaskProgress(u64, u64);

//TEMP
use super::stream::ProductData;
use std::collections::HashMap;
use std::default;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all="lowercase")]
pub enum ProductResult {
    Data(ProductData),
    Error(String)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all="lowercase")]
pub enum TaskResult {
    Data(HashMap<String, ProductResult>),
    Error(String)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OrderTypes {
    #[serde(rename="products")]
    Products,
}

impl std::fmt::Display for OrderTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Products => write!(f, "products")
        }
    }
}

impl Default for OrderTypes {
    fn default() -> Self {
        Self::Products
    }
}

// Можно сделать отдельную имплементацию для этой структуры вычисления sha1_hash необходимых полей
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Order {
    #[serde(skip)]
    pub token_id: String,
    #[serde(rename="type")]
    pub type_: OrderTypes,
    pub items: Vec<String>,
    pub proxy_list: Vec<String>
}

impl Order {
    fn sha1_hash(&self) -> OrderHash {
        let order_hash_data = format!(
            "{} {} {}",
            self.token_id,
            self.type_,
            self.items.join(",")
        );
        sha1_hash(order_hash_data.as_bytes())
    }
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
    pub created: u64,
}

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
            created: timestamp_now(),
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

    pub fn insert_result_item(&mut self, k: String, v: ProductResult) {
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
        let total = self.order.items.len() as u64;
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
