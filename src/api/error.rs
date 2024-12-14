#![allow(warnings)]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

/// API-specific error type with structured error handling
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("{{ \"message\": \"Invalid master token has been transferred.\", \"error\": \"InvalidMasterToken\" }}")]
    InvalidMasterToken,

    #[error("{{ \"message\": \"The 'Authorization' header is required but was not included in the request.\", \"error\": \"AuthorizationHeaderMissing\" }}")]
    AuthorizationHeaderMissing,

    #[error("{{ \"message\": \"Invalid Authorization header: expected format 'Bearer <token>'.\", \"error\": \"InvalidAuthorizationHeader\" }}")]
    InvalidAuthorizationHeader,

    #[error("{{ \"message\": \"Unknown server error.\", \"error\": \"Unknown\" }}")]
    Unknown,

    #[error("{{ \"message\": \"The required URL query parameter '{0}' is missing.\", \"error\": \"MissingUrlParameter\" }}")]
    MissingUrlQueryParameter(String),

    #[error("{{ \"message\": \"The value of URL query parameter '{0}' is invalid.\", \"error\": \"InvalidUrlParameter\" }}")]
    InvalidUrlQueryParameter(String),

    #[error("{{ \"message\": \"Token does not exist.\", \"error\": \"TokenDoesNotExist\" }}")]
    TokenDoesNotExist,

    #[error("{{ \"message\": \"Database transaction failed.\", \"error\": \"DatabaseError\" }}")]
    DatabaseError,

    #[error("{0}")]
    Info(String),
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
