#![allow(warnings)]
use std::net::IpAddr;
use once_cell::sync::OnceCell;
use regex::Regex;
use thiserror::Error;

use super::{
	scraper::Symbol,
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
    #[error("Invalid proxy format: '{0}'")]
    InvalidProxyFormat(String),
    #[error("Invalid IP address: '{0}'")]
    InvalidProxyIp(String),
    #[error("Invalid port number: '{0}'")]
    InvalidProxyPort(String),
	#[error("Invalid proxy symbol: '{0}'")]
    InvalidProxySymbol(String),
}

#[derive(Debug, Error)]
pub enum InvalidProduct {
	#[error("Invalid product format: '{0}'")]
    InvalidProductFormat(String),
    #[error("Invalid product ID: '{0}'")]
    InvalidProductId(String),
    #[error("Invalid product symbol: '{0}'")]
    InvalidProductSymbol(String),
}

pub trait Validation {
	type Error: From<ValidationError>;

	fn validation(&self) -> Result<(), Self::Error>;
}

impl Validation for Order {
	type Error = ValidationError;

	fn validation(&self) -> Result<(), Self::Error> {
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

		for product in self.products.iter() {
			product_str_validation(product)
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

fn product_str_validation(s: &str) -> Result<(), InvalidProduct> {
	let (symbol, id) = s.trim().split_once('/')
		.ok_or(InvalidProduct::InvalidProductFormat(s.into()))?;

	let symbol = Symbol::from_string(symbol)
		.map_err(|_| InvalidProduct::InvalidProductSymbol(symbol.into()))?;

	match symbol {
		Symbol::OZ | Symbol::WB | Symbol::MM => id.parse::<u64>()
			.map_err(|_| InvalidProduct::InvalidProductId(id.into()))
			.map(|_| ()),

		Symbol::YM => {
			let parts = id.splitn(3, '-');
			let conditions = parts.into_iter()
				.map(|v| v.parse::<u64>().is_err()).collect::<Vec<_>>();
			if conditions.len() != 3 {
				return Err(InvalidProduct::InvalidProductId(id.into()));
			}
			if conditions.iter().any(|v| *v) {
				return Err(InvalidProduct::InvalidProductId(id.into()));
			}

			Ok(())
		}

		_ => Ok(()),
	}
}
