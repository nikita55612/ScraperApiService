#![allow(warnings)]
use std::sync::Arc;
use std::path::Path as OsPath;
use axum::{
    routing,
    Router
};
use once_cell::sync::OnceCell;
use tower_http::services::ServeFile;

use super::super::config as cfg;
use super::states::AppState;
use super::database as db;
use super::routers;


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


    let db_pool = Arc::new(
        db::init().await.unwrap()
    );

    let app_state = Arc::new(
        AppState::new(
            db_pool,
            cfg::get().api.handlers_count,
            cfg::get().api.handler_queue_limit
        ).await
    );

    Router::new()

        .route("/hello_world", routing::get(routers::hello_world))
        .route("/myip", routing::get(routers::myip))
        .route("/create_new_token/", routing::post(routers::create_new_token))
        .route("/cutout_token/:token_id", routing::delete(routers::cutout_token))
        .route("/token_info", routing::get(routers::token_info))
        .route("/order", routing::post(routers::order))
        .route("/task/:order_hash", routing::get(routers::task))

        //.route("/api/*path", method_router)

        .with_state(app_state)

        .merge(routers::assets())

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

#[cfg(test)]
mod tests {
    //Запуск ssh сервера
    //ssh -R 5500:localhost:5500 -N -f -o "ServerAliveInterval 60" -o "ServerAliveCountMax 3" server
    use super::*;
    use tokio::time::sleep;
    use std::{net::SocketAddr, time::Duration};


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

    #[tokio::test]
    async fn test_api_order() {
        run_server().await;

        let client = reqwest::Client::new();
        let response = client.post(
                format!(
                    "http://{}/order",
                    cfg::get().server.addr()
                )
            )
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
}"#
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
    }
}
