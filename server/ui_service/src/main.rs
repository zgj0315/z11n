use std::{
    fs::{self, File},
    path::Path,
};

use clap::Parser;
use migration::{Migrator, MigratorTrait};

use sea_orm::Database;
use tokio::sync::broadcast;
use ui_service::{server, uds::listen_uds};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _args = Args::parse();
    log4rs::init_file("./config/log4rs.yml", Default::default())?;

    let db_dir = Path::new(pub_lib::DB_DIR);
    if !db_dir.exists() {
        fs::create_dir_all(db_dir)?;
        log::info!("create dir: {}", db_dir.to_string_lossy());
    }

    let db_path = Path::new(pub_lib::DB_PATH);
    if !db_path.exists() {
        File::create(db_path)?;
        log::info!("create file: {}", db_path.to_string_lossy());
    }

    let db_url = format!("sqlite://{}", db_path.to_string_lossy());
    let db_conn = Database::connect(&db_url).await?;
    log::info!("connect to {}", db_url);

    Migrator::up(&db_conn, None).await?;
    let data_path = Path::new(pub_lib::DATA_DIR);
    if !data_path.exists() {
        fs::create_dir_all(data_path)?;
        log::info!("create dir: {}", data_path.to_string_lossy());
    }
    let sled_path = data_path.join("sled_db");
    let sled_db = sled::open(sled_path)?;
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

    server::serve(db_conn, sled_db, tx_heartbeat_rsp).await
}
