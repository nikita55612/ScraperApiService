use axum::{body::Bytes, extract::Request, http::HeaderMap, middleware::Next, response::Response};
use reqwest::header;
use std::collections::HashMap;

use crate::{
    api::{app::MASTER_TOKEN, database as db, error::ApiError, logger},
    models::api::{Order, Token},
};

#[inline]
pub async fn log_middleware(req: Request, next: Next) -> Response {
    let req_method = req.method().to_string();
    let req_path = req.uri().to_string();

    let res = next.run(req).await;

    logger::write(
        log::Level::Info,
        "API",
        format!("{} {} {}", req_method, req_path, res.status()),
    )
    .await;

    res
}

#[inline]
pub fn extract_token_from_headers(headers: &HeaderMap) -> Result<&str, ApiError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(ApiError::MissingAuthorizationHeader)?;
    match auth_header.strip_prefix("Bearer ") {
        Some(token) => Ok(token),
        None => Err(ApiError::MalformedAuthorizationHeader),
    }
}

#[inline]
pub fn verify_master_token(headers: &HeaderMap) -> Result<(), ApiError> {
    let master_token = extract_token_from_headers(headers)?;
    if master_token != &*MASTER_TOKEN {
        return Err(ApiError::InvalidMasterToken);
    }

    Ok(())
}

#[inline]
pub async fn verify_token(token_id: &str, db_pool: &db::Pool) -> Result<Token, ApiError> {
    let token = db::read_token(db_pool, token_id)
        .await?
        .ok_or(ApiError::InvalidAccessToken)?;
    if token.is_expired() {
        return Err(ApiError::AccessTokenExpired);
    }

    Ok(token)
}

#[inline]
pub fn new_token_from_query(query: &HashMap<String, String>) -> Result<Token, ApiError> {
    let new_token = Token::new(
        get_query_param(query, "ttl")?
            .parse::<u64>()
            .map_err(|_| ApiError::InvalidUrlQueryParameter("ttl".into()))?,
        get_query_param(query, "op_limit")?
            .parse::<u64>()
            .map_err(|_| ApiError::InvalidUrlQueryParameter("op_limit".into()))?,
        get_query_param(query, "tc_limit")?
            .parse::<u64>()
            .map_err(|_| ApiError::InvalidUrlQueryParameter("tc_limit".into()))?,
    );

    Ok(new_token)
}

#[inline]
pub fn get_query_param<'a>(
    query: &'a HashMap<String, String>,
    key: &'a str,
) -> Result<&'a String, ApiError> {
    query
        .get(key)
        .ok_or(ApiError::MissingUrlQueryParameter(key.into()))
}

#[inline]
pub fn extract_and_handle_order_from_body(body: &Bytes) -> Result<Order, ApiError> {
    if body.is_empty() {
        return Err(ApiError::EmptyRequestBody("Order".into()));
    }
    let mut order =
        serde_json::from_slice::<Order>(&body).map_err(|_| ApiError::InvalidOrderFormat)?;
    if order.products.is_empty() {
        return Err(ApiError::EmptyOrder);
    }
    order.remove_duplicates();

    Ok(order)
}
