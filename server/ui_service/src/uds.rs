use std::fs;
use std::path::Path;

use crate::z11n::HeartbeatRsp;
use prost::Message;
use tokio::sync::broadcast;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

pub async fn listen_uds(
    tx_heartbeat_rsp: broadcast::Sender<(String, HeartbeatRsp)>,
) -> anyhow::Result<()> {
    let path = Path::new(pub_lib::UDS_PATH);
    if path.exists() {
        if let Err(e) = fs::remove_file(path) {
            log::error!("remove file err: {}", e);
        }
    }
    let unix_listener = UnixListener::bind(pub_lib::UDS_PATH)?;
    loop {
        match unix_listener.accept().await {
            Ok((unix_stream, socket_addr)) => {
                log::info!("unix listener accept socket addr: {:?}", socket_addr);
                let rx_heartbeat_rsp = tx_heartbeat_rsp.subscribe();
                tokio::spawn(consume_unix_stream(unix_stream, rx_heartbeat_rsp));
            }
            Err(e) => {
                log::error!("unix listener accept err: {}", e);
                continue;
            }
        }
    }
}

async fn consume_unix_stream(
    mut unix_stream: UnixStream,
    mut rx_heartbeat_rsp: broadcast::Receiver<(String, HeartbeatRsp)>,
) -> anyhow::Result<()> {
    loop {
        let mut buf = vec![0; 1024];
        match unix_stream.read(&mut buf).await {
            Ok(n) => {
                log::info!(
                    "receive from client_service: {}",
                    String::from_utf8_lossy(&buf[..n])
                );
            }
            Err(e) => {
                log::error!("unix_stream.read err: {}", e);
            }
        }
        while let Ok((agent_id, heartbeat_rsp)) = rx_heartbeat_rsp.recv().await {
            let encoded: Vec<u8> = match bincode::encode_to_vec(
                &(agent_id, heartbeat_rsp.encode_to_vec()),
                bincode::config::standard(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("bincode::encode_to_vec err: {}", e);
                    continue;
                }
            };
            if let Err(e) = unix_stream.write_all(&encoded).await {
                log::error!("unix_stream.write_all err: {}", e);
            };
            log::info!("send to client_service: {:?}", heartbeat_rsp);
        }
    }
}
