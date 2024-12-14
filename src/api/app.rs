#![allow(warnings)]
use std::collections::HashMap;
use std::sync::Arc;
use std::{net::SocketAddr, path::Path as OsPath};
use axum::{
    http::{
        header,
        StatusCode
    },
    extract::{Request, State, Path, Query, ConnectInfo},
    response::{IntoResponse, Response, Json},
    routing,
    body::Body,
    Router
};
use axum_macros::debug_handler;
use once_cell::sync::OnceCell;
use tower_http::services::{ServeDir, ServeFile};

use super::super::utils::list_dir;
use super::states::AppState;
use super::error::ApiError;
use super::database as db;
use super::config as cfg;
use super::models::Token;


static MASTER_TOKEN: OnceCell<String> = OnceCell::new();
const DEFAULT_MASTER_TOKEN: &'static str = "ARk9dD6EjWRylJ4i2cPbW3sOjw7TTY529sIDiRSpXmAEiRdJ5IKjaOfcRLAXM7Q6p5LJsYsaUyCVmJhZ6q0jXGK0Yd1r2WI1wLEB0AJcTqqj6g7FBcOY06q8kfXzcsrM";


pub fn get_master_token() -> &'static str {
    MASTER_TOKEN.get_or_init(|| {
        dotenv::dotenv().ok();
        std::env::var("MASTER_TOKEN")
            .unwrap_or(
                DEFAULT_MASTER_TOKEN.into()
            )
    })
}


async fn init() -> Router {
    let assets_path = OsPath::new(
        &cfg::get().api.assets_path
    );

    let mut assets_router: Router = Router::new();
    for i in list_dir(&assets_path).unwrap_or_default().iter() {
        if let Some(file) = i.to_str() {
            assets_router = assets_router.route_service(
                &format!("/{file}"),
                ServeFile::new(
                    assets_path.join(file)
                )
            )
        }
    }

    let db_pool = Arc::new(
        db::init().await.unwrap()
    );

    let app_state = Arc::new(
        AppState::new(db_pool, 0).await
    );

    Router::new()

        .route("/hello_world", routing::get(hello_world))
        .route("/myip", routing::get(myip))
        .route("/create_new_token/", routing::post(create_new_token))
        .route("/cutout_token/:token_id", routing::delete(cutout_token))
        .route("/token_info", routing::get(token_info))
        .route("/order", routing::post(order))
        .route("/task/:order_hash", routing::get(task))

        //.route("/api/*path", method_router)

        .with_state(app_state)

        .merge(assets_router)

        .route_service(
            "/",
            ServeFile::new(
                assets_path.join("index.html")
            )
        )



        .fallback_service(
            ServeFile::new(
                assets_path.join("404.html")
            )
        )

}

pub async fn hello_world() -> &'static str {
    "Hello world!"
}

pub async fn myip(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    addr.to_string()
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
    let read_token = db::read_token(
        &state.db_pool,
        token_id
    ).await?;

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

#[cfg(test)]
mod tests {
    //Запуск ssh сервера
    //ssh -R 5500:localhost:5500 -N -f -o "ServerAliveInterval 60" -o "ServerAliveCountMax 3" server
    use super::*;
    use tokio::time::sleep;
    use std::time::Duration;


    async fn run_server() {
        let app = init().await;

        let listener = tokio::net::TcpListener::bind(
            cfg::get().server.addr()
        ).await.unwrap();

        tokio::task::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>()
            )
            .await
            .unwrap()
        });
    }


    #[tokio::test]
    async fn test_server() {
        run_server().await;
        assert_eq!(true, true);
    }

    #[tokio::test]
    async fn test_api_create_new_token() {
        run_server().await;

        let client = reqwest::Client::new();
        let response = client.post(
                format!(
                    "http://{}/create_new_token/?ttl=120000&ilimit=400",
                    cfg::get().server.addr()
                )
            )
            .header(
                "Authorization",
                format!("Bearer {}", get_master_token())
            )
            .send()
            .await
            .unwrap();

        println!("{:#?}", response);
        println!(
            "Response body: {:#?}",
            serde_json::from_str::<serde_json::Value>(
                &response.text().await.unwrap()
            ).unwrap()
        );
        assert_eq!(true, true);
    }

    #[tokio::test]
    async fn test_api_cutout_token() {
        run_server().await;

        let client = reqwest::Client::new();
        let response = client.delete(
                format!(
                    "http://{}/cutout_token/ss.ff4d207047b44209abf298d02c12eb7c",
                    cfg::get().server.addr()
                )
            )
            .header(
                "Authorization",
                format!("Bearer {}", get_master_token())
            )
            .send()
            .await
            .unwrap();

        println!("{:#?}", response);
        println!(
            "Response body: {:#?}",
            serde_json::from_str::<serde_json::Value>(
                &response.text().await.unwrap()
            ).unwrap()
        );
        assert_eq!(true, true);
    }

    #[tokio::test]
    async fn test_api_token_info() {
        run_server().await;

        let client = reqwest::Client::new();
        let response = client.get(
                format!(
                    "http://{}/token_info",
                    cfg::get().server.addr()
                )
            )
            .header(
                "Authorization",
                format!("Bearer {}", "ss.e5ac5b92c29c42e8ac61ef859f5af1f1")
            )
            .send()
            .await
            .unwrap();

        println!("{:#?}", response);
        println!(
            "Response body: {:#?}",
            serde_json::from_str::<serde_json::Value>(
                &response.text().await.unwrap()
            ).unwrap()
        );
        assert_eq!(true, true);
    }
}
