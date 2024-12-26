#![allow(warnings)]
use thiserror::Error as ThisError;


#[derive(ThisError, Debug)]
pub enum ScraperError {
    #[error(r#"InvalidSymbol"#)]
    InvalidSymbol,
    #[error(r#"InvalidProductId"#)]
    InvalidProductId,
    #[error(r#"ParseProductError"#)]
    ParseProductError,
}
