#![allow(warnings)]
pub mod database;
pub mod routers;
pub mod stream;
pub mod states;
pub mod error;
pub mod app;


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::utils;

    #[tokio::test]
    async fn test_api() {
    }
}
