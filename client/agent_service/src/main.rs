use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use agent_service::{
    AGENT_ID_TOKEN,
    config::AGENT_SERVICE_TOML,
    host,
    proto::{
        Empty, HeartbeatRsp, HostReq, RegisterReq, heartbeat_rsp::Task, upload_host::InfoType,
    },
};
use parking_lot::RwLock;
use rustls::crypto::{CryptoProvider, ring};
use tokio::sync::mpsc;
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

    let (tx_heartbeat_rsp, rx_heartbeat_rsp) = mpsc::channel(1_000);

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

    tokio::spawn(async move {
        if let Err(e) = consume_heartbeat_rsp(rx_heartbeat_rsp).await {
            log::error!("consume_heartbeat_rsp err: {}", e);
        }
    });
    heartbeat(tx_heartbeat_rsp).await?;
    Ok(())
}

async fn heartbeat(tx_heartbeat_rsp: mpsc::Sender<HeartbeatRsp>) -> anyhow::Result<()> {
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
                    tx_heartbeat_rsp.send(heartbeat_rsp).await?;
                }
                Err(e) => {
                    log::error!("stream {}", e);
                    break;
                }
            }
        }
    }
}

async fn consume_heartbeat_rsp(
    mut rx_heartbeat_rsp: mpsc::Receiver<HeartbeatRsp>,
) -> anyhow::Result<()> {
    while let Some(heartbeat_rsp) = rx_heartbeat_rsp.recv().await {
        if let Some(task) = heartbeat_rsp.task {
            match task {
                Task::UploadHost(upload_host) => match upload_host.info_type() {
                    InfoType::System => {
                        log::info!("upload system info");
                        if let Err(e) = upload_system_info().await {
                            log::error!("upload_system_info err: {}", e);
                        };
                    }
                    InfoType::Disk => {
                        log::info!("upload disk info");
                    }
                    InfoType::Network => {
                        log::info!("upload network info");
                    }
                },
            }
        }
    }
    Ok(())
}

async fn upload_system_info() -> anyhow::Result<()> {
    let mut client = agent_service::build_client(&AGENT_SERVICE_TOML.server.addr).await?;
    let system = host::system()?;
    let host_req = HostReq {
        system: Some(system),
        disk: None,
        network: None,
    };
    let rsp = client.host(host_req).await?;
    log::info!("host rsp: {rsp:?}");
    Ok(())
}
