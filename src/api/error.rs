#![allow(warnings)]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use super::super::models::validation::ValidationError;


#[derive(Error, Debug)]
pub enum ApiError {
    #[error("{{ \"message\": \"Invalid master token has been transferred.\", \"error\": \"InvalidMasterToken\", \"code\": 100 }}")]
    InvalidMasterToken,

    #[error("{{ \"message\": \"The Authorization header is required but was not included in the request.\", \"error\": \"AuthorizationHeaderMissing\", \"code\": 101 }}")]
    AuthorizationHeaderMissing,

    #[error("{{ \"message\": \"Invalid Authorization header: expected format 'Bearer <token>'.\", \"error\": \"InvalidAuthorizationHeader\", \"code\": 102 }}")]
    InvalidAuthorizationHeader,

    #[error("{{ \"message\": \"Invalid Authorization token was sent.\", \"error\": \"InvalidAuthorizationToken\", \"code\": 103 }}")]
    InvalidAuthorizationToken,

    #[error("{{ \"message\": \"Your Token has expired.\", \"error\": \"TokenLifetimeExceeded\", \"code\": 104 }}")]
    TokenLifetimeExceeded,

    #[error("{{ \"message\": \"The required URL query parameter '{0}' is missing.\", \"error\": \"MissingUrlQueryParameter\", \"code\": 200 }}")]
    MissingUrlQueryParameter(String),

    #[error("{{ \"message\": \"The value of URL query parameter '{0}' is invalid.\", \"error\": \"InvalidUrlQueryParameter\", \"code\": 201 }}")]
    InvalidUrlQueryParameter(String),

    #[error("{{ \"message\": \"{0}\", \"error\": \"InvalidProxyParameter\", \"code\": 202 }}")]
    InvalidOrderProxyParameter(String),

    #[error("{{ \"message\": \"{0}\", \"error\": \"InvalidProductParameter\", \"code\": 203 }}")]
    InvalidOrderProductParameter(String),

    #[error("{{ \"message\": \"Failed to deserialize the request body into the order object.\", \"error\": \"InvalidOrderFormat\", \"code\": 204 }}")]
    InvalidOrderFormat,

    #[error("{{ \"message\": \"The order is empty.\", \"error\": \"EmptyOrder\", \"code\": 205 }}")]
    EmptyOrder,

    #[error("{{ \"message\": \"The handler queue is full '{0}' and cannot accept new tasks.\", \"error\": \"HandlerQueueOverflow\", \"code\": 300 }}")]
    HandlerQueueOverflow(u64),

    #[error("{{ \"message\": \"The order exceeds the maximum '{0}' allowed limit.\", \"error\": \"OrderLimitExceeded\", \"code\": 301 }}")]
    OrderLimitExceeded(u64),

    #[error("{{ \"message\": \"The token has exceeded the limit '{0}' for concurrent order processing.\", \"error\": \"ConcurrentProcessingLimitExceeded\", \"code\": 302 }}")]
    ConcurrencyLimitExceeded(u64),

    #[error("{{ \"message\": \"Failed to send the task to the handler.\", \"error\": \"TaskSendFailure\", \"code\": 303 }}")]
    TaskSendFailure,

    #[error("{{ \"message\": \"A task with the specified order_hash '{0}' already exists in the handler.\", \"error\": \"TaskAlreadyExists\", \"code\": 304 }}")]
    TaskAlreadyExists(String),

    #[error("{{ \"message\": \"A task with the specified order_hash does not exist.\", \"error\": \"TaskNotFound\", \"code\": 305 }}")]
    TaskNotFound,

    #[error("{{ \"message\": \"Token does not exist.\", \"error\": \"TokenDoesNotExist\", \"code\": 400 }}")]
    TokenDoesNotExist,

    #[error("{{ \"message\": \"Database transaction failed.\", \"error\": \"DatabaseError\", \"code\": 401 }}")]
    DatabaseError,

    #[error("{{ \"message\": \"Unknown server error.\", \"error\": \"Unknown\", \"code\": 0 }}")]
    Unknown,

    #[error("{{ \"message\": \"{0}\" }}")]
    Warning(String),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::AuthorizationHeaderMissing
            | Self::MissingUrlQueryParameter(_)
            | Self::InvalidUrlQueryParameter(_)
            | Self::InvalidOrderProxyParameter(_)
            | Self::InvalidOrderProductParameter(_)
            | Self::InvalidOrderFormat
            | Self::EmptyOrder => StatusCode::BAD_REQUEST,

            Self::TaskNotFound
            | Self::TokenDoesNotExist => StatusCode::NOT_FOUND,

            Self::HandlerQueueOverflow(_)
            | Self::TaskAlreadyExists(_)
            | Self::OrderLimitExceeded(_)
            | Self::ConcurrencyLimitExceeded(_) => StatusCode::CONFLICT,

            Self::InvalidMasterToken
            | Self::InvalidAuthorizationHeader
            | Self::InvalidAuthorizationToken
            | Self::TokenLifetimeExceeded => StatusCode::UNAUTHORIZED,

            Self::Unknown
            | Self::DatabaseError
            | Self::TaskSendFailure => StatusCode::INTERNAL_SERVER_ERROR,

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::from_str(&self.to_string()).unwrap_or_else(|_| {
            serde_json::json!({
                "message": "An error occurred.",
                "error": "InvalidErrorFormat"
            })
        })
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let body = Json(self.to_json());
        (status_code, body).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        ApiError::DatabaseError
    }
}


impl From<ValidationError> for ApiError {
    fn from(value: ValidationError) -> Self {
        match value {
            ValidationError::Proxy(e) =>
                ApiError::InvalidOrderProxyParameter(e.to_string()),

            ValidationError::Product(e) =>
                ApiError::InvalidOrderProductParameter(e.to_string()),
        }
    }
}
