#![allow(warnings)]
use once_cell::sync::OnceCell;
use serde::{Deserialize, Deserializer};

use browser_bridge::{
    BrowserSessionConfig,
    BrowserTimings
};
use super::utils::read_file;


static CFG: OnceCell<Config> = OnceCell::new();

pub fn get() -> &'static Config {
    CFG.get_or_init(|| {
        let config = match read_file("Config.toml") {
            Ok(cfg) => toml::from_str::<Config>(&cfg)
                .unwrap_or_else(|e| {
                    eprint!("Fail to parse config!\n {e}");
                    Config::default()
                }
            ),
            Err(e) => {
                eprint!("Fail to read config!\n {e}");
                Config::default()
            }
        };
        for (key, value) in config.env.iter() {
            std::env::set_var(key, value);
        }

        config
    })
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Config {
    env: Vec<(String, String)>,
    pub server: Server,
    pub api: Api,
    pub browser: Browser,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Server {
    pub host: String,
    pub port: u64
}

impl Server {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Api {
    pub version: String,
    pub assets_path: String,
    pub db_path: String,
    pub db_max_conn: u32,
    pub handlers_count: usize,
    pub handler_queue_limit: usize
}

#[derive(Deserialize, Debug, Clone)]
struct Browser {
    pub executable: Option<String>,
    pub user_data_dir: Option<String>,
    pub args: Vec<String>,
    pub headless_mod: u8,
    pub sandbox: bool,
    pub extensions: Vec<String>,
    pub incognito: bool,
    pub available_ports: Vec<u16>,
    pub launch_timeout: u64,
    pub request_timeout: u64,
    pub cache_enabled: bool,
    pub timings: DeBrowserTimings,
    pub page_param: BrowserPageParam
}

#[derive(Deserialize, Debug, Clone)]
struct DeBrowserTimings {
    pub launch_sleep: u64,
    pub set_proxy_sleep: u64,
    pub action_sleep: u64,
    pub wait_page_timeout: u64,
}

#[derive(Deserialize, Default, Debug, Clone)]
struct BrowserPageParam {
    pub stealth_mode: bool,
    pub rand_user_agent: bool,
    pub duration: BrowserPageParamDuration,
}

#[derive(Deserialize, Default, Debug, Clone)]
struct BrowserPageParamDuration {
    pub base_: u64,
    pub oz: u64,
    pub wb: u64,
    pub ym: u64,
    pub mm: u64,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 5500
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self {
            version: "0.1.0".into(),
            assets_path: "assets".into(),
            db_path: "sqlite:scraper_api.db".into(),
            db_max_conn: 2,
            handlers_count: 1,
            handler_queue_limit: 10
        }
    }
}

impl Default for Browser {
    fn default() -> Self {
        let default = BrowserSessionConfig::default();
        Self {
            executable: default.executable,
            user_data_dir: default.user_data_dir,
            args: default.args,
            headless_mod: 0,
            sandbox: default.sandbox,
            extensions: default.extensions,
            incognito: default.incognito,
            available_ports: vec![0],
            launch_timeout: default.launch_timeout,
            request_timeout: default.request_timeout,
            cache_enabled: default.cache_enabled,
            timings: DeBrowserTimings::default(),
            page_param: BrowserPageParam::default()
        }
    }
}

impl Default for DeBrowserTimings {
    fn default() -> Self {
        let default = BrowserTimings::default();
        Self {
            launch_sleep: default.launch_sleep,
            set_proxy_sleep: default.set_proxy_sleep,
            action_sleep: default.action_sleep,
            wait_page_timeout: default.wait_page_timeout,
        }
    }
}

pub const LOGO: &'static str = r##"
__________               __   _________
\______   \__ __  ______/  |_/   _____/ ________________  ______   ___________
 |       _/  |  \/  ___|   __\_____  \_/ ___\_  __ \__  \ \____ \_/ __ \_  __ \
 |    |   \  |  /\___ \ |  | /        \  \___|  | \// __ \|  |_> >  ___/|  | \/
 |____|_  /____//____  >|__|/_______  /\___  >__|  (____  /   __/ \___  >__|
        \/           \/             \/     \/           \/|__|        \/
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_config() {
        println!("{:#?}", get());
        assert_eq!(true, true);
    }
}
