use crate::{
    config::Z11N_AGENT_TOML,
    proto::{Empty, RegisterReq, z11n_service_client::Z11nServiceClient},
};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use tokio_stream::StreamExt;
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

pub static AGENT_ID_TOKEN: OnceCell<RwLock<(String, String)>> = OnceCell::new();

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

fn intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    if let Some(agent_id_token) = AGENT_ID_TOKEN.get() {
        let agent_id_token = agent_id_token.read();
        let agent_id = &agent_id_token.0;
        let token = &agent_id_token.1;
        req.metadata_mut()
            .insert("agent_id", agent_id.parse().unwrap());
        req.metadata_mut().insert("token", token.parse().unwrap());
        Ok(req)
    } else {
        Err(Status::unauthenticated("agent_id_token not initialized"))
    }
}

pub async fn agent_register() -> anyhow::Result<()> {
    let agent_id_config = Path::new("./config/.agent_id");
    let agent_id = if agent_id_config.exists() {
        fs::read_to_string(agent_id_config)?
    } else {
        let agent_id = uuid::Uuid::new_v4().to_string();
        let mut file = File::create(agent_id_config)?;
        file.write_all(agent_id.as_bytes())?;
        agent_id
    };
    let version = env!("CARGO_PKG_VERSION");
    log::info!("agent_id: {agent_id}, version: {version}");
    let agent_version_config = Path::new("./config/.agent_version");
    let mut file = File::create(agent_version_config)?;
    file.write_all(version.as_bytes())?;

    let register_req = RegisterReq {
        agent_id: agent_id.clone(),
        agent_version: version.to_string(),
    };
    if AGENT_ID_TOKEN.get().is_none() {
        if let Err(e) = AGENT_ID_TOKEN.set(RwLock::new((agent_id.clone(), "".to_string()))) {
            log::error!("AGENT_ID_TOKEN set err: {:?}", e);
        }
    }
    let mut client = build_client(&Z11N_AGENT_TOML.server.addr).await?;
    let register_rsp = client.register(register_req).await?;
    let token = register_rsp.get_ref().token.clone();

    if let Some(lock) = AGENT_ID_TOKEN.get() {
        let mut write = lock.write();
        *write = (agent_id, token);
    }
    Ok(())
}

pub async fn heartbeat() -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
    loop {
        interval.tick().await;
        let mut client = build_client(&Z11N_AGENT_TOML.server.addr).await?;
        let req = Empty {};
        let rsp = client.heartbeat(req).await?;
        let mut stream = rsp.into_inner();
        while let Some(v) = stream.next().await {
            match v {
                Ok(heartbeat_rsp) => {
                    log::info!("heartbeat_rsp: {heartbeat_rsp:?}");
                }
                Err(e) => {
                    log::error!("stream {}", e);
                    break;
                }
            }
        }
    }
}
