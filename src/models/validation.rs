#![allow(warnings)]
use regex::Regex;
use thiserror::Error;
use std::net::IpAddr;
use once_cell::sync::OnceCell;


static PROXY_REGEX: OnceCell<Regex> = OnceCell::new();

fn get_proxy_regex() -> &'static Regex {
    PROXY_REGEX.get_or_init(|| {
		Regex::new(
			r"^(?P<username>[^:@]+):(?P<password>[^:@]+)@(?P<host>[^:@]+):(?P<port>\d+)$"
		).unwrap()
	})
}

#[derive(Debug, Error)]
pub enum ValidProxyError {
    #[error("Invalid proxy format: '{0}'")]
    InvalidFormat(String),
    #[error("Invalid IP address: '{0}'")]
    InvalidIp(String),
    #[error("Invalid port number: '{0}'")]
    InvalidPort(String),
}

pub fn proxy_str_validation(s: &str) -> Result<(), ValidProxyError> {
	let caps = get_proxy_regex()
		.captures(s)
		.ok_or(ValidProxyError::InvalidFormat(s.into()))?;

	caps.name("host")
		.ok_or(ValidProxyError::InvalidFormat(s.into()))?
		.as_str()
		.to_string()
		.parse::<IpAddr>()
		.map_err(|_| ValidProxyError::InvalidIp(s.into()))?;

	caps.name("port")
		.and_then(|m| m.as_str().parse::<u16>().ok())
		.ok_or(ValidProxyError::InvalidPort(s.into()))?;

	Ok(())
}
