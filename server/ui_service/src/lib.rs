use moka::sync::Cache;
use sea_orm::DatabaseConnection;
use tokio::sync::broadcast;

use crate::{auth::CaptchaEntry, z11n::HeartbeatRsp};

pub mod agent;
pub mod auth;
pub mod config;
pub mod host;
pub mod llm_task;
pub mod role;
pub mod server;
pub mod uds;
pub mod user;
pub mod z11n;

#[derive(Clone)]
pub struct AppState {
    pub db_conn: DatabaseConnection,
    pub sled_db: sled::Db,
    pub tx_heartbeat_rsp: broadcast::Sender<(String, HeartbeatRsp)>,
    pub captcha_cache: Cache<String, CaptchaEntry>,
}
