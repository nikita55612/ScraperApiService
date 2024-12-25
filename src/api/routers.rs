#![allow(warnings)]
use std::{
	collections::HashMap,
	path::Path as OsPath,
	net::SocketAddr,
	sync::Arc
};
use axum::{
    extract::{ConnectInfo, Path, Query, Request, State}, http::{
        header,
        StatusCode
    }, response::{IntoResponse, Json, Response}, Router
};
use axum_macros::debug_handler;
use tower_http::services::ServeFile;

use super::super::models::api::Token;
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

fn extract_auth_header_from_request(req: &Request) -> Result<&str, ApiError> {
    req
        .headers()
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

fn extract_token_from_request(req: &Request) -> Result<&str, ApiError> {
    let auth_header = extract_auth_header_from_request(&req)?;
    extract_token_from_auth_header(auth_header)
}

fn verify_master_token(req: &Request) -> Result<(), ApiError> {
    let master_token = extract_token_from_request(&req)?;
    if master_token != get_master_token() {
        return Err(ApiError::InvalidMasterToken);
    }

    Ok(())
}

pub async fn hello_world() -> &'static str {
    "Hello world!"
}

pub async fn myip(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    addr.to_string()
}

#[debug_handler]
pub async fn create_new_token(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
    req: Request,
) -> Result<Response, ApiError> {

    verify_master_token(&req)?;

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
    );

    db::insert_token(&state.db_pool, &new_token).await?;

	Ok (
        ( StatusCode::CREATED, Json(new_token) ).into_response()
    )
}


#[debug_handler]
pub async fn cutout_token(
    State(state): State<Arc<AppState>>,
    Path(token_id): Path<String>,
    req: Request,
) -> Result<Response, ApiError> {

    verify_master_token(&req)?;

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
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, ApiError> {

    let token_id = extract_token_from_request(&req)?;
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
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, ApiError> {

    let token_id = extract_token_from_request(&req)?;

    // let _read_token = db::read_token(
    //     &state.db_pool,
    //     token_id
    // ).await?;

    // Поменять модель Order
    // Вместо поля items сделать products
    // Для парсинга других предметов добавить поля
    // Необходимость в поле type_ отпадает
    // Добавить больше кастомизации для парсинга (куки, заголовки, геолокация)
    // Вопрос по геолокации (Как передавать и будет ли работать)

    Ok (
        ( StatusCode::OK, "" ).into_response()
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
