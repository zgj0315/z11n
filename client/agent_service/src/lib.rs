use crate::proto::z11n_service_client::Z11nServiceClient;
use std::fs;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

pub mod proto {
    tonic::include_proto!("z11n");
}
pub mod config;

pub async fn build_client(url: &'static str) -> anyhow::Result<Z11nServiceClient<Channel>> {
    let mut pem = Vec::new();
    pem.extend_from_slice(&fs::read("./config/z11n-ca.crt")?);
    pem.extend_from_slice(&fs::read("./config/sub-ca.crt")?);
    let ca = Certificate::from_pem(pem);
    let tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("z11n.com");
    let channel = Channel::from_static(url).tls_config(tls)?.connect().await?;
    Ok(Z11nServiceClient::new(channel))
}
