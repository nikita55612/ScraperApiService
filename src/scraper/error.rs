#![allow(warnings)]
use browser_bridge::BrowserError;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("InvalidSymbol")]
    InvalidSymbol,
    #[error("InvalidProductId")]
    InvalidProductId,
    #[error("ParseProductError")]
    ParseProductError,
}

#[derive(Error, Debug)]
pub enum ReqSessionError {
    #[error("BrowserError: {0}")]
	Browser(String),
    #[error("Failed to build a req client")]
	BuildReqClient,
    #[error("The request method is not available")]
    NotAvailableReqMethod,
    #[error("Request sending error")]
    RequestSending,
    #[error("Error extracting the response content")]
    ExtractResponseContent

}

impl From<BrowserError> for ReqSessionError {
	fn from(value: BrowserError) -> Self {
		Self::Browser(value.to_string())
	}
}
