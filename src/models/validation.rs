#![allow(warnings)]
use once_cell::sync::{
	OnceCell,
	Lazy
};
use std::{
	collections::HashMap,
	net::IpAddr
};
use thiserror::Error;
use regex::Regex;
use reqwest::Url;

use super::{
	scraper::{
		AVAILABLE_MARKETS,
		Symbol
	},
	api::Order,
};


static PROXY_REGEX: OnceCell<Regex> = OnceCell::new();

fn get_proxy_regex() -> &'static Regex {
    PROXY_REGEX.get_or_init(|| {
		Regex::new(
			r"^(?P<username>[^:@]+):(?P<password>[^:@]+)@(?P<host>[^:@]+):(?P<port>\d+)$"
		).unwrap()
	})
}

pub struct ProxyParam {
    pub username: Option<String>,
    pub password: Option<String>,
    pub host: String,
    pub port: u16
}

impl ProxyParam {
	pub fn from_str(s: &str) -> Result<Self, ValidationError> {
		let caps = get_proxy_regex()
			.captures(s)
			.ok_or(InvalidProxy::InvalidProxyFormat(s.into()))?;

		let host = caps.name("host")
			.ok_or(InvalidProxy::InvalidProxyFormat(s.into()))?
			.as_str()
			.to_string()
			.parse::<IpAddr>()
			.map_err(|_| InvalidProxy::InvalidProxyIp(s.into()))?
			.to_string();

		let port = caps.name("port")
			.and_then(|m| m.as_str().parse::<u16>().ok())
			.ok_or(InvalidProxy::InvalidProxyPort(s.into()))?;

		let username = caps.name("username")
			.map(|m| m.as_str().to_string());

		let password = caps.name("password")
			.map(|m| m.as_str().to_string());

		Ok (
			Self {
				username,
				password,
				host,
				port
			}
		)
	}

	pub fn addrs(&self) -> String {
		format!("{}:{}", self.host, self.port)
	}
}

pub enum ValidationError {
	Proxy(InvalidProxy),
	Product(InvalidProduct)
}

impl From<InvalidProxy> for ValidationError {
	fn from(value: InvalidProxy) -> Self {
		Self::Proxy(value)
	}
}

impl From<InvalidProduct> for ValidationError {
	fn from(value: InvalidProduct) -> Self {
		Self::Product(value)
	}
}

#[derive(Debug, Error)]
pub enum InvalidProxy {
    #[error("format: '{0}'.")]
    InvalidProxyFormat(String),
    #[error("IP address: '{0}'.")]
    InvalidProxyIp(String),
    #[error("port number: '{0}'.")]
    InvalidProxyPort(String),
	#[error("symbol: '{0}'.")]
    InvalidProxySymbol(String),
}

#[derive(Debug, Error)]
pub enum InvalidProduct {
	#[error("format: '{0}'")]
    InvalidProductFormat(String),
    #[error("ID: '{0}'")]
    InvalidProductId(String),
    #[error("symbol: '{0}'")]
    InvalidProductSymbol(String),
	#[error("url: '{0}'")]
    InvalidProductUrl(String),
	#[error("symbol '{0}' is temporarily unavailable.")]
    SymbolUnavailable(String),
}

pub trait Validation {
	type Error: From<ValidationError>;

	fn validation(&mut self) -> Result<(), Self::Error>;
}

impl Validation for Order {
	type Error = ValidationError;

	fn validation(&mut self) -> Result<(), Self::Error> {
		for proxy in self.proxy_pool.iter() {
            proxy_str_validation(proxy)
				.map_err(|e| ValidationError::Proxy(e))?;
        }
		// for (symbol, proxy_pool) in self.proxy_map.iter() {
		// 	Symbol::from_string(symbol)
		// 		.map_err(|_| ValidationError::Proxy(
		// 			InvalidProxy::InvalidProxySymbol(symbol.into())
		// 		))?;
		// 	for proxy in proxy_pool {
		// 		proxy_str_validation(proxy)
		// 			.map_err(|e| ValidationError::Proxy(e))?;
		// 	}
        // }
		for product in self.products.iter_mut() {
			*product = product_str_validation(product)
				.map_err(|e| ValidationError::Product(e))?;
		}

		Ok(())
	}
}

fn proxy_str_validation(s: &str) -> Result<(), InvalidProxy> {
	let caps = get_proxy_regex()
		.captures(s)
		.ok_or(InvalidProxy::InvalidProxyFormat(s.into()))?;

	caps.name("host")
		.ok_or(InvalidProxy::InvalidProxyFormat(s.into()))?
		.as_str()
		.to_string()
		.parse::<IpAddr>()
		.map_err(|_| InvalidProxy::InvalidProxyIp(s.into()))?;

	caps.name("port")
		.and_then(|m| m.as_str().parse::<u16>().ok())
		.ok_or(InvalidProxy::InvalidProxyPort(s.into()))?;

	Ok(())
}

fn product_str_validation(s: &str) -> Result<String, InvalidProduct> {
	let (symbol, id) = if let Ok(url) = Url::parse(s) {
		let get_segment = |i: usize|
			-> Result<String, InvalidProduct>
		{
			let segments = url.path_segments()
				.ok_or(InvalidProduct::InvalidProductUrl(s.into()))?
				.collect::<Vec<_>>();
			segments.get(i)
				.map(|v| v.to_string())
				.ok_or(InvalidProduct::InvalidProductUrl(s.into()))
		};
		if s.starts_with("https://www.ozon.ru/product/") {
			let mut segment = get_segment(1)?;
			if let Some((_, id_)) = segment.rsplit_once('-') {
				segment = id_.into();
			}
			("oz", segment)
		} else if s.starts_with("https://www.wildberries.ru/catalog/") {
			let segment = get_segment(1)?;
			("wb", segment)
		} else if s.starts_with("https://market.yandex.ru/product") {
			let mut segment = get_segment(1)?;
			let params = url.query_pairs().collect::<HashMap<_, _>>();
			segment = format!(
				"{}-{}-{}",
				segment,
				params.get("sku")
					.ok_or(InvalidProduct::InvalidProductUrl(s.into()))?,
				params.get("uniqueId")
					.ok_or(InvalidProduct::InvalidProductUrl(s.into()))?,
			);
			("ym", segment)
		} else if s.starts_with("https://megamarket.ru/catalog/details/") {
			let mut segment = get_segment(2)?;
			if let Some((_, id_)) = segment.rsplit_once('-') {
				segment = id_.into();
			}
			("mm", segment)
		} else {
			return Err(InvalidProduct::InvalidProductUrl(s.into()));
		}
	} else {
		let parts = s.trim().split_once('/')
			.ok_or(InvalidProduct::InvalidProductFormat(s.into()))?;
		(parts.0, parts.1.into())
	};
	let symbol = Symbol::from_string(symbol)
		.map_err(|_| InvalidProduct::InvalidProductSymbol(symbol.into()))?;
	if !AVAILABLE_MARKETS.contains(&symbol.as_str().into()) {
		return Err(InvalidProduct::SymbolUnavailable(symbol.as_str().into()));
	}
	match symbol {
		Symbol::OZ | Symbol::WB | Symbol::MM => {
			id.parse::<u64>()
				.map_err(|_| InvalidProduct::InvalidProductId(id.clone()))?;
		},

		Symbol::YM => {
			let parts = id.splitn(3, '-');
			let conditions = parts.into_iter()
				.map(|v| v.parse::<u64>().is_err()).collect::<Vec<_>>();
			if conditions.len() != 3 {
				return Err(InvalidProduct::InvalidProductId(id.clone()));
			}
			if conditions.iter().any(|v| *v) {
				return Err(InvalidProduct::InvalidProductId(id.clone()));
			}
		}
	}
	let valid = format!("{}/{}", id, symbol.as_str());
	if valid.len() < 7 {
		return Err(InvalidProduct::InvalidProductId(id));
	}

	Ok(valid)
}

#[cfg(test)]
mod tests {
    //Запуск ssh сервера
    //ssh -R 5500:localhost:5500 -N -f -o "ServerAliveInterval 60" -o "ServerAliveCountMax 3" server
    use super::*;
    use tokio::time::sleep;
    use std::{collections::HashMap, net::SocketAddr, time::Duration};

	#[test]
    fn test_parse_url() {
        let url = Url::parse("https://megamarket.ru/catalog/details/motornoe-maslo-lukoil-genesis-armortech-5w40-4l-100026336791/").unwrap();
		let segments = url.path_segments().unwrap().collect::<Vec<_>>();
		let params = url.query_pairs().collect::<HashMap<_, _>>();
		println!("{:?}", segments);
		println!("{:?}", params);
    }
}
