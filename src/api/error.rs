#![allow(warnings)]
use axum::{
    response::{
        IntoResponse,
        Response
    },
    http::StatusCode,
    Json,
};
use thiserror::Error;

use super::super::{
    scraper::error::ReqSessionError,
    models::validation::ValidationError
};


#[derive(Error, Debug)]
pub enum ApiError {
    #[error("{{ \"error\": \"InvalidMasterToken\", \"code\": 101, \"message\": \"Invalid master token provided.\" }}")]
    InvalidMasterToken,

    #[error("{{ \"error\": \"MissingAuthorizationHeader\", \"code\": 102, \"message\": \"Missing Authorization header.\" }}")]
    MissingAuthorizationHeader,

    #[error("{{ \"error\": \"MalformedAuthorizationHeader\", \"code\": 103, \"message\": \"Invalid Authorization header format. Expected 'Bearer <token>'.\" }}")]
    MalformedAuthorizationHeader,

    #[error("{{ \"error\": \"InvalidAccessToken\", \"code\": 104, \"message\": \"Invalid access token provided.\" }}")]
    InvalidAccessToken,

    #[error("{{ \"error\": \"AccessTokenExpired\", \"code\": 105, \"message\": \"Access token has expired.\" }}")]
    AccessTokenExpired,

    #[error("{{ \"error\": \"MissingUrlQueryParameter\", \"code\": 200, \"message\": \"Missing required URL query parameter: '{0}'.\" }}")]
    MissingUrlQueryParameter(String),

    #[error("{{ \"error\": \"InvalidUrlQueryParameter\", \"code\": 201, \"message\": \"Invalid value for URL query parameter: '{0}'.\" }}")]
    InvalidUrlQueryParameter(String),

    #[error("{{ \"error\": \"InvalidOrderParameter\", \"code\": 202, \"message\": \"Invalid parameter value for: {0}.\" }}")]
    InvalidOrderParameter(String),

    #[error("{{ \"error\": \"InvalidOrderFormat\", \"code\": 203, \"message\": \"Failed to deserialize the request body into the order object.\" }}")]
    InvalidOrderFormat,

    #[error("{{ \"error\": \"EmptyRequestBody\", \"code\": 204, \"message\": \"Request body is empty. Expected '{0}' structure.\" }}")]
    EmptyRequestBody(String),

    #[error("{{ \"error\": \"EmptyOrder\", \"code\": 205, \"message\": \"The submitted order is empty.\" }}")]
    EmptyOrder,

    #[error("{{ \"error\": \"QueueOverflow\", \"code\": 300, \"message\": \"Handler queue is full. Maximum tasks allowed: '{0}'.\" }}")]
    QueueOverflow(u64),

    #[error("{{ \"error\": \"ProductLimitExceeded\", \"code\": 301, \"message\": \"Order exceeds the maximum product limit: '{0}'.\" }}")]
    ProductLimitExceeded(u64),

    #[error("{{ \"error\": \"ConcurrencyLimitExceeded\", \"code\": 302, \"message\": \"Token has exceeded the concurrent processing limit: '{0}'.\" }}")]
    ConcurrencyLimitExceeded(u64),

    #[error("{{ \"error\": \"DuplicateTask\", \"code\": 303, \"message\": \"Task with the specified order_hash '{0}' already exists.\" }}")]
    DuplicateTask(String),

    #[error("{{ \"error\": \"WebSocketLimitExceeded\", \"code\": 304, \"message\": \"Cannot establish new WebSocket connection as server has reached maximum limit of '{0}' concurrent connections.\" }}")]
    WebSocketLimitExceeded(u32),

    #[error("{{ \"error\": \"AccessRestricted\", \"code\": 305, \"message\": \"Access to the method is restricted.\" }}")]
    AccessRestricted,

    #[error("{{ \"error\": \"TokenDoesNotExist\", \"code\": 400, \"message\": \"Token does not exist.\" }}")]
    TokenDoesNotExist,

    #[error("{{ \"error\": \"TaskNotFound\", \"code\": 401, \"message\": \"A task with the specified order_hash does not exist.\" }}")]
    TaskNotFound,

    #[error("{{ \"error\": \"PathNotFound\", \"code\": 404, \"message\": \"The requested path was not found.\" }}")]
    PathNotFound,

    #[error("{{ \"error\": \"TaskSendFailure\", \"code\": 500, \"message\": \"Failed to send the task to the handler.\" }}")]
    TaskSendFailure,

    #[error("{{ \"error\": \"ReqwestSessionError\", \"code\": 501, \"message\": \"{0}.\" }}")]
    ReqwestSessionError(String),

    #[error("{{ \"error\": \"DatabaseError\", \"code\": 502, \"message\": \"Database transaction failed.\" }}")]
    DatabaseError,

    #[error("{{ \"error\": \"SerializationError\", \"code\": 503, \"message\": \"Failed to serialize object.\" }}")]
    SerializationError,

    #[error("{{ \"error\": \"Unknown\", \"code\": 0, \"message\": \"Unknown server error.\" }}")]
    UnknownError
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingAuthorizationHeader
            | Self::MissingUrlQueryParameter(_)
            | Self::InvalidUrlQueryParameter(_)
            | Self::InvalidOrderParameter(_)
            | Self::InvalidOrderFormat
            | Self::EmptyRequestBody(_)
            | Self::EmptyOrder => StatusCode::BAD_REQUEST,

            Self::TaskNotFound
            | Self::TokenDoesNotExist
            | Self::PathNotFound => StatusCode::NOT_FOUND,

            Self::QueueOverflow(_)
            | Self::DuplicateTask(_)
            | Self::ProductLimitExceeded(_)
            | Self::ConcurrencyLimitExceeded(_)
            | Self::WebSocketLimitExceeded(_)
            | Self::AccessRestricted => StatusCode::CONFLICT,

            Self::InvalidMasterToken
            | Self::MalformedAuthorizationHeader
            | Self::InvalidAccessToken
            | Self::AccessTokenExpired => StatusCode::UNAUTHORIZED,

            Self::UnknownError
            | Self::DatabaseError
            | Self::TaskSendFailure
            | Self::ReqwestSessionError(_)
            | Self::SerializationError => StatusCode::INTERNAL_SERVER_ERROR,

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

impl From<ReqSessionError> for ApiError {
    fn from(value: ReqSessionError) -> Self {
        ApiError::ReqwestSessionError(value.to_string())
    }
}

impl From<ValidationError> for ApiError {
    fn from(value: ValidationError) -> Self {
        match value {
            ValidationError::Proxy(e) =>
                ApiError::InvalidOrderParameter(format!("order proxy {}", e)),

            ValidationError::Product(e) =>
                ApiError::InvalidOrderParameter(format!("order product {}", e)),
        }
    }
}
