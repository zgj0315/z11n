use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    thread,
};

use agent_service::{
    AGENT_ID_TOKEN,
    config::AGENT_SERVICE_TOML,
    host,
    proto::{
        Empty, HeartbeatRsp, HostReq, RegisterReq, heartbeat_rsp::Task, upload_host::InfoType,
    },
};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use rustls::crypto::{CryptoProvider, ring};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
static HOST_INFO: OnceCell<RwLock<HostReq>> = OnceCell::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("agent service starting");
    if let Err(e) = HOST_INFO.set(HostReq::default().into()) {
        log::error!("HOST_INFO set err: {:?}", e);
    }
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

    let (tx_heartbeat_rsp, rx_heartbeat_rsp) = mpsc::channel(1_000);
    let (tx_req, rx_req) = mpsc::channel(1_000);

    thread::spawn(|| {
        if let Err(e) = consume_heartbeat_rsp(rx_heartbeat_rsp, tx_req) {
            log::error!("consume_heartbeat_rsp err: {}", e);
        }
    });
    tokio::spawn(async move {
        if let Err(e) = consume_req(rx_req).await {
            log::error!("consume_heartbeat_rsp err: {}", e);
        }
    });

    if let Err(e) = heartbeat(tx_heartbeat_rsp.clone()).await {
        log::error!("heartbeat api err: {}", e);
    }
    Ok(())
}

enum Req {
    HostReq(HostReq),
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

async fn consume_req(mut rx_req: mpsc::Receiver<Req>) -> anyhow::Result<()> {
    while let Some(req) = rx_req.recv().await {
        match req {
            Req::HostReq(host_req) => {
                let mut client =
                    agent_service::build_client(&AGENT_SERVICE_TOML.server.addr).await?;
                if let Err(e) = client.host(host_req).await {
                    log::error!("host api err: {}", e);
                }
            }
        }
    }
    Ok(())
}

fn consume_heartbeat_rsp(
    mut rx_heartbeat_rsp: mpsc::Receiver<HeartbeatRsp>,
    tx_req: mpsc::Sender<Req>,
) -> anyhow::Result<()> {
    while let Some(heartbeat_rsp) = rx_heartbeat_rsp.blocking_recv() {
        if let Some(task) = heartbeat_rsp.task {
            match task {
                Task::UploadHost(upload_host) => match upload_host.info_type() {
                    InfoType::System => {
                        log::info!("upload system info");
                        let system = host::system()?;
                        if let Some(lock) = HOST_INFO.get() {
                            let mut write = lock.write();
                            write.system = Some(system.clone()).into();
                        }
                        if let Some(lock) = HOST_INFO.get() {
                            let read = lock.read();
                            let host_req = HostReq {
                                system: read.system.clone(),
                                disks: read.disks.clone(),
                                networks: read.networks.clone(),
                            };
                            if let Err(e) = tx_req.blocking_send(Req::HostReq(host_req)) {
                                log::error!("tx_req send err: {}", e);
                            }
                        }
                    }
                    InfoType::Disk => {
                        log::info!("upload disk info");
                        let disks = host::disk()?;
                        if let Some(lock) = HOST_INFO.get() {
                            let mut write = lock.write();
                            write.disks = disks.into();
                        }
                        if let Some(lock) = HOST_INFO.get() {
                            let read = lock.read();
                            let host_req = HostReq {
                                system: read.system.clone(),
                                disks: read.disks.clone(),
                                networks: read.networks.clone(),
                            };
                            if let Err(e) = tx_req.blocking_send(Req::HostReq(host_req)) {
                                log::error!("tx_req send err: {}", e);
                            }
                        }
                    }
                    InfoType::Network => {
                        log::info!("upload network info");
                        let networks = host::network()?;
                        if let Some(lock) = HOST_INFO.get() {
                            let mut write = lock.write();
                            write.networks = networks.into();
                        }
                        if let Some(lock) = HOST_INFO.get() {
                            let read = lock.read();
                            let host_req = HostReq {
                                system: read.system.clone(),
                                disks: read.disks.clone(),
                                networks: read.networks.clone(),
                            };
                            if let Err(e) = tx_req.blocking_send(Req::HostReq(host_req)) {
                                log::error!("tx_req send err: {}", e);
                            }
                        }
                    }
                },
            }
        }
    }
    Ok(())
}
