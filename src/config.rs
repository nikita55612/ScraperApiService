use super::utils::{is_port_open, mkdir_if_not_exists, read_file, remove_all_dirs, write_to_file};
use browser_bridge::{chromiumoxide::browser::HeadlessMode, BrowserSessionConfig, BrowserTimings};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::OnceLock};
use utoipa::ToSchema;

static CFG: OnceLock<Config> = OnceLock::new();

static BROWSER_SESSION_CFG: OnceLock<BrowserSessionConfig> = OnceLock::new();

pub fn get() -> &'static Config {
    CFG.get_or_init(|| match read_file("Config.toml") {
        Ok(cfg) => toml::from_str::<Config>(&cfg).expect("Deserialize config error"),
        Err(e) => {
            log::error!("Fail to read config!\n {e}");
            if !Path::new("Config.toml").exists() {
                log::error!("Config.toml is not exists!\n {e}");
                let config = toml::to_string_pretty::<Config>(&Config::default())
                    .expect("Serialize config error");
                write_to_file("Config.toml", config.as_bytes())
                    .expect("Write to Config.toml error");
                log::warn!("Config.toml is default!");
                log::info!("Edit the config and restart app...")
            }
            panic!();
        }
    })
}

pub fn init() {
    let config = get();
    if let Some(pub_env) = &config.pub_env {
        for (key, value) in pub_env.iter() {
            log::info!("[SET_PUB_ENV_VAR] {}={}", key, value);
            std::env::set_var(key, value);
        }
    }
    if let Some(env) = &config.env {
        for (key, value) in env.iter() {
            log::info!("[SET_ENV_VAR] {}={}", key, value);
            std::env::set_var(key, value);
        }
    }
    let _ = mkdir_if_not_exists(&config.browser.users_temp_data_dir);
    let _ = remove_all_dirs(&config.browser.users_temp_data_dir);
    if !is_port_open(config.server.port) {
        log::error!("Server port {} already in use", config.server.port);
        panic!();
    }
    if dotenv::dotenv().is_err() {
        log::warn!(".env file not found");
    }
    if let Err(_) = std::env::var("MASTER_TOKEN") {
        log::error!("Env var MASTER_TOKEN not defined");
        panic!();
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone, ToSchema)]
pub struct Config {
    #[serde(skip_serializing)]
    env: Option<Vec<(String, String)>>,
    #[schema(value_type = Option<Vec<Vec<String>>>)]
    pub pub_env: Option<Vec<(String, String)>>,
    pub server: Server,
    pub api: Api,
    pub browser: Browser,
    pub req_session: ReqSession,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

impl Server {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct Api {
    pub root_api_path: String,
    pub description_file_path: Option<String>,
    pub log_file_path: Option<String>,
    pub assets_path: String,
    pub db_path: String,
    pub db_max_conn: u32,
    pub handlers_count: usize,
    pub handler_queue_limit: usize,
    pub task_ws_sending_interval: u64,
    pub open_ws_limit: u32,
    pub test_token: TestToken,
    pub interrupt_check_step: u64,
    pub available_markets: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct Browser {
    pub executable: Option<String>,
    pub users_temp_data_dir: String,
    pub args: Vec<String>,
    pub headless_mod: u8,
    pub sandbox: bool,
    pub extensions: Vec<String>,
    pub incognito: bool,
    pub launch_timeout: u64,
    pub request_timeout: u64,
    pub cache_enabled: bool,
    pub timings: DeBrowserTimings,
    pub page_param: BrowserPageParam,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct ReqSession {
    pub set_proxy_interval: u64,
    pub close_tabs_interval: u64,
    pub launch_sleep: u64,
    pub timings: ReqTimings,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct ReqTimings {
    pub timeout: u64,
    pub conn_timeout: u64,
    pub read_timeout: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct DeBrowserTimings {
    pub launch_sleep: u64,
    pub set_proxy_sleep: u64,
    pub action_sleep: u64,
    pub page_goto_timeout: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct TestToken {
    pub ttl: u64,
    pub op_limit: u64,
    pub tc_limit: u64,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone, ToSchema)]
pub struct BrowserPageParam {
    pub rand_user_agent: bool,
    pub wait_for_el_timeout: u64,
    #[serde(default)]
    pub symbol: HashMap<String, SymbolPageParam>,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone, ToSchema)]
pub struct SymbolPageParam {
    pub wait_for_el: Option<String>,
    pub wait_for_el_until: Option<(String, String)>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 5500,
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self {
            root_api_path: "/api/v1".into(),
            description_file_path: None,
            log_file_path: None,
            assets_path: "assets".into(),
            db_path: "sqlite:scraper_api.db".into(),
            db_max_conn: 2,
            handlers_count: 1,
            handler_queue_limit: 10,
            task_ws_sending_interval: 1000,
            open_ws_limit: 20,
            test_token: TestToken::default(),
            interrupt_check_step: 60,
            available_markets: vec!["oz".into(), "wb".into(), "ym".into(), "mm".into()],
        }
    }
}

impl Default for Browser {
    fn default() -> Self {
        let default = BrowserSessionConfig::default();
        Self {
            executable: default.executable,
            users_temp_data_dir: "./users_temp_data".into(),
            args: default.args,
            headless_mod: 0,
            sandbox: default.sandbox,
            extensions: default.extensions,
            incognito: default.incognito,
            launch_timeout: default.launch_timeout,
            request_timeout: default.request_timeout,
            cache_enabled: default.cache_enabled,
            timings: DeBrowserTimings::default(),
            page_param: BrowserPageParam::default(),
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
            page_goto_timeout: default.page_goto_timeout,
        }
    }
}

impl Default for ReqSession {
    fn default() -> Self {
        Self {
            set_proxy_interval: 14,
            close_tabs_interval: 40,
            launch_sleep: 700,
            timings: ReqTimings::default(),
        }
    }
}

impl Default for ReqTimings {
    fn default() -> Self {
        Self {
            timeout: 700,
            conn_timeout: 500,
            read_timeout: 500,
        }
    }
}

impl Default for TestToken {
    fn default() -> Self {
        Self {
            ttl: 86400,
            tc_limit: 40,
            op_limit: 1,
        }
    }
}

impl From<DeBrowserTimings> for BrowserTimings {
    fn from(value: DeBrowserTimings) -> Self {
        Self {
            launch_sleep: value.launch_sleep,
            set_proxy_sleep: value.set_proxy_sleep,
            action_sleep: value.action_sleep,
            page_goto_timeout: value.page_goto_timeout,
        }
    }
}

impl Browser {
    pub fn session_config(&self, port: u16, user_data_dir: String) -> BrowserSessionConfig {
        let mut session_config = BROWSER_SESSION_CFG
            .get_or_init(|| BrowserSessionConfig {
                executable: self.executable.clone(),
                args: self.args.clone(),
                headless: {
                    match self.headless_mod {
                        0 => HeadlessMode::False,
                        1 => HeadlessMode::True,
                        _ => HeadlessMode::New,
                    }
                },
                sandbox: self.sandbox,
                extensions: self.extensions.clone(),
                incognito: self.incognito,
                launch_timeout: self.launch_timeout,
                request_timeout: self.request_timeout,
                cache_enabled: self.cache_enabled,
                timings: self.timings.clone().into(),
                ..Default::default()
            })
            .clone();
        session_config.port = port;
        session_config.user_data_dir = Some(user_data_dir);

        session_config
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
        init();
        assert_eq!(true, true);
    }
}
