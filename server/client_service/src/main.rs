use client_service::{
    config::CLIENT_SERVICE_TOML,
    proto::z11n_service_server::Z11nServiceServer,
    server::{Z11nInterceptor, Z11nServer},
};
use std::fs;
use tonic::{
    codec::CompressionEncoding,
    service::interceptor::InterceptedService,
    transport::{Identity, Server, ServerTlsConfig},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("client service starting");
    let server = Z11nServer::default();
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
