use client_service::{
    agent,
    config::CLIENT_SERVICE_TOML,
    proto::z11n_service_server::Z11nServiceServer,
    server::{Z11nInterceptor, Z11nServer},
};
use migration::{Migrator, MigratorTrait};
use rustls::crypto::{CryptoProvider, ring};
use sea_orm::Database;
use std::{
    fs::{self, File},
    path::Path,
};
use tonic::{
    codec::CompressionEncoding,
    service::interceptor::InterceptedService,
    transport::{Identity, Server, ServerTlsConfig},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("client service starting");

    CryptoProvider::install_default(ring::default_provider())
        .expect("failed to install CryptoProvider");

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

    let online_agent_cache = agent::init_cache(&db_conn).await?;

    let server = Z11nServer {
        db_conn,
        online_agent_cache,
    };
    let service = Z11nServiceServer::new(server)
        .send_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Gzip)
        .max_decoding_message_size(8 * 1024 * 1024)
        .max_encoding_message_size(8 * 1024 * 1024);
    let z11n_interceptor = Z11nInterceptor {};
    let cert = fs::read("./config/z11n-ca.crt")?;
    let key = fs::read("./config/z11n-ca.key")?;
    let identity = Identity::from_pem(cert, key);
    let addr = CLIENT_SERVICE_TOML.server.addr.parse()?;
    log::info!("client service listening on {}", addr);
    log::info!("client service is running");
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .add_service(InterceptedService::new(service, z11n_interceptor))
        .serve(addr)
        .await?;
    Ok(())
}
