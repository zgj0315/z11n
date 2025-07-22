use agent_service::{config::AGENT_SERVICE_TOML, proto::HeartbeatReq};
use rustls::crypto::{CryptoProvider, ring};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("agent service starting");
    CryptoProvider::install_default(ring::default_provider())
        .expect("failed to install CryptoProvider");

    let mut client = agent_service::build_client(&AGENT_SERVICE_TOML.server.addr).await?;
    let req = HeartbeatReq {
        agent_id: "".to_string(),
        agent_type: "".to_string(),
    };
    let rsp = client.heartbeat(req).await?;
    log::info!("{:?}", rsp);
    Ok(())
}
