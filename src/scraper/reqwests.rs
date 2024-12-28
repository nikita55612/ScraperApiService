use super::browser;


pub fn get_reqwest_client() -> reqwest::Client {
	reqwest::Client::new()
}

pub fn get_browser_client() -> reqwest::Client {
	reqwest::Client::new()
}

pub fn reqwest_product() -> reqwest::Client {
	reqwest::Client::new()
}

async fn reqwest_get(url: &String) -> Option<String> {
	let client_builder = reqwest::Client::builder()
		.user_agent(utils::get_rand_user_agent())
		.timeout(Duration::from_millis(REQWEST_CONFIG.timeout_millis));

	let client;

	if REQWEST_CONFIG.proxy_list.len() > 0 && utils::rand_gen::<f64>() < REQWEST_CONFIG.proxy_percentage {
		let pp = ProxyParams::from_string(
			&REQWEST_CONFIG.proxy_list[utils::random_rng(0, REQWEST_CONFIG.proxy_list.len())]);
		let proxy = reqwest::Proxy::https(format!("http://{}", pp.addrs)).ok()?
			.basic_auth(&pp.username, &pp.password);
		client = client_builder.proxy(proxy).build().ok()?;
	} else {
		client = client_builder.build().ok()?;
	}
	let content = client.get(url).send().await.ok()?.text().await.ok()?;
	Some(content)
}

async fn reqwest_get_without_proxy(url: &String) -> Option<String> {
	let client = reqwest::Client::builder()
		.user_agent(
			utils::get_rand_user_agent()
		)
		.timeout(
			Duration::from_millis(REQWEST_CONFIG.timeout_millis)
		).build().ok()?;
	let content = client.get(url).send().await.ok()?.text().await.ok()?;

	Some(content)
}
