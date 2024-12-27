#![allow(warnings)]
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
    super::models::api::{
        Order,
        Token,
        Task,
        TaskStatus
    },
    stream::task_stream,
    super::config as cfg
};


type OrderHash = String;

struct TaskHandler {
    pub task_heap: Arc<RwLock<HashMap<OrderHash, Task>>>,
    pub queue_limit: u64,
    pub sender: Sender<OrderHash>,
    pub join_handle: JoinHandle<()>
}

impl TaskHandler {
    pub async fn run(db_pool: Arc<SqlitePool>, queue_limit: usize) -> Self {
        let task_heap = Arc::new(
            RwLock::new(
                HashMap::with_capacity(queue_limit)
            )
        );
        let (
            sender,
            receiver
        ) = mpsc::channel::<OrderHash>(queue_limit);
        let join_handle = Self::spawn_handler(
            db_pool,
            receiver,
            task_heap.clone()
        ).await;

        Self {
            task_heap: task_heap,
            queue_limit: queue_limit as u64,
            sender,
            join_handle
        }
    }

    async fn spawn_handler(
        db_pool: Arc<db::Pool>,
        mut receiver: Receiver<OrderHash>,
        task_heap: Arc<RwLock<HashMap<OrderHash, Task>>>
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(order_hash) = receiver.recv().await {
                let task = task_heap.read().await.get(&order_hash).unwrap().clone();
                let mut stream = task_stream(task.clone());

                while let Some(task) = stream.next().await {
                    if matches!(task.status, TaskStatus::Completed | TaskStatus::Error) {
                        if task_heap.write().await.remove(&order_hash).is_some() {
                            task_heap.write().await.values_mut()
                                .for_each(|t| t.queue_num -= 1);
                        }

                        let _ = db::insert_task(&db_pool, &task).await;
                    } else {
                        task_heap.write().await.insert(order_hash.clone(), task);
                    }
                }
            }
        })
    }

    pub async fn registering_task(&self, mut task: Task) -> Result<OrderHash, ApiError> {
        let task_count = self.task_heap.read().await.len() as u64;
        if task_count >= self.queue_limit {
            return Err(
                ApiError::HandlerQueueOverflow(self.queue_limit)
            );
        }
        task.queue_num = task_count;
        let order_hash = task.order_hash.clone();
        if !self.task_heap.read().await.contains_key(&order_hash) {
            self.task_heap.write().await.insert(order_hash.clone(), task);

            if self.sender.send(order_hash.clone()).await.is_err() {
                return Err(
                    ApiError::TaskSendFailure
                );
            }
            return Ok(order_hash);
        }

        Err (ApiError::TaskAlreadyExists(order_hash))
    }

    pub async fn task_count_by_token_id(&self, token_id: &str) -> usize {
        self.task_heap.read().await.values()
            .filter(|t| t.order.token_id == token_id).count()
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
    pub handlers_count: usize,
    pub handler_queue_limit: usize,
}

impl AppState {
    pub async fn new(
        db_pool: Arc<db::Pool>,
        handlers_count: usize,
        handler_queue_limit: usize
    ) -> Self {
        let mut task_handlers = Vec::with_capacity(handlers_count);
        for _ in 0..handlers_count {
            task_handlers.push(
                TaskHandler::run(
                    db_pool.clone(),
                    handler_queue_limit
                ).await
            );
        }
        Self {
            db_pool,
            task_handlers,
            handlers_count,
            handler_queue_limit
        }
    }

    pub async fn insert_order(&self, order: Order) -> Result<OrderHash, ApiError> {
        let task = Task::from_order(order);
        let handler_index = self.select_handler_index().await;
        self.task_handlers.get(handler_index).unwrap()
            .registering_task(task).await
    }

    pub async fn task_count_by_token_id(&self, token_id: &str) -> usize {
        let mut task_count = 0_usize;
        for handler in self.task_handlers.iter() {
            task_count += handler.task_count_by_token_id(token_id).await;
        }
        task_count
    }

    pub async fn get_task_state(&self, order_hash: &String) -> Result<Task, ApiError> {
        for th in self.task_handlers.iter() {
            if th.contains_task(order_hash).await {
                if let Some(task) = th.get_task(order_hash).await {
                    return Ok ( task );
                }
                return Err(ApiError::Unknown);
            }
        }
        db::cutout_task(&self.db_pool, order_hash).await
            .map_err(|_| ApiError::TaskNotFound)
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
