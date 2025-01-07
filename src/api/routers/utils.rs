use std::collections::HashMap;

use axum::http::HeaderMap;
use reqwest::header;

use crate::{
	api::{
		app::get_master_token,
		error::ApiError,
		database as db
	},
	models::api::Token
};


pub fn extract_token_from_headers(headers: &HeaderMap) -> Result<&str, ApiError> {
    let auth_header = headers
		.get(header::AUTHORIZATION)
		.and_then(|header| header.to_str().ok())
		.ok_or(ApiError::AuthorizationHeaderMissing)?;
	match auth_header.strip_prefix("Bearer ") {
        Some(token) => Ok(token),
        None => Err(ApiError::InvalidAuthorizationHeader)
    }
}

pub fn verify_master_token(headers: &HeaderMap) -> Result<(), ApiError> {
    let master_token = extract_token_from_headers(headers)?;
    if master_token != get_master_token() {
        return Err(ApiError::InvalidMasterToken);
    }

    Ok(())
}

pub async fn verify_token(token_id: &str, db_pool: &db::Pool) -> Result<Token, ApiError> {
    let token = db::read_token(
        db_pool,
        token_id
    ).await?
        .ok_or(ApiError::InvalidAuthorizationToken)?;
    if token.is_expired() {
        return Err(
            ApiError::TokenLifetimeExceeded
        );
    }

    Ok(token)
}

pub fn new_token_from_query(query: &HashMap<String, String>) -> Result<Token, ApiError> {
	let new_token = Token::new(
        get_query_param(query, "ttl")?
            .parse::<u64>().map_err(
                |_| ApiError::InvalidUrlQueryParameter("ttl".into())
            )?,
        get_query_param(query, "op_limit")?
            .parse::<u64>().map_err(
                |_| ApiError::InvalidUrlQueryParameter("op_limit".into())
            )?,
        get_query_param(query, "tc_limit")?
            .parse::<u64>().map_err(
                |_| ApiError::InvalidUrlQueryParameter("tc_limit".into())
            )?,
    );

	Ok (new_token)
}

pub fn get_query_param<'a>(
    query: &'a HashMap<String, String>,
    key: &'a str
) -> Result<&'a String, ApiError> {

    query.get(key).ok_or(
        ApiError::MissingUrlQueryParameter(key.into())
    )
}

//pub fn order_validation(order: Order)
