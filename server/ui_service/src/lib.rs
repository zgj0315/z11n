use sea_orm::DatabaseConnection;

pub mod agent;
pub mod auth;
pub mod config;
pub mod host;
pub mod z11n;

#[derive(Clone)]
pub struct AppState {
    pub db_conn: DatabaseConnection,
    pub sled_db: sled::Db,
}
