use clap::Parser;
use client_service::{server, uds};
use migration::{Migrator, MigratorTrait};
use rustls::crypto::{CryptoProvider, ring};
use sea_orm::Database;

use std::{
    fs::{self, File},
    path::Path,
};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _args = Args::parse();
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("client service starting");

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

    let sled_db_clone = sled_db.clone();
    tokio::spawn(async move {
        if let Err(e) = uds::connect_uds(sled_db_clone).await {
            log::error!("uds::connect_uds err: {}", e);
        }
    });
    CryptoProvider::install_default(ring::default_provider())
        .expect("failed to install CryptoProvider");
    server::serve(db_conn, sled_db).await
}
