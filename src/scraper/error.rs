#![allow(warnings)]
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ScraperError {
    #[error(r#"InvalidSymbol"#)]
    InvalidSymbol,
    #[error(r#"InvalidProductId"#)]
    InvalidProductId,
    #[error(r#"ParseProductError"#)]
    ParseProductError,
}
