use std::{
    fs::{self, File},
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use axum::{Router, middleware::from_extractor_with_state};
use axum_server::tls_rustls::RustlsConfig;
use migration::{Migrator, MigratorTrait};
use rustls::crypto;
use sea_orm::Database;
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};
use ui_service::{
    AppState, agent,
    auth::{self, RequireAuth},
    config::UI_SERVICE_TOML,
    host,
    uds::listen_uds,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;

    let db_dir = Path::new("../db");
    if !db_dir.exists() {
        fs::create_dir_all(db_dir)?;
        log::info!("create dir: {}", db_dir.to_string_lossy());
    }

    let db_path = Path::new(pub_lib::DB_PATH);
    if !db_path.exists() {
        File::create(&db_path)?;
        log::info!("create file: {}", db_path.to_string_lossy());
    }

    let db_url = format!("sqlite://{}", db_path.to_string_lossy());
    let db_conn = Database::connect(&db_url).await?;
    log::info!("connect to {}", db_url);

    Migrator::up(&db_conn, None).await?;

    let (tx_heartbeat_rsp, mut rx_heartbeat_rsp) = broadcast::channel(1_000);
    tokio::spawn(async move {
        while let Ok((_agent_id, _heartbeat_rsp)) = rx_heartbeat_rsp.recv().await {
            // log::info!("got {agent_id} heartbeat_rsp: {heartbeat_rsp:?}");
        }
    });
    let tx_heartbeat_rsp_clone = tx_heartbeat_rsp.clone();
    tokio::spawn(async move {
        if let Err(e) = listen_uds(tx_heartbeat_rsp_clone).await {
            log::error!("listen uds err: {}", e);
        }
    });
    let sled_dir = Path::new("./data");
    if !sled_dir.exists() {
        fs::create_dir_all(sled_dir)?;
        log::info!("create dir: {}", sled_dir.to_string_lossy());
    }
    let sled_path = sled_dir.join("sled_db");
    let sled_db = sled::open(sled_path)?;
    auth::token_expired_task(sled_db.clone()).await?;
    let app_state = AppState {
        db_conn,
        sled_db,
        tx_heartbeat_rsp,
    };
    let dist_path = if Path::new("../ui_web/dist").exists() {
        // 工程目录
        "../ui_web/dist"
    } else {
        // 部署目录
        "./html"
    };
    let app = Router::new()
        .fallback_service(
            ServeDir::new(dist_path).fallback(ServeFile::new(format!("{dist_path}/index.html"))),
        )
        .nest("/api", agent::routers(app_state.clone()))
        .nest("/api", auth::routers(app_state.clone()))
        .nest("/api", host::routers(app_state.clone()))
        .layer(from_extractor_with_state::<RequireAuth, _>(Arc::new(
            app_state,
        )));

    if let Err(e) = crypto::ring::default_provider().install_default() {
        log::error!("default_provider install err: {:?}", e);
    }
    let config = RustlsConfig::from_pem_file(
        PathBuf::from("./config").join("z11n-ca.crt"),
        PathBuf::from("./config").join("z11n-ca.key"),
    )
    .await?;
    let addr = SocketAddr::from_str(&UI_SERVICE_TOML.server.addr)?;
    log::info!("listening on {}", addr);
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    Ok(())
}
