use std::sync::LazyLock;

use axum::Router;
use axum::routing::get;
use clap::Parser;
use tower_http::trace::TraceLayer;
use tracing::info;

mod error;
mod filetype;
mod handlers;
mod myclap;
mod template;

use handlers::handler;
use myclap::Cli;

use crate::myclap::Config;

pub static CONFIG: LazyLock<Config> =
    LazyLock::new(|| Config::from_parser(Cli::parse()).expect("无法读取配置"));

#[tokio::main]
async fn main() {
    //日志
    tracing_subscriber::fmt()
        .with_max_level(CONFIG.loglevel)
        .init();

    info!("Log Level: {}",CONFIG.loglevel.to_string());
    info!("Image Width: {}px",CONFIG.width);

    info!("Working Directory: [{}]", CONFIG.directory.display());
    // 设置工作目录
    std::env::set_current_dir(CONFIG.directory.clone()).expect("无法设置工作目录");

    // 路由
    let app = Router::new()
        .route("/", get(handler))
        .route("/{*filename}", get(handler))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", CONFIG.port))
        .await
        .unwrap();
    info!("Listen on 127.0.0.1:{}", CONFIG.port);
    info!("Listen on 0.0.0.0:{}", CONFIG.port);

    axum::serve(listener, app).await.unwrap();
}
