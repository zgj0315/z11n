use pub_lib::UDS_PATH;
use tokio::{io::AsyncReadExt, net::UnixStream};

pub async fn connect_uds(sled_db: sled::Db) -> anyhow::Result<()> {
    'connect_loop: loop {
        log::info!("UnixStream::connect start");
        match UnixStream::connect(UDS_PATH).await {
            Ok(mut unix_stream) => {
                log::info!("UnixStream::connect success");
                loop {
                    let mut buf = vec![0; 1024 * 1024];
                    match unix_stream.read(&mut buf).await {
                        Ok(n) => {
                            log::info!("UnixStream::connect read");
                            let ((agent_id, heartbeat_rsp_encoded), _len): (
                                (String, Vec<u8>),
                                usize,
                            ) = match bincode::decode_from_slice(
                                &buf[..n],
                                bincode::config::standard(),
                            ) {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("bincode::decode_from_slice err: {}", e);
                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                    continue 'connect_loop;
                                }
                            };
                            match sled_db.remove(&agent_id) {
                                Ok(op) => match op {
                                    Some(encoded) => {
                                        let (mut heartbeat_rsp_encodeds, _len): (
                                            Vec<Vec<u8>>,
                                            usize,
                                        ) = match bincode::decode_from_slice(
                                            &encoded[..],
                                            bincode::config::standard(),
                                        ) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                log::error!(
                                                    "bincode::decode_from_slice err: {}",
                                                    e
                                                );
                                                continue;
                                            }
                                        };
                                        heartbeat_rsp_encodeds.push(heartbeat_rsp_encoded);
                                        let encoded: Vec<u8> = match bincode::encode_to_vec(
                                            &heartbeat_rsp_encodeds,
                                            bincode::config::standard(),
                                        ) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                log::error!("bincode::encode_to_vec err: {}", e);
                                                continue;
                                            }
                                        };
                                        if let Err(e) = sled_db.insert(agent_id.clone(), encoded) {
                                            log::error!("sled_db.insert err: {}", e);
                                        }
                                        log::info!(
                                            "receive from ui_service: {agent_id}, cmd size: {}",
                                            heartbeat_rsp_encodeds.len()
                                        );
                                    }
                                    None => {
                                        let heartbeat_rsp_encodeds = vec![heartbeat_rsp_encoded];
                                        let encoded: Vec<u8> = match bincode::encode_to_vec(
                                            &heartbeat_rsp_encodeds,
                                            bincode::config::standard(),
                                        ) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                log::error!("bincode::encode_to_vec err: {}", e);
                                                continue;
                                            }
                                        };
                                        if let Err(e) = sled_db.insert(agent_id.clone(), encoded) {
                                            log::error!("sled_db.insert err: {}", e);
                                        }
                                        log::info!(
                                            "receive from ui_service: {agent_id}, cmd size: {}",
                                            heartbeat_rsp_encodeds.len()
                                        );
                                    }
                                },
                                Err(e) => {
                                    log::error!("sled.remove err: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("unix_stream.read err: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("UnixStream::connect err: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }
    }
}
