use async_stream::stream;
use tokio_stream::Stream;
use super::models::{
    Task, 
    TaskProgress, 
    TaskStatus
};


pub fn task_stream(mut task: Task) -> impl Stream<Item = Task> {
    let s = stream! {
        let mut step = 0;
        task.status = TaskStatus::Processing;
        while !matches!(task.status, TaskStatus::Completed | TaskStatus::Error) {
            if step >= 20 {
                task.status = TaskStatus::Completed;
            }
            task.progress = Some(TaskProgress::new(step, 20));
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            yield task.clone();
            step += 1;
        }
    };
    Box::pin(s)
}

