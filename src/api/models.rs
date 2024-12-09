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
pub struct TaskProgress {
    pub done: u64,
    pub total: u64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all="lowercase")]
pub enum TaskResult {
    #[serde(rename="data")]
    Data(String),
    #[serde(rename="error")]
    Error(String)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OrderTypes {
    #[serde(rename="products")]
    Products,
}

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
    pub queue_number: u64,
    pub status: TaskStatus,
    pub progress: Option<TaskProgress>,
    pub result: Option<TaskResult>,
    pub created: u64,
}

impl TaskProgress {
    pub fn new(done: u64, total: u64) -> Self {
        Self { done, total }
    }

    pub fn next_step(&mut self) {
        self.done += 1;
    }
}

impl Task {
    pub fn from_order(order: Order) -> Self {
        let order_data = serde_json::to_string(&order).unwrap();
        let order_hash = sha1_hash(order_data.as_bytes());
        Self {
            order: order,
            order_hash: order_hash,
            queue_number: 0,
            status: TaskStatus::Waiting,
            progress: None,
            result: None,
            created: timestamp_now(),
        }
    }
}