use agent_service::{
    AGENT_ID_TOKEN,
    config::AGENT_SERVICE_TOML,
    proto::{HeartbeatReq, RegisterReq},
};
use parking_lot::RwLock;
use rustls::crypto::{CryptoProvider, ring};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("agent service starting");
    CryptoProvider::install_default(ring::default_provider())
        .expect("failed to install CryptoProvider");

    let mut client = agent_service::build_client(&AGENT_SERVICE_TOML.server.addr).await?;

    let agent_id = uuid::Uuid::new_v4().to_string();
    if let Err(e) = AGENT_ID_TOKEN.set(RwLock::new((agent_id.clone(), "".to_string()))) {
        log::error!("AGENT_ID_TOKEN set err: {:?}", e);
    }
    let register_req = RegisterReq {
        agent_id: agent_id.clone(),
        agent_version: "0.1.0".to_string(),
    };
    let register_rsp = client.register(register_req).await?;
    let token = register_rsp.get_ref().token.clone();

    if let Some(lock) = AGENT_ID_TOKEN.get() {
        let mut write = lock.write();
        *write = (agent_id, token);
    }

    let req = HeartbeatReq {};
    let rsp = client.heartbeat(req).await?;
    let mut stream = rsp.into_inner();
    loop {
        match stream.next().await {
            Some(v) => match v {
                Ok(heartbeat_rsp) => {
                    log::info!("heartbeat_rsp: {heartbeat_rsp:?}");
                }
                Err(e) => {
                    log::error!("stream {}", e);
                    break;
                }
            },
            None => {
                break;
            }
        }
    }
    Ok(())
}
