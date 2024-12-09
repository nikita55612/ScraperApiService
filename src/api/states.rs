use std::{
    collections::HashMap, 
    sync::Arc
};
use sqlx::SqlitePool;
use tokio::{
    sync::{
        mpsc::{
            self, 
            Receiver, 
            Sender
        }, 
        RwLock
    }, 
    task::JoinHandle
};
use tokio_stream::StreamExt;
use super::{
    database as db, 
    error::ApiError, 
    models::{Order, Task, TaskStatus}, 
    stream::task_stream
};


struct TaskHandler {
    pub task_heap: Arc<RwLock<HashMap<String, Task>>>,
    pub sender: Sender<String>,
    pub join_handle: JoinHandle<()>
}

impl TaskHandler {
    pub async fn run(db_pool: Arc<SqlitePool>) -> Self {
        let task_heap = Arc::new(
            RwLock::new(HashMap::with_capacity(10))
        );
        let (sender, receiver) = mpsc::channel::<String>(10);
        let join_handle = Self::spawn_handler(
            db_pool,
            receiver, 
            task_heap.clone()
        ).await;
        Self {
            task_heap: task_heap,
            sender,
            join_handle
        }
    }

    async fn spawn_handler(
        db_pool: Arc<db::Pool>,
        mut receiver: Receiver<String>, 
        task_heap: Arc<RwLock<HashMap<String, Task>>>
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(task_key) = receiver.recv().await {
                let task = task_heap.read().await.get(&task_key).unwrap().clone();
                let mut stream = task_stream(task.clone());

                while let Some(task) = stream.next().await {

                    if matches!(task.status, TaskStatus::Completed | TaskStatus::Error) {
                        if task_heap.write().await.remove(&task_key).is_some() {
                            task_heap.write().await.values_mut()
                                .for_each(|t| t.queue_number -= 1);
                        }

                        let _ = db::insert_task(&db_pool, &task).await;
                    } else {
                        task_heap.write().await.insert(task_key.clone(), task);
                    }
                }
            }
        })
    }

    pub async fn registering_task(&self, mut task: Task) -> Result<String, ApiError> {
        let task_count = self.task_heap.read().await.len() as u64;
        if task_count >= 10 { 
            return Err(ApiError::Info("task_count >= 10".into())); 
        }

        task.queue_number = task_count;
        let order_hash = task.order_hash.clone();
        if !self.task_heap.read().await.contains_key(&order_hash) {
            self.task_heap.write().await.insert(order_hash.clone(), task);

            if self.sender.send(order_hash.clone()).await.is_err() {
                return Err(ApiError::Info("Sender Err".into()));
            }
            return Ok(order_hash);
        }
        Err(ApiError::Info("not contains_key".into()))
    }

    pub async fn contains_task(&self, key: &String) -> bool {
        self.task_heap.read().await.contains_key(key)
    }

    pub async fn get_task(&self, key: &String) -> Option<Task> {
        self.task_heap.read().await.get(key).map(|t| t.clone())
    }

    pub async fn with_task<R>(&self, key: &String, f: impl FnOnce(&Task) -> R) -> Option<R> {
        self.task_heap.read().await.get(key).map(f)
    }

    pub async fn len(&self) -> usize {
        self.task_heap.read().await.len()
    }

    pub async fn abort(&self) {
        self.join_handle.abort();
    }
}

pub struct AppState {
    pub db_pool: Arc<db::Pool>,
    pub task_handlers: Vec<TaskHandler>,
    pub handlers_count: usize
}

impl AppState {
    pub async fn new(db_pool: Arc<db::Pool>, handlers_count: usize) -> Self {
        let mut task_handlers = Vec::with_capacity(handlers_count);
        for _ in 0..handlers_count {
            task_handlers.push(
                TaskHandler::run(
                    db_pool.clone()
                ).await
            );
        }
        Self {
            db_pool,
            task_handlers,
            handlers_count
        }
    }

    pub async fn insert_order(&self, order: Order) -> Result<String, ApiError> {
        let task = Task::from_order(order);
        let handler_index = self.select_handler_index().await;
        self.task_handlers.get(handler_index).unwrap()
            .registering_task(task).await
    }

    pub async fn get_task_state(&self, task_id: &String) -> Result<String, ApiError> {
        for th in self.task_handlers.iter() {
            if th.contains_task(task_id).await {
                if let Some(task) = th.get_task(task_id).await {
                    return Ok(
                        serde_json::to_string(&task)
                            .unwrap_or_default()
                    );
                }
                return Err(ApiError::Unknown);
            }
        }
        db::cutout_task(&self.db_pool, task_id).await
            .map_err(|_| ApiError::Unknown)
    }

    async fn select_handler_index(&self) -> usize {
        if self.handlers_count == 1 {
            return 0;
        }
        let mut task_handlers_queue: Vec<usize> = Vec::with_capacity(
            self.handlers_count
        );
        for th in self.task_handlers.iter() {
            task_handlers_queue.push(
                th.len().await
            );
        }
        task_handlers_queue.iter()
            .enumerate()
            .min_by_key(|(_, value)| *value)
            .unwrap_or(
                (0, &0)
            ).0
    }
}