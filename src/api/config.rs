use once_cell::sync::OnceCell;
use serde::Deserialize;
use super::utils::read_file;


static INIT: OnceCell<Config> = OnceCell::new();

pub fn get() -> &'static Config {
    INIT.get_or_init(|| {
        match read_file("Config.toml") {
            Ok(cfg) => toml::from_str::<Config>(&cfg)
                .unwrap_or_else(|_| {
                    eprint!("Fail to parse config!");
                    Config::default()
                }
            ),
            Err(_) => {
                eprint!("Fail to read config!");
                Config::default()
            }
        }
    })
}

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    pub server: Server,
    pub api: Api
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub addr: String,
    pub port: u64
}

#[derive(Deserialize, Debug)]
pub struct Api {
    pub version: String,
    pub order_limit_items: u64,
    pub db: String,
    pub db_max_conn: u32,
    pub task_handlers: u64,
    pub handler_queue_limit: usize
}

impl Default for Server {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0".into(),
            port: 5500
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self {
            version: "0.1.0".into(),
            order_limit_items: 250,
            db: "sqlite:scraper_api.db".into(),
            db_max_conn: 2,
            task_handlers: 1,
            handler_queue_limit: 10
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_config() {
        println!("{:#?}", get());
        assert_eq!(true, true);
    }
}