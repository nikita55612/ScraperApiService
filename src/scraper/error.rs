#![allow(warnings)]
use thiserror::Error as ThisError;


#[derive(ThisError, Debug)]
pub enum ScraperError {
    #[error(r#"InvalidMP"#)]
    InvalidMP,
    #[error(r#"InvalidProductId"#)]
    InvalidProductId,
    #[error(r#"ParseProduct"#)]
    ParseProduct,
}
