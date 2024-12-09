use serde_json::{Value, from_str};
use thiserror::Error as ThisError;
use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use axum::Json;


#[derive(ThisError, Debug)]
pub enum ApiError {
  #[error(r#"{{
    "message": "Invalid order ID or the order is not ready yet!",
    "error": "OrderNotReady",
    "queue_index": {0}
  }}"#)]
  OrderNotReady(u64),
  #[error(r#"["unknown error"]"#)]
  Unknown,
  #[error(r#"{0}"#)]
  Info(String)
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let json_string_error = self.to_string();
    (
        StatusCode::INTERNAL_SERVER_ERROR, 
        Json(from_str::<Value>(&json_string_error).unwrap())
    ).into_response()
  }
}