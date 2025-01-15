use once_cell::sync::{Lazy, OnceCell};
use std::{
	collections::HashMap,
	sync::Arc,
	time::Duration
};
use browser_bridge::{
	chromiumoxide::{
		Page,
		cdp::browser_protocol::network::{
			CookieParam as BrowserCookieParam,
			CookieSameSite as BrowserCookieSameSite
		}
	},
	random_user_agent,
	BrowserError,
	BrowserSession,
	BrowserSessionConfig,
	PageParam
};
use tokio::{
	time::sleep,
	sync::Mutex
};
use reqwest::cookie::Jar;

use super::{
	error::ReqSessionError,
	extractor::product::extract_data,
	super::{
		config::{
			self as cfg,
			ReqSession as ReqSessionConfig
		},
		utils::is_port_open,
		models::{
			api::OrderCookieParam,
			scraper::{Product, ProductData, Symbol},
			validation::ProxyParam
		}
	},
};

#[derive(Clone, Debug)]
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

#[derive(Debug)]
struct BrowserStates(Mutex<Vec<BrowserState>>);

impl BrowserStates {
	async fn launch_browser_session(&self) -> Result<Browser, BrowserError> {
		let mut lock_states = self.0.lock().await;
		let (config, port) = if let Some(
			state
		) = lock_states
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

fn get_browser_states() -> &'static BrowserStates {
    BROWSER_STATES.get_or_init(|| {
		let mut states = Vec::with_capacity(
			cfg::get().api.handlers_count + 2
		);
		let mut port = (cfg::get().server.port + 1) as u16;
		while states.len() < states.capacity() {
			if is_port_open(port) {
				let temp_dir = tempfile::tempdir_in(
					&cfg::get().browser.users_temp_data_dir
				).expect("Create temp dir error");
				states.push(
					BrowserState {
						port,
						user_data_dir: temp_dir.path()
							.to_str()
							.map(String::from)
							.expect("Temp dir to string error"),
						running: false
					}
				);
			}
			port += 1;
		}

		BrowserStates(
			Mutex::new(states)
		)
	})
}

static DEFAULT_PAGE_PARAM: Lazy<PageParam<'static>> = Lazy::new(|| PageParam::default());
static PRODUCT_PAGE_PARAMS: OnceCell<HashMap<String, PageParam>> = OnceCell::new();

pub fn get_product_page_param(symbol: &str) -> &'static PageParam {
    PRODUCT_PAGE_PARAMS.get_or_init(|| {
		let cfg_page_param = &cfg::get()
			.browser
			.page_param;
		let wait_el_timeout = cfg_page_param.wait_for_element_timeout;
		let wait_for_element = |s: &str| {
			cfg_page_param
				.symbol
				.get(s)
				.and_then(|v|
					v.wait_for_product_element
						.as_ref()
						.map(|v| (v.as_str(), wait_el_timeout))
				)
		};
		HashMap::from([
				(
					Symbol::OZ.as_str().into(),
					PageParam {
						wait_for_element: wait_for_element(Symbol::OZ.as_str()),
						..Default::default()
					}
				),
				(
					Symbol::YM.as_str().into(),
					PageParam {
						wait_for_element: wait_for_element(Symbol::YM.as_str()),
						..Default::default()
					}
				),
				(
					Symbol::MM.as_str().into(),
					PageParam {
						wait_for_element: wait_for_element(Symbol::MM.as_str()),
						..Default::default()
					}
				)
		])
	}).get(symbol)
		.unwrap_or(&*DEFAULT_PAGE_PARAM)
}

struct Browser {
	port: u16,
	session: BrowserSession
}

#[allow(dead_code)]
pub enum ReqMethod {
	Combined,
	Browser,
	Reqwest,
}

pub struct ReqSession {
	browser: Option<Browser>,
	req_client: Option<reqwest::Client>,
	proxy_pool: Vec<String>,
	set_proxy_interval: u8,
	close_tabs_interval: u16,
	req_count: usize
}

impl ReqSession {
	pub async fn new(
		config: &ReqSessionConfig,
		method: ReqMethod,
		cookies: &Vec<OrderCookieParam>,
		proxy_pool: Vec<String>
	) -> Result<Self, ReqSessionError> {
		let req_client = if matches!(
			method, ReqMethod::Reqwest | ReqMethod::Combined
		) {
			let jar = Arc::new(Jar::default());
			for cookie in cookies {
				let url = cookie.url.as_str()
					.parse::<reqwest::Url>();
				if url.is_err() { continue; };
				let cookie_str = format!(
					"name={}; value={}; domain={}; path={}; secure={}; httpOnly={}; sameSite={}",
					cookie.name.as_str(),
					cookie.value.as_str(),
					cookie.domain.as_ref().unwrap_or(&"".into()),
					cookie.path.as_ref().unwrap_or(&"".into()),
					cookie.secure.as_ref().map(|v| if *v {"true"} else {"false"}).unwrap_or(""),
					cookie.http_only.as_ref().map(|v| if *v {"true"} else {"false"}).unwrap_or(""),
					cookie.same_site.as_ref().unwrap_or(&"".into()),
				);
				jar.add_cookie_str(
					cookie_str.as_str(),
					&url.unwrap()
				);
			}

			let req_timings = &config.timings;
			let mut req_client_builder = reqwest::Client::builder()
				.user_agent(random_user_agent())
				.cookie_provider(jar)
				.timeout(
					Duration::from_millis(req_timings.timeout)
				)
				.connect_timeout(
					Duration::from_millis(req_timings.conn_timeout)
				)
				.read_timeout(
					Duration::from_millis(req_timings.read_timeout)
				);

			if !proxy_pool.is_empty() {
				let proxy_str = proxy_pool.get(0).unwrap();
				if let Ok(proxy_param) = ProxyParam::from_str(proxy_str) {
					let mut proxy = reqwest::Proxy::https(
						format!("http://{}", proxy_param.addrs())
						)
						.map_err(|_| ReqSessionError::BuildReqClient)?;
					if let (
						Some(username),
						Some(password)
					) = (proxy_param.username, proxy_param.password) {
						proxy = proxy.basic_auth(&username, &password);
					}
					req_client_builder = req_client_builder
						.proxy(proxy);
				}
			}

			let req_client = req_client_builder
				.build()
				.map_err(|_| {ReqSessionError::BuildReqClient})?;

			Some(req_client)
		} else {
			None
		};

		let browser = if matches!(
			method, ReqMethod::Browser | ReqMethod::Combined
		) {
			let browser_states = get_browser_states();
			let browser = browser_states
				.launch_browser_session()
				.await?;

			let browser_cookies = cookies.into_iter()
				.map(|cp| {
					BrowserCookieParam {
						name: cp.name.clone(),
						value: cp.value.clone(),
						url: Some(cp.url.clone()),
						domain: cp.domain.clone(),
						path: cp.path.clone(),
						secure: cp.secure.clone(),
						http_only: cp.http_only.clone(),
						same_site: {
							match cp.same_site.as_ref().map(|v| v.as_str()) {
								Some("lax") => Some(BrowserCookieSameSite::Lax),
								None => None,
								_ => Some(BrowserCookieSameSite::None)
							}
						},
						..BrowserCookieParam::builder().build().unwrap()
					}
				})
				.collect::<Vec<_>>();

			let _ = browser.session.clear_data().await;
			let _ = browser.session.reset_proxy().await;
			let _ = browser.session.browser.clear_cookies().await;
			let _ = browser.session.browser.set_cookies(browser_cookies).await;
			if !proxy_pool.is_empty() {
				let _ = browser.session.set_proxy(&proxy_pool[0]).await;
			}

			Some(browser)
		} else {
			None
		};

		sleep(
			Duration::from_millis(
				config.launch_sleep
			)
		).await;

		Ok(
			Self {
				browser,
				req_client,
				proxy_pool,
				set_proxy_interval: config.set_proxy_interval as u8,
				close_tabs_interval: config.close_tabs_interval as u16,
				req_count: 0
			}
		)
	}

	pub async fn req_product_data(
		&mut self,
		product: &Product
	) -> Result<Option<ProductData>, ReqSessionError> {
		if (self.req_count + 1) % self.close_tabs_interval as usize == 0 {
			let _ = self.browser_close_tabs().await;
		}
		let url = product.get_parse_url();
		let content = match product.symbol {
			Symbol::OZ | Symbol::YM | Symbol::MM => {
				let mut page_parsm = get_product_page_param(
					product.symbol.as_str()
				).clone();
				if self.proxy_pool.len() > 1 {
					if (self.req_count + 1) % self.set_proxy_interval as usize == 0 {
						let index = (
							self.req_count / self.set_proxy_interval as usize
							) % self.proxy_pool.len();
						page_parsm.proxy = Some(&self.proxy_pool[index]);
					}
				}
				self.browser_get_content(&url, &page_parsm).await?
			},
			Symbol::WB => {
				self.reqwest_get_content(&url).await?
			}
		};
		let product_data = extract_data(
			product,
			&content
		);
		self.req_count += 1;

		Ok (product_data)
	}

	pub async fn browser_get_content(
		&self,
		url: &str,
		page_parsm: &PageParam<'_>
	) -> Result<String, ReqSessionError> {

		let page = self.browser_open_page(
			url, page_parsm
		).await?;
		let content = page.content()
			.await
			.map_err(|e| BrowserError::from(e))?;
		let _ = page.close().await;

		Ok (content)
	}

	pub async fn browser_open_page(
		&self,
		url: &str,
		page_parsm: &PageParam<'_>
	) -> Result<Page, ReqSessionError> {
		self.browser
			.as_ref()
			.ok_or(ReqSessionError::NotAvailableReqMethod)?
			.session
			.open_with_param(
				url,
				page_parsm
			)
			.await
			.map_err(|e| e.into())
	}

	pub async fn browser_close_tabs(&self) -> Result<(), ReqSessionError> {
		self.browser
			.as_ref()
			.ok_or(ReqSessionError::NotAvailableReqMethod)?
			.session
			.close_tabs()
			.await
			.map_err(|e| e.into())
	}

	pub async fn reqwest_get_content(&self, url: &str) -> Result<String, ReqSessionError> {
		self.req_client
			.as_ref()
			.ok_or(ReqSessionError::NotAvailableReqMethod)?
			.get(url)
			.send()
			.await
			.map_err(|_| ReqSessionError::RequestSending)?
			.text()
			.await
			.map_err(|_| ReqSessionError::ExtractResponseContent)
	}

	pub async fn close(&mut self) {
		if let Some(browser) = &mut self.browser {
			browser.session.close().await;
			let browser_states = get_browser_states();
			browser_states.stop_running(browser.port).await;
		}
		self.req_count = 0;
	}
}


#[cfg(test)]
mod tests {
    use super::*;
    // use tokio::time::sleep;
    // use std::time::Duration;


    #[tokio::test]
    async fn test_req_session() {
		cfg::init();
		println!("run test_req_session...");
		println!("{:?}", get_browser_states());
		//sleep(Duration::from_millis(10000)).await;
        let mut rs = ReqSession::new(
				&cfg::get().req_session,
				ReqMethod::Combined,
				&vec![],
				vec![]
			)
			.await
			.unwrap();
		println!("run ReqSession...");
		let str_products = vec![
			"ym/357396943-103478532298-85861607",
			"ym/44497758-102626213158-757083",
			"ym/273181503-103028807462-916755",
			"ym/1915673993-102282726841-62878861",
			"oz/142724424",
			"oz/32549314",
			"oz/1680678914",
			"oz/1628554693",
			"mm/100070722113",
			"mm/100070722080",
			"mm/100000579167",
			"mm/100065768898",
			"wb/248939630",
			"wb/177900370",
			"wb/259666228",
			"wb/27090074",
		];
		for str_product in str_products {
			println!("{str_product}");

			let ptroduct = Product::from_string_without_valid(
				str_product
			);
			let product_data = rs.req_product_data(
				&ptroduct
				)
				.await;

			println!("{:?}", get_browser_states());

			println!("{product_data:#?}");
		}

		rs.close().await;

		println!("{:?}", get_browser_states());
        assert_eq!(true, true);
    }

	#[tokio::test]
    async fn test_proxy_pool() {
		let proxy_pool: Vec<i32> = vec![1];
		let set_proxy_interval = 8;
		for req_count in 0..122 {
			if (req_count + 1) % set_proxy_interval == 0 {
				let index = (req_count / set_proxy_interval) % proxy_pool.len();
				println!("n[{}] index {} = {}", req_count, index, proxy_pool[index])
			}
		}
	}
}
