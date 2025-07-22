use agent_service::{config::AGENT_SERVICE_TOML, proto::HeartbeatReq};
use rustls::crypto::{CryptoProvider, ring};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("agent service starting");
    CryptoProvider::install_default(ring::default_provider())
        .expect("failed to install CryptoProvider");

    let mut client = agent_service::build_client(&AGENT_SERVICE_TOML.server.addr).await?;

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
