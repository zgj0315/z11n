use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use axum::{Router, middleware::from_extractor_with_state};
use axum_server::tls_rustls::RustlsConfig;
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};

use crate::{
    AppState, agent,
    auth::{self, RequireAuth, auth_init},
    config::UI_SERVICE_TOML,
    host, llm_task, role, user,
    z11n::HeartbeatRsp,
};

pub async fn serve(
    db_conn: sea_orm::DatabaseConnection,
    sled_db: sled::Db,
    tx_heartbeat_rsp: broadcast::Sender<(String, HeartbeatRsp)>,
) -> anyhow::Result<()> {
    auth_init(db_conn.clone()).await?;
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
        .nest("/api", role::routers(app_state.clone()))
        .nest("/api", user::routers(app_state.clone()))
        .nest("/api", host::routers(app_state.clone()))
        .nest("/api", llm_task::routers(app_state.clone()))
        .layer(from_extractor_with_state::<RequireAuth, _>(Arc::new(
            app_state,
        )));

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
