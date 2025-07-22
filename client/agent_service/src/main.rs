use agent_service::{config::AGENT_SERVICE_TOML, proto::HeartbeatReq};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("agent service starting");

    let mut client = agent_service::build_client(&AGENT_SERVICE_TOML.server.addr).await?;
    let req = HeartbeatReq {
        agent_id: "".to_string(),
        agent_type: "".to_string(),
    };
    let rsp = client.heartbeat(req).await?;
    log::info!("{:?}", rsp);
    Ok(())
}
