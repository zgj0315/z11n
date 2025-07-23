use crate::proto::z11n_service_client::Z11nServiceClient;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::fs;
use tonic::{
    Request, Status,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Certificate, Channel, ClientTlsConfig},
};

pub mod proto {
    tonic::include_proto!("z11n");
}
pub mod config;
pub mod host;

pub async fn build_client(
    url: &'static str,
) -> anyhow::Result<Z11nServiceClient<InterceptedService<Channel, impl Interceptor>>> {
    let mut pem = Vec::new();
    pem.extend_from_slice(&fs::read("./config/z11n-ca.crt")?);
    pem.extend_from_slice(&fs::read("./config/sub-ca.crt")?);
    let ca = Certificate::from_pem(pem);
    let tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("z11n.com");
    let channel = Channel::from_static(url).tls_config(tls)?.connect().await?;

    Ok(Z11nServiceClient::with_interceptor(
        channel.clone(),
        intercept,
    ))
}

pub static AGENT_ID_TOKEN: OnceCell<RwLock<(String, String)>> = OnceCell::new();

fn intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    if let Some(agent_id_token) = AGENT_ID_TOKEN.get() {
        let agent_id_token = agent_id_token.read();
        let agent_id = &agent_id_token.0;
        let token = &agent_id_token.1;
        req.metadata_mut()
            .insert("agent_id", agent_id.parse().unwrap());
        req.metadata_mut().insert("token", token.parse().unwrap());
    };
    Ok(req)
}
