use once_cell::sync::Lazy;
use tokio::sync::OnceCell;
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    sync::mpsc::{self, Sender},
};
use chrono::{
    DateTime,
    Utc
};

use crate::config as cfg;


static PRINT_LOGS: Lazy<bool> = Lazy::new(|| {
    match std::env::var("PRINT_LOGS") {
        Ok(v) => if v == "0" {false} else {true},
        Err(_) => true
    }
});

static LOGGER: OnceCell<Option<LoggerManagenr>> = OnceCell::const_new();

pub async fn init() {
	if let Some(logger) = LOGGER.get_or_init(|| async {
        if let Some(log_file) = &cfg::get().api.log_file_path {
            Some(LoggerManagenr::new(log_file).await)
        } else {
            None
        }
	}).await {
        logger.log(LogMessage {
            timestamp: Utc::now(),
            level: log::Level::Trace,
            from: "LAUNCH".into(),
            message: serde_json::to_string(&cfg::get()).unwrap_or_default(),
        }).await;
    }
}

pub async fn write(level: log::Level, from: &str, message: String) {
    if *PRINT_LOGS {
        log::log!(level, "[{}]: {}", from, message);
    }
    if let Some(Some(logger)) = LOGGER.get() {
        logger.log(LogMessage {
            timestamp: Utc::now(),
            level,
            from: from.into(),
            message,
        }).await;
    }
}

#[derive(Debug)]
struct LogMessage {
    timestamp: DateTime<Utc>,
    level: log::Level,
    from: String,
    message: String,
}

struct LoggerManagenr {
    sender: Sender<LogMessage>,
}

impl LoggerManagenr {
    async fn new(log_file: &str) -> Self {
        let (sender,
            mut receiver) = mpsc::channel::<LogMessage>(1024);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .await
            .expect("Failed to open log file");

        tokio::spawn(async move {
            let mut file = file;

            while let Some(log) = receiver.recv().await {
                let log_entry = format!(
                    "[{:?}] [{}] [{}]: {}\n",
                    log.timestamp,
                    log.level,
                    log.from,
                    log.message
                );

                if let Err(e) = file.write_all(log_entry.as_bytes()).await {
                    log::error!("Failed to write log: {}", e);
                }

                if let Err(e) = file.flush().await {
                    log::error!("Failed to flush log: {}", e);
                }
            }
        });

        Self { sender }
    }

    async fn log(&self, log_message: LogMessage) {
        if let Err(e) = self.sender.send(log_message).await {
            log::error!("Failed to send log message: {}", e);
        }
    }
}
