use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use agent_service::{
    AGENT_ID_TOKEN,
    config::AGENT_SERVICE_TOML,
    host,
    proto::{Empty, HostReq, RegisterReq},
};
use parking_lot::RwLock;
use rustls::crypto::{CryptoProvider, ring};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("agent service starting");
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

    CryptoProvider::install_default(ring::default_provider())
        .expect("failed to install CryptoProvider");

    let mut client = agent_service::build_client(&AGENT_SERVICE_TOML.server.addr).await?;

    if let Err(e) = AGENT_ID_TOKEN.set(RwLock::new((agent_id.clone(), "".to_string()))) {
        log::error!("AGENT_ID_TOKEN set err: {:?}", e);
    }
    let register_req = RegisterReq {
        agent_id: agent_id.clone(),
        agent_version: version.to_string(),
    };
    let register_rsp = client.register(register_req).await?;
    let token = register_rsp.get_ref().token.clone();

    if let Some(lock) = AGENT_ID_TOKEN.get() {
        let mut write = lock.write();
        *write = (agent_id, token);
    }

    let system = host::system()?;
    let host_req = HostReq {
        system: Some(system),
    };
    let rsp = client.host(host_req).await?;
    log::info!("host rsp: {rsp:?}");
    heartbeat().await?;
    Ok(())
}

async fn heartbeat() -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
    loop {
        interval.tick().await;
        let mut client = agent_service::build_client(&AGENT_SERVICE_TOML.server.addr).await?;
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
