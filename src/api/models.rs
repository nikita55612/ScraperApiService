use serde::{Serialize, Deserialize};
use super::utils::{
    gen_token_id,
    timestamp_now,
    sha1_hash
};


#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub struct Token {
    pub id: String,
    pub master: bool,
    pub created_at: u64
}

impl Token {
    pub fn new(master: bool) -> Self  {
        Self {
            id: gen_token_id(),
            master: master,
            created_at: timestamp_now()
        }
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

// Необходимо создать отдельную структуру для конфигурации закза
// proxy_list должен входить в конфигурацию заказа
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Order {
    #[serde(skip_serializing)]
    pub token_id: String,
    #[serde(rename="type")]
    pub type_: OrderTypes,
    pub items: Vec<String>,
    pub proxy_list: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    #[serde(skip_serializing)]
    pub order: Order,
    #[serde(skip_serializing)]
    pub order_hash: String,
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
        let order_data = serde_json::to_string(&order).unwrap();
        let order_hash = sha1_hash(order_data.as_bytes());
        Self {
            order: order,
            order_hash: order_hash,
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

