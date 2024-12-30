#![allow(warnings)]
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::Path as OsPath,
    sync::Arc,
};
use axum::{
    body::Bytes,
    extract::{
        ConnectInfo,
        Path,
        Query,
        Request,
        State,
    },
    http::{
        header,
        HeaderMap,
        StatusCode,
    },
    response::{
        IntoResponse,
        Json,
        Response,
    },
    Router,
};
use tower_http::services::ServeFile;
use axum_macros::debug_handler;

use super::super::models::{
    api::{Order, Token},
    validation::Validation
};
use super::super::utils::list_dir;
use super::app::get_master_token;
use super::super::config as cfg;
use super::states::AppState;
use super::error::ApiError;
use super::database as db;


pub fn assets() -> Router {
	let assets_path = OsPath::new(
		&cfg::get().api.assets_path
	);

	let mut assets_router: Router = Router::new();
	for i in list_dir(&cfg::get().api.assets_path).unwrap_or_default().iter() {
		if let Some(file) = i.to_str() {
			assets_router = assets_router.route_service(
				&format!("/{file}"),
				ServeFile::new(
					assets_path.join(file)
				)
			)
		}
	}

	assets_router
}

fn extract_auth_header_from_request(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(ApiError::AuthorizationHeaderMissing)
}

fn extract_token_from_auth_header(auth_header: &str) -> Result<&str, ApiError> {
    match auth_header.strip_prefix("Bearer ") {
        Some(token) => Ok(token),
        None => Err(ApiError::InvalidAuthorizationHeader)
    }
}

fn extract_token_from_request(headers: &HeaderMap) -> Result<&str, ApiError> {
    let auth_header = extract_auth_header_from_request(headers)?;
    extract_token_from_auth_header(auth_header)
}

fn verify_master_token(headers: &HeaderMap) -> Result<(), ApiError> {
    let master_token = extract_token_from_request(headers)?;
    if master_token != get_master_token() {
        return Err(ApiError::InvalidMasterToken);
    }

    Ok(())
}

async fn verify_token(token_id: &str, db_pool: &db::Pool) -> Result<Token, ApiError> {
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

pub async fn hello_world() -> &'static str {
    "Hello world!"
}

pub async fn myip(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    addr.to_string()
}

#[debug_handler]
pub async fn create_new_token(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {

    verify_master_token(&headers)?;

    let new_token = Token::new(
        query.get("ttl").ok_or(
                ApiError::MissingUrlQueryParameter("ttl".into())
            )?
            .parse::<u64>().map_err(
                |_| ApiError::InvalidUrlQueryParameter("ttl".into())
            )?,
        query.get("ilimit").ok_or(
                ApiError::MissingUrlQueryParameter("ilimit".into())
            )?
            .parse::<u64>().map_err(
                |_| ApiError::InvalidUrlQueryParameter("ilimit".into())
            )?,
        query.get("cplimit").ok_or(
                ApiError::MissingUrlQueryParameter("cplimit".into())
            )?
            .parse::<u64>().map_err(
                |_| ApiError::InvalidUrlQueryParameter("cplimit".into())
            )?,
    );

    db::insert_token(&state.db_pool, &new_token).await?;

	Ok (
        ( StatusCode::CREATED, Json(new_token) ).into_response()
    )
}


#[debug_handler]
pub async fn cutout_token(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(token_id): Path<String>,
) -> Result<Response, ApiError> {

    verify_master_token(&headers)?;

    let cutout_token = db::cutout_token(
        &state.db_pool,
        &token_id
    ).await?;

    if let Some(token) = cutout_token {
        return Ok (
            ( StatusCode::OK, Json(token) ).into_response()
        );
    }

    Err(ApiError::TokenDoesNotExist)
}

#[debug_handler]
pub async fn token_info(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Response, ApiError> {

    let token_id = extract_token_from_request(&headers)?;
    let read_token = db::read_token(
        &state.db_pool,
        token_id
    ).await?;

    if let Some(token) = read_token {
        return Ok (
            ( StatusCode::OK, Json(token) ).into_response()
        );
    }

    Err(ApiError::TokenDoesNotExist)
}

#[debug_handler]
pub async fn order(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<Response, ApiError> {

    let token_id = extract_token_from_request(&headers)?;
    let token = verify_token(token_id, &state.db_pool).await?;

    let mut order = serde_json::from_slice::<Order>(&body)
        .map_err(|_| ApiError::InvalidOrderFormat)?;
    // Проверка пуст ли заказ. Внести другие элементы заказа при доступности
    if order.products.is_empty() {
        return Err(
            ApiError::EmptyOrder
        );
    }
    if order.products.len() > token.ilimit as usize {
        return Err(
            ApiError::OrderLimitExceeded(token.ilimit)
        );
    }
    if state.task_count_by_token_id(token_id).await > token.climit as usize {
        return Err(
            ApiError::ConcurrencyLimitExceeded(token.climit)
        );
    }
    order.validation()?;

    order.token_id = token_id.into();

    println!("{:#?}", order);

    let order_hash = state.insert_order(order).await?;

    Ok (
        ( StatusCode::OK, format!(r#"{{ "order_hash": {} }}"#, order_hash) ).into_response()
    )
}

#[debug_handler]
pub async fn task(
    State(state): State<Arc<AppState>>,
    Path(order_hash): Path<String>,
) -> Result<Response, ApiError> {

    let task = state.get_task_state(&order_hash).await?;

    Ok (
        ( StatusCode::OK, Json(task) ).into_response()
    )
}
