use std::{
    fs::{self, File},
    path::Path,
};

use clap::Parser;
use migration::{Migrator, MigratorTrait};
use prost::Message;
use rustls::crypto::{CryptoProvider, ring};
use sea_orm::Database;
use tokio::sync::broadcast;
use ui_service::z11n::HeartbeatRsp;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _args = Args::parse();
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("server starting");

    let data_path = Path::new(pub_lib::DATA_DIR);
    if !data_path.exists() {
        fs::create_dir_all(data_path)?;
        log::info!("create dir: {}", data_path.to_string_lossy());
    }

    let db_path = data_path.join("z11n.sqlite");
    if !db_path.exists() {
        File::create(&db_path)?;
        log::info!("create file: {}", db_path.to_string_lossy());
    }

    let db_url = format!("sqlite://{}", db_path.to_string_lossy());
    let db_conn = Database::connect(&db_url).await?;
    log::info!("connect to {}", db_url);

    Migrator::up(&db_conn, None).await?;

    let sled_path = data_path.join("heartbeat_rsp.sled_db");
    let heartbeat_rsp_sled_db = sled::open(sled_path)?;

    let db_conn_clone = db_conn.clone();
    let heartbeat_rsp_sled_db_clone = heartbeat_rsp_sled_db.clone();
    CryptoProvider::install_default(ring::default_provider())
        .expect("failed to install CryptoProvider");

    tokio::spawn(async move {
        if let Err(e) =
            client_service::server::serve(db_conn_clone, heartbeat_rsp_sled_db_clone).await
        {
            log::error!("client_service::server::serve err: {}", e);
        }
    });
    let (tx_heartbeat_rsp, rx_heartbeat_rsp) = broadcast::channel::<(String, HeartbeatRsp)>(1_000);
    tokio::spawn(async move {
        if let Err(e) = put_heartbeat_rsp_2_sled(rx_heartbeat_rsp, heartbeat_rsp_sled_db).await {
            log::error!("put_heartbeat_rsp_2_sled err: {}", e);
        }
    });

    let sled_path = data_path.join("token.sled_db");
    let token_sled_db = sled::open(sled_path)?;
    ui_service::server::serve(db_conn, token_sled_db, tx_heartbeat_rsp).await
}

async fn put_heartbeat_rsp_2_sled(
    mut rx_heartbeat_rsp: broadcast::Receiver<(String, HeartbeatRsp)>,
    sled_db: sled::Db,
) -> anyhow::Result<()> {
    while let Ok((agent_id, heartbeat_rsp)) = rx_heartbeat_rsp.recv().await {
        match sled_db.remove(&agent_id)? {
            Some(encoded) => {
                let (mut heartbeat_rsp_encodeds, _len): (Vec<Vec<u8>>, usize) =
                    match bincode::decode_from_slice(&encoded[..], bincode::config::standard()) {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("bincode::decode_from_slice err: {}", e);
                            continue;
                        }
                    };
                heartbeat_rsp_encodeds.push(heartbeat_rsp.encode_to_vec());
                let encoded: Vec<u8> = match bincode::encode_to_vec(
                    &heartbeat_rsp_encodeds,
                    bincode::config::standard(),
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("bincode::encode_to_vec err: {}", e);
                        continue;
                    }
                };
                if let Err(e) = sled_db.insert(agent_id.clone(), encoded) {
                    log::error!("sled_db.insert err: {}", e);
                }
                log::info!(
                    "receive from ui_service: {agent_id}, cmd size: {}",
                    heartbeat_rsp_encodeds.len()
                );
            }
            None => {
                let heartbeat_rsp_encodeds = vec![heartbeat_rsp.encode_to_vec()];
                let encoded: Vec<u8> = match bincode::encode_to_vec(
                    &heartbeat_rsp_encodeds,
                    bincode::config::standard(),
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("bincode::encode_to_vec err: {}", e);
                        continue;
                    }
                };
                if let Err(e) = sled_db.insert(agent_id.clone(), encoded) {
                    log::error!("sled_db.insert err: {}", e);
                }
                log::info!(
                    "receive from ui_service: {agent_id}, cmd size: {}",
                    heartbeat_rsp_encodeds.len()
                );
            }
        }
    }
    Ok(())
}
