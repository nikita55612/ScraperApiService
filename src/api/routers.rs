use std::{net::SocketAddr, path::Path};

use axum::{
    extract::ConnectInfo, response::IntoResponse, routing::get, Router
    
};
use tower_http::services::{ServeDir, ServeFile};


fn init_router() -> Router {
    let assets_path = Path::new("assets");
    Router::new()
        .route_service(
            "/", 
            ServeFile::new(
                assets_path.join("index.html")
            )
        )
        .route_service(
            "/index.html", 
            ServeFile::new(
                assets_path.join("index.html")
            )
        )
        .route_service(
            "/main.js", 
            ServeFile::new(
                assets_path.join("main.js")
            )
        )
        .route_service(
            "/style.css", 
            ServeFile::new(
                assets_path.join("style.css")
            )
        )
        .route_service(
            "/404.html", 
            ServeFile::new(
                assets_path.join("404.html")
            )
        )

        .route("/hello_world", get(hello_world))
        .route("/myip", get(myip))

        .fallback_service(
            ServeFile::new(
                assets_path.join("404.html")
            )
        )
}

async fn hello_world() -> &'static str {
    "Hello world!"
}

async fn myip(
    ConnectInfo(addr): ConnectInfo<SocketAddr>
) -> String {
    addr.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_routers() {
        let app = init_router();
        let listener = tokio::net::TcpListener::bind("0.0.0.0:5501").await.unwrap();
        axum::serve(
            listener, app.into_make_service_with_connect_info::<SocketAddr>()
        ).await
        .unwrap();
        assert_eq!(true, true);
    }
}

