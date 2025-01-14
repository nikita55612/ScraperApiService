#![allow(warnings)]
mod utils;
use tokio::time::{
    sleep,
    timeout
};
use once_cell::sync::Lazy;
use std::{
    sync::Arc,
    time::Duration,
    net::SocketAddr,
    collections::HashMap,
    path::Path as OsPath,
};
use axum::{
    routing,
    body::Bytes,
    extract::{
        ws::{
            WebSocket,
            Message
        },
        WebSocketUpgrade,
        ConnectInfo,
        Request,
        State,
        Query,
        Path,
    },
    http::{
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
use utoipa::OpenApi;
use utils::{
	verify_token,
    get_query_param,
	verify_master_token,
	new_token_from_query,
	extract_token_from_headers,
    extract_and_handle_order_from_body,
};


use super::{
    doc::ApiDoc,
    states::AppState,
    error::ApiError,
    database as db,
    app::ROOT_API_PATH,
    super::{
        utils::list_dir,
        config as cfg,
        models::{
            api::{
                Task,
                Order,
                Token,
                TaskStatus,
                TaskProgress,
                ApiState
            },
            scraper::{
                Market,
                MARKET_MAP
            },
            validation::Validation
        },
    },
};


static TASK_WS_SENDING_INTERVAL: Lazy<u64> = Lazy::new(
    || cfg::get().api.task_ws_sending_interval
);

pub fn api(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/create-token/", routing::post(create_token))
        .route("/update-token/", routing::post(update_token))
        .route("/cutout-token/{token_id}", routing::delete(cutout_token))

        .route("/token-info", routing::get(token_info))
        .route("/token-info/{token_id}", routing::get(token_info_))
        .route("/test-token", routing::get(test_token))

        .route("/state", routing::get(state))

        .route("/order", routing::post(order))
        .route("/task/{order_hash}", routing::get(task).post(task))
        .route("/task-ws/{order_hash}", routing::any(task_ws))
        .route("/valid-order", routing::post(valid_order).get(valid_order))

        .with_state(app_state)

        .route("/config", routing::get(config))
        .route("/markets", routing::get(markets))
        //.route("/openapi.json", routing::get(openapi))
        .route("/ping", routing::get(ping))
        .route("/myip", routing::get(myip))

        .fallback(api_fallback)
}

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

static TEST_TOKEN: Lazy<Token> = Lazy::new(
    || {
        let test_token_cfg = &cfg::get().api.test_token;
        Token::new(
            test_token_cfg.ttl,
            test_token_cfg.op_limit,
            test_token_cfg.tc_limit
        )
    }
);

async fn ping() -> &'static str {
    "pong"
}

async fn myip(
    ConnectInfo(addr): ConnectInfo<SocketAddr>
) -> String {
    addr.to_string()
}

async fn openapi() -> Response {
    (StatusCode::OK, Json(ApiDoc::openapi())).into_response()
}

#[debug_handler]
async fn create_token(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {

    verify_master_token(&headers)?;

    let new_token = new_token_from_query(&query)?;
    db::insert_token(&state.db_pool, &new_token).await?;

	Ok ((StatusCode::CREATED, Json(new_token)).into_response())
}

#[debug_handler]
async fn cutout_token(
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
            (StatusCode::OK, Json(token)).into_response()
        );
    }

    Err (ApiError::TokenDoesNotExist)
}

#[debug_handler]
async fn update_token(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {

    verify_master_token(&headers)?;

    let mut update_token = new_token_from_query(&query)?;
    update_token.id = get_query_param(&query, "id")?.clone();
    db::update_token(&state.db_pool, &update_token).await?;

	Ok ((StatusCode::CREATED, Json(update_token)).into_response())
}

#[debug_handler]
async fn token_info(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Response, ApiError> {

    let token_id = extract_token_from_headers(&headers)?;
    let read_token = db::read_token(
        &state.db_pool,
        token_id
    ).await?;
    if let Some(token) = read_token {
        return Ok (
            (StatusCode::OK, Json(token)).into_response()
        );
    }

    Err (ApiError::TokenDoesNotExist)
}

#[debug_handler]
async fn token_info_(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(token_id): Path<String>
) -> Result<Response, ApiError> {

    let read_token = db::read_token(
        &state.db_pool,
        &token_id
    ).await?;
    if let Some(token) = read_token {
        return Ok (
            (StatusCode::OK, Json(token)).into_response()
        );
    }

    Err (ApiError::TokenDoesNotExist)
}

#[debug_handler]
async fn test_token(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>
) -> Result<Response, ApiError> {

    let mut cache_lock = state.cache.lock().await;
    if !cache_lock.blocked_addrs.insert(addr.to_string()) {
        return Err (ApiError::AccessRestricted);
    }
    let test_token = TEST_TOKEN.clone();
    db::insert_token(&state.db_pool, &test_token).await?;

	Ok ((StatusCode::CREATED, Json(test_token)).into_response())
}

#[debug_handler]
async fn config() -> Response {
    (StatusCode::OK, Json(cfg::get())).into_response()
}

#[debug_handler]
async fn state(
    State(state): State<Arc<AppState>>
) -> Response {

    let api_state = ApiState {
        handlers_count: state.handlers_count,
        tasks_queue_limit: state.handler_queue_limit * state.handlers_count,
        curr_task_queue: state.get_task_count().await,
        open_ws_limit: state.open_ws_limit,
        curr_open_ws: *state.open_ws_counter.lock().await
    };

    (StatusCode::OK, Json(api_state)).into_response()
}

#[debug_handler]
async fn markets() -> Response {
    (StatusCode::OK, Json(MARKET_MAP.clone())).into_response()
}

#[debug_handler]
async fn order(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<Response, ApiError> {

    let token_id = extract_token_from_headers(&headers)?;
    let token = verify_token(token_id, &state.db_pool).await?;
    let mut order = extract_and_handle_order_from_body(&body)?;
    if order.products.len() > token.op_limit as usize {
        return Err(ApiError::ProductLimitExceeded(token.op_limit));
    }
    if state.task_count_by_token_id(token_id).await > token.tc_limit as usize {
        return Err(ApiError::ConcurrencyLimitExceeded(token.tc_limit));
    }
    order.validation()?;
    order.token_id = token_id.into();
    let order_hash = state.insert_order(order).await?;

    Ok ((StatusCode::OK, order_hash).into_response())
}

#[debug_handler]
async fn valid_order(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<Response, ApiError> {

    let token_id = extract_token_from_headers(&headers)?;
    let _ = verify_token(token_id, &state.db_pool).await?;
    let mut order = extract_and_handle_order_from_body(&body)?;
    if order.products.len() > 1000 {
        return Err(ApiError::ProductLimitExceeded(1000));
    }
    order.validation()?;

    Ok ((StatusCode::OK, Json(order)).into_response())
}

#[debug_handler]
async fn task(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(order_hash): Path<String>
) -> Result<Response, ApiError> {

    let token_id = extract_token_from_headers(&headers)?;
    let _ = verify_token(token_id, &state.db_pool).await?;
    let task = state.get_task_state(&order_hash).await?;

    Ok ((StatusCode::OK, Json(task)).into_response())
}

#[debug_handler]
async fn task_ws(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(order_hash): Path<String>,
) -> Result<Response, ApiError> {

    let token_id = extract_token_from_headers(&headers)?;
    let _ = verify_token(token_id, &state.db_pool).await?;
    state.open_websocket().await?;
    let res = ws.protocols(["send-only"])
        .on_upgrade(
            move |socket| handle_task_ws(
                socket,
                state.clone(),
                order_hash
        )
    );

    Ok (res)
}

async fn handle_task_ws(
    mut socket: WebSocket,
    state: Arc<AppState>,
    order_hash: String
) {
    let mut prev_task = None;
    loop {
        let ping_msg = Message::Ping(Bytes::default());
        if socket.send(ping_msg).await.is_err() { break; }
        let task_res = state
            .get_task_state(&order_hash)
            .await;
        match task_res {
            Ok(task) => {
                if Some(&task) != prev_task.as_ref() {
                    let json_task = serde_json::to_string(&task);
                    if let Ok(json_task) = json_task {
                        let msg = Message::Text(
                            json_task.into()
                        );
                        if socket.send(msg).await.is_err() { break; }
                    } else {
                        let msg = Message::Text(
                            ApiError::SerializationError.to_string().into()
                        );
                        if socket.send(msg).await.is_err() { break; }
                        break;
                    }
                    prev_task = Some(task);
                }
            },
            Err(e) => {
                let msg = Message::Text(
                    e.to_string().into()
                );
                if socket.send(msg).await.is_err() { break; }
                break;
            }
        }
        let _ = timeout(
            Duration::from_millis(*TASK_WS_SENDING_INTERVAL),
            socket.recv()
        ).await;
        sleep(
            Duration::from_millis(20)
        ).await;
    }

    state.close_websocket().await;
}

async fn api_fallback() -> Response {
    ApiError::PathNotFound.into_response()
}
