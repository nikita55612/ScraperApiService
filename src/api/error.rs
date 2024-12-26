#![allow(warnings)]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use super::super::models::validation::ValidProxyError;


#[derive(Error, Debug)]
pub enum ApiError {
    #[error("{{ \"message\": \"Invalid master token has been transferred.\", \"error\": \"InvalidMasterToken\", \"code\": 100 }}")]
    InvalidMasterToken,

    #[error("{{ \"message\": \"The 'Authorization' header is required but was not included in the request.\", \"error\": \"AuthorizationHeaderMissing\", \"code\": 101 }}")]
    AuthorizationHeaderMissing,

    #[error("{{ \"message\": \"Invalid Authorization header: expected format 'Bearer <token>'.\", \"error\": \"InvalidAuthorizationHeader\", \"code\": 102 }}")]
    InvalidAuthorizationHeader,

    #[error("{{ \"message\": \"The required URL query parameter '{0}' is missing.\", \"error\": \"MissingUrlParameter\", \"code\": 200 }}")]
    MissingUrlQueryParameter(String),

    #[error("{{ \"message\": \"The value of URL query parameter '{0}' is invalid.\", \"error\": \"InvalidUrlParameter\", \"code\": 201 }}")]
    InvalidUrlQueryParameter(String),

    #[error("{{ \"message\": \"The handler queue is full '{0}' and cannot accept new tasks.\", \"error\": \"HandlerQueueOverflow\", \"code\": 300 }}")]
    HandlerQueueOverflow(u64),

    #[error("{{ \"message\": \"Failed to send the task to the handler.\", \"error\": \"TaskSendFailure\", \"code\": 301 }}")]
    TaskSendFailure,

    #[error("{{ \"message\": \"A task with the specified order_hash '{0}' already exists in the handler.\", \"error\": \"TaskAlreadyExists\", \"code\": 302 }}")]
    TaskAlreadyExists(String),

    #[error("{{ \"message\": \"A task with the specified order_hash does not exist.\", \"error\": \"TaskNotFound\", \"code\": 303 }}")]
    TaskNotFound,

    #[error("{{ \"message\": \"{0}\", \"error\": \"InvalidProxyParameter\", \"code\": 202 }}")]
    InvalidProxyParameter(String),

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
            | Self::InvalidUrlQueryParameter(_) => StatusCode::BAD_REQUEST,

            Self::InvalidMasterToken
            | Self::InvalidAuthorizationHeader
            | Self::TokenDoesNotExist => StatusCode::UNAUTHORIZED,

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

impl From<ValidProxyError> for ApiError {
    fn from(value: ValidProxyError) -> Self {
        ApiError::InvalidProxyParameter(value.to_string())
    }
}

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("Invalid proxy format")]
    InvalidFormat,
    #[error("Invalid IP address")]
    InvalidIp,
    #[error("Invalid port number")]
    InvalidPort
}
