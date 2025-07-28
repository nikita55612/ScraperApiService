#![allow(warnings)]
use axum::{routing, Router};
use std::sync::LazyLock;
use std::{net::SocketAddr, path::Path as OsPath, sync::Arc};
use tower_http::services::ServeFile;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::super::config as cfg;
use super::database as db;
use super::doc::ApiDoc;
use super::routers;
use super::states::AppState;

const DEFAULT_MASTER_TOKEN: &'static str = "ARk9dD6EjWRylJ4i2cPbW3sOjw7TTY52";
pub static MASTER_TOKEN: LazyLock<String> = LazyLock::new(|| {
    dotenv::dotenv().ok();
    std::env::var("MASTER_TOKEN").unwrap_or(DEFAULT_MASTER_TOKEN.into())
});
pub static ROOT_API_PATH: LazyLock<String> = LazyLock::new(|| cfg::get().api.root_api_path.clone());

pub async fn init() -> (tokio::net::TcpListener, Router) {
    let config = cfg::get();
    let assets_path = OsPath::new(&config.api.assets_path);
    let db_pool = Arc::new(db::init().await.expect("Database initialization error"));
    let app_state = Arc::new(
        AppState::new(
            db_pool,
            config.api.handlers_count,
            config.api.handler_queue_limit,
            config.api.open_ws_limit,
        )
        .await,
    );
    let app = Router::new()
        .nest(&*ROOT_API_PATH, routers::api(app_state))
        .merge(SwaggerUi::new("/swagger-ui").url(
            format!("{}/openapi.json", &*ROOT_API_PATH),
            ApiDoc::openapi(),
        ))
        .merge(routers::assets())
        .route_service("/", ServeFile::new(assets_path.join("index.html")))
        .fallback_service(ServeFile::new(assets_path.join("404.html")));
    let listener = tokio::net::TcpListener::bind(config.server.addr())
        .await
        .expect("Bind TcpListener Error");

    (listener, app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::SocketAddr, time::Duration};
    use tokio::time::sleep;

    async fn run_server() {
        let (listener, app) = init().await;

        tokio::task::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .unwrap()
        });
    }

    #[tokio::test]
    async fn test_run_server() {
        run_server().await;
        assert_eq!(true, true);
    }

    #[tokio::test]
    async fn test_run_server_loop() {
        cfg::init();
        println!("Server is runing...");
        run_server().await;
        loop {
            sleep(Duration::from_millis(100)).await;
        }
        assert_eq!(true, true);
    }

    #[tokio::test]
    async fn test_api_create_new_token() {
        run_server().await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "http://{}/create_new_token/?ttl=120000&ilimit=400&climit=400",
                cfg::get().server.addr()
            ))
            .header("Authorization", format!("Bearer {}", *MASTER_TOKEN))
            .send()
            .await
            .unwrap();

        println!("{:#?}", response);
        println!(
            "Response body: {:#?}",
            serde_json::from_str::<serde_json::Value>(&response.text().await.unwrap()).unwrap()
        );
        assert_eq!(true, true);
    }

    #[tokio::test]
    async fn test_api_cutout_token() {
        run_server().await;

        let client = reqwest::Client::new();
        let response = client
            .delete(format!(
                "http://{}/cutout_token/ss.ff4d207047b44209abf298d02c12eb7c",
                cfg::get().server.addr()
            ))
            .header("Authorization", format!("Bearer {}", *MASTER_TOKEN))
            .send()
            .await
            .unwrap();

        println!("{:#?}", response);
        println!(
            "Response body: {:#?}",
            serde_json::from_str::<serde_json::Value>(&response.text().await.unwrap()).unwrap()
        );
        assert_eq!(true, true);
    }

    #[tokio::test]
    async fn test_api_token_info() {
        run_server().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/token_info", cfg::get().server.addr()))
            .header(
                "Authorization",
                format!("Bearer {}", "ss.e5ac5b92c29c42e8ac61ef859f5af1f1"),
            )
            .send()
            .await
            .unwrap();

        println!("{:#?}", response);
        println!(
            "Response body: {:#?}",
            serde_json::from_str::<serde_json::Value>(&response.text().await.unwrap()).unwrap()
        );
        assert_eq!(true, true);
    }

    #[tokio::test]
    async fn test_api_order() {
        run_server().await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://{}/order", cfg::get().server.addr()))
            .body(
                r#"{
  "tokenId": "abc123dadadadadadaxyz",
  "products": [
    "wb/121212121",
    "oz/1212121233",
    "ym/9999999999-12121-1212121"
  ],
  "proxyList": [
    "GGR48S:12dsgb@147.45.62.117:8000",
    "TT5bS:1QswfUb@176.34.52.124:8000"
  ],
  "cookieList": [
    {
      "name": "session",
      "value": "session1234",
      "domain": "example.com",
      "path": "/",
      "secure": true
    },
    {
      "name": "user_pref",
      "value": "dark_mode",
      "domain": "example.com",
      "path": "/",
      "secure": false
    }
  ]
}"#,
            )
            .send()
            .await
            .unwrap();

        println!("{:#?}", response);
        println!(
            "Response body: {:#?}",
            serde_json::from_str::<serde_json::Value>(&response.text().await.unwrap()).unwrap()
        );
    }
}
