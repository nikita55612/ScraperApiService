use once_cell::sync::OnceCell;
use browser_bridge::{
	random_user_agent, BrowserError, BrowserSession, BrowserSessionConfig, PageParam
};
use serde_json::map;
use tokio::sync::Mutex;

use crate::models::{api::OrderCookiesParam, scraper::{Product, ProductData, Symbol}};

use super::{super::config as cfg, extractor::product::extract_data};
use super::super::utils::is_port_open;

// ЭТО БАЗА

struct BrowserState {
	port: u16,
	user_data_dir: String,
	running: bool
}

impl BrowserState {
	fn to_config(&self) -> BrowserSessionConfig {
		cfg::get().browser
			.session_config(
				self.port,
				self.user_data_dir.clone()
			)
	}
}

struct BrowserStates(Mutex<Vec<BrowserState>>);

impl BrowserStates {
	async fn launch_browser_session(&self) -> Result<Browser, BrowserError> {
		let mut lock_states = self.0.lock().await;
		let (config, port) = if let Some(state) = lock_states
			.iter_mut()
			.find(|state| !state.running)
		{
			state.running = true;
			(
				state.to_config(),
				state.port
			)
		} else {
			return Err(BrowserError::Unknown);
		};
		let session = BrowserSession::launch(config).await?;

		Ok (
			Browser {port, session}
		)
	}

	async fn stop_running(&self, port: u16) {
		let mut lock_states = self.0.lock().await;
		lock_states.iter_mut()
			.find(|v| v.port == port)
			.map(|v| v.running = false);
	}
}

static BROWSER_STATES: OnceCell<BrowserStates> = OnceCell::new();

// Как  то настроить чтобы порты брались не из available_ports а автоматически подбирались доступные
pub fn get_browser_states() -> &'static BrowserStates {
    BROWSER_STATES.get_or_init(|| {
		let mut states = Vec::with_capacity(
			cfg::get().api.handlers_count
		);
		let port = (cfg::get().server.port + 1) as u16;
		while states.len() < states.capacity() {
			if is_port_open(port) {
				states.push(
					BrowserState {
						port,
						// Инициализация временных директорий
						user_data_dir: String::new(),
						running: false
					}
				);
			}
		}

		BrowserStates(
			Mutex::new(states)
		)
	})
}

enum ReqwestError {
	Browser(String),
}

impl From<BrowserError> for ReqwestError {
	fn from(value: BrowserError) -> Self {
		Self::Browser(value.to_string())
	}
}

struct Browser {
	port: u16,
	session: BrowserSession
}

struct Client {
	browser: Browser,
	reqwest_client: reqwest::Client,
	// Передача конфигурации запросов (список прокси, куки итд)
	reqwest_config: bool,
	cookies: Vec<OrderCookiesParam>,
	// Количество запросов за сессию
	reqwest_count: usize
}

impl Client {
	async fn new(cookies: Vec<OrderCookiesParam>) -> Result<Self, BrowserError> {
		let browser_states = get_browser_states();
		let browser = browser_states
			.launch_browser_session()
			.await?;

		let reqwest_client = reqwest::Client::new();

		Ok(
			Self {
				browser: browser,
				reqwest_client,
				cookies,
				reqwest_config: true,
				reqwest_count: 0
			}
		)
	}

	async fn reqwest_product_data(&mut self, product: Product) -> Result<Option<ProductData>, ReqwestError> {
		let content = match product.symbol {
			Symbol::OZ | Symbol::YM | Symbol::MM => {
				let page_parsm = PageParam {
					..Default::default()
				};
				let page = self.browser
					.session
					.open_with_param(
						&product.get_parse_url(),
						&page_parsm
					)
					.await?;
				let content = page.content()
					.await
					.map_err(|e| BrowserError::from(e))?;
				let _ = page.close().await;

				content
			},
			Symbol::WB => String::new(),
		};
		let product_data = extract_data(
			product.symbol,
			&content
		);
		self.reqwest_count += 1;

		Ok (
			product_data
		)
	}

	async fn close(&mut self) {
		self.browser.session.close().await;
		let browser_states = get_browser_states();
		browser_states.stop_running(self.browser.port).await;
		self.reqwest_count = 0;
	}
}


// Состояние клиента
// Однократная инициализация
// RwLock для изменчивости в многопоточной среде
// Хранит в себе массив сессий браузера
// Параметры массива: user_data_dir, port
// Имеет один изменчивый параметр - открыт/закрыт


// Клиент
// Инициализация браузера и reqwest::Client

// pub fn reqwest_product_data(product: &Product, bs: Option<&BrowserSession>) -> ProductData {
// 	match product.symbol {
// 		Symbol::OZ | Symbol::YM | Symbol::MM => (),
// 		Symbol::WB => (),
// 	}
// 	ProductData::rand()
// }

// async fn reqwest_get(url: &String) -> Option<String> {
// 	let client_builder = reqwest::Client::builder()
// 		.user_agent(random_user_agent())
// 		.timeout(Duration::from_millis(REQWEST_CONFIG.timeout_millis));

// 	let client;

// 	if REQWEST_CONFIG.proxy_list.len() > 0 && utils::rand_gen::<f64>() < REQWEST_CONFIG.proxy_percentage {
// 		let pp = ProxyParams::from_string(
// 			&REQWEST_CONFIG.proxy_list[utils::random_rng(0, REQWEST_CONFIG.proxy_list.len())]);
// 		let proxy = reqwest::Proxy::https(format!("http://{}", pp.addrs)).ok()?
// 			.basic_auth(&pp.username, &pp.password);
// 		client = client_builder.proxy(proxy).build().ok()?;
// 	} else {
// 		client = client_builder.build().ok()?;
// 	}
// 	let content = client.get(url)
// 		.send()
// 		.await.ok()?.text().await.ok()?;
// 	Some(content)
// }

// async fn reqwest_get_without_proxy(url: &String) -> Option<String> {
// 	let client = reqwest::Client::builder()
// 		.user_agent(
// 			utils::get_rand_user_agent()
// 		)
// 		.timeout(
// 			Duration::from_millis(REQWEST_CONFIG.timeout_millis)
// 		).build().ok()?;
// 	let content = client.get(url).send().await.ok()?.text().await.ok()?;

// 	Some(content)
// }
