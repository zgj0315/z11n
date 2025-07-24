use sea_orm::DatabaseConnection;
use tokio::sync::broadcast;

use crate::z11n::HeartbeatRsp;

pub mod agent;
pub mod auth;
pub mod config;
pub mod host;
pub mod uds;
pub mod z11n;
#[derive(Clone)]
pub struct AppState {
    pub db_conn: DatabaseConnection,
    pub sled_db: sled::Db,
    pub tx_heartbeat_rsp: broadcast::Sender<HeartbeatRsp>,
}
