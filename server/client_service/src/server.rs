use crate::proto::{
    Empty, HeartbeatRsp, HostReq, RegisterReq, RegisterRsp, z11n_service_server::Z11nService,
};
use entity::{tbl_agent, tbl_host};
use moka::sync::Cache;
use prost::Message;
use pub_lib::AgentState;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status, metadata::MetadataMap, service::Interceptor};

#[derive(Debug, Clone)]
pub struct Z11nInterceptor {}

impl Interceptor for Z11nInterceptor {
    fn call(&mut self, req: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        let _agent_id = extract_metadata_value(req.metadata(), "agent_id")?;
        // log::info!("agent_id: {agent_id}");
        Ok(req)
    }
}

fn extract_metadata_value<'a>(metadata: &'a MetadataMap, key: &str) -> Result<&'a str, Status> {
    match metadata.get(key) {
        Some(v) => {
            if v.is_empty() {
                log::warn!("{} is empty", key);
                return Err(Status::new(
                    Code::InvalidArgument,
                    format!("{} is empty", key),
                ));
            }
            match v.to_str() {
                Ok(val) => Ok(val),
                Err(e) => {
                    log::error!("{} to_str error: {}", key, e);
                    Err(Status::new(Code::Internal, format!("{} to_str error", key)))
                }
            }
        }
        None => {
            log::warn!("{} not exist", key);
            Err(Status::new(
                Code::InvalidArgument,
                format!("{} not exist", key),
            ))
        }
    }
}

#[derive(Debug)]
pub struct Z11nServer {
    pub db_conn: DatabaseConnection,
    pub online_agent_cache: Cache<String, String>,
    pub sled_db: sled::Db,
}

#[tonic::async_trait]
impl Z11nService for Z11nServer {
    type HeartbeatStream = ReceiverStream<Result<HeartbeatRsp, Status>>;
    async fn heartbeat(
        &self,
        req: Request<Empty>,
    ) -> Result<Response<Self::HeartbeatStream>, Status> {
        let agent_id = extract_metadata_value(req.metadata(), "agent_id")?;
        let token = extract_metadata_value(req.metadata(), "token")?;
        self.online_agent_cache
            .insert(agent_id.to_string(), token.to_string());
        // log::info!("online in cache {}", agent_id);
        match self.online_agent_cache.get(agent_id) {
            Some(v) => {
                if !v.eq(token) {
                    return Err(tonic::Status::new(
                        tonic::Code::Unauthenticated,
                        "Unauthenticated".to_string(),
                    ));
                }
            }
            None => {
                return Err(tonic::Status::new(
                    tonic::Code::Unauthenticated,
                    "Unauthenticated".to_string(),
                ));
            }
        }
        let (tx, rx) = mpsc::channel(10);
        let sled_db_clone = self.sled_db.clone();
        let agent_id = agent_id.to_string();
        tokio::spawn(async move {
            match sled_db_clone.remove(&agent_id) {
                Ok(op) => {
                    if let Some(encoded) = op {
                        let (heartbeat_rsp_encodeds, _len): (Vec<Vec<u8>>, usize) =
                            match bincode::decode_from_slice(
                                &encoded[..],
                                bincode::config::standard(),
                            ) {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("bincode::decode_from_slice err: {}", e);
                                    return;
                                }
                            };
                        for heartbeat_rsp_encoded in heartbeat_rsp_encodeds {
                            if let Ok(heartbeat_rsp) = HeartbeatRsp::decode(&*heartbeat_rsp_encoded)
                            {
                                if let Err(e) = tx.send(Ok(heartbeat_rsp)).await {
                                    log::error!("tx send err: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("sled_db.get err: {}", e);
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn register(&self, req: Request<RegisterReq>) -> Result<Response<RegisterRsp>, Status> {
        let register_req = req.get_ref();
        let token = uuid::Uuid::new_v4().to_string();
        self.online_agent_cache
            .insert(register_req.agent_id.clone(), token.clone());
        log::info!("online in cache {}", register_req.agent_id);
        match tbl_agent::Entity::find_by_id(&register_req.agent_id)
            .one(&self.db_conn)
            .await
        {
            Ok(tbl_agent_op) => match tbl_agent_op {
                Some(tbl_agent) => {
                    let mut tbl_agent_am = tbl_agent.into_active_model();
                    tbl_agent_am.state = Set(AgentState::Online.to_string());
                    tbl_agent_am.version = Set(register_req.agent_version.to_string());
                    tbl_agent_am.token = Set(token.clone());
                    if let Err(e) = tbl_agent_am.save(&self.db_conn).await {
                        log::error!("tbl_agent save err: {}", e);
                        return Err(tonic::Status::new(
                            tonic::Code::Internal,
                            "tbl_agent find by id err".to_string(),
                        ));
                    }
                    log::info!("online in db {}", register_req.agent_id);
                }
                None => {
                    let tbl_agent_am = tbl_agent::ActiveModel {
                        id: Set(register_req.agent_id.to_string()),
                        version: Set(register_req.agent_version.to_string()),
                        state: Set(AgentState::Online.to_string()),
                        token: Set(token.clone()),
                        ..Default::default()
                    };
                    if let Err(e) = tbl_agent::Entity::insert(tbl_agent_am)
                        .exec(&self.db_conn)
                        .await
                    {
                        log::error!("tbl_agent save err: {}", e);
                        return Err(tonic::Status::new(
                            tonic::Code::Internal,
                            "tbl_agent find by id err".to_string(),
                        ));
                    }
                    log::info!("online in db {}", register_req.agent_id);
                }
            },
            Err(e) => {
                log::error!("tbl_agent find by id err: {}", e);
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "tbl_agent find by id err".to_string(),
                ));
            }
        };
        let register_rsp = RegisterRsp { token };
        Ok(Response::new(register_rsp))
    }

    async fn host(&self, req: Request<HostReq>) -> Result<Response<Empty>, Status> {
        let agent_id = extract_metadata_value(req.metadata(), "agent_id")?;
        let host_req = req.get_ref();

        match tbl_host::Entity::find_by_id(agent_id)
            .one(&self.db_conn)
            .await
        {
            Ok(tbl_host_op) => match tbl_host_op {
                // 数据里有
                Some(tbl_host) => {
                    if let Some(system) = &host_req.system {
                        let mut tbl_host_am = tbl_host.clone().into_active_model();
                        tbl_host_am.name = Set(system.name.clone());
                        tbl_host_am.host_name = Set(system.host_name.clone());
                        tbl_host_am.os_version = Set(system.os_version.clone());
                        tbl_host_am.cpu_arch = Set(system.cpu_arch.clone());
                        tbl_host_am.content = Set(host_req.encode_to_vec());
                        tbl_host_am.updated_at = Set(chrono::Utc::now().naive_utc());
                        if let Err(e) = tbl_host_am.save(&self.db_conn).await {
                            log::error!("tbl_host save err: {}", e);
                            return Err(tonic::Status::new(
                                tonic::Code::Internal,
                                "tbl_host save err".to_string(),
                            ));
                        }
                    } else {
                        let mut tbl_host_am = tbl_host.clone().into_active_model();
                        tbl_host_am.name = Set(None);
                        tbl_host_am.host_name = Set(None);
                        tbl_host_am.os_version = Set(None);
                        tbl_host_am.cpu_arch = Set("".to_string());
                        tbl_host_am.updated_at = Set(chrono::Utc::now().naive_utc());
                        tbl_host_am.content = Set(host_req.encode_to_vec());
                        if let Err(e) = tbl_host_am.save(&self.db_conn).await {
                            log::error!("tbl_host save err: {}", e);
                            return Err(tonic::Status::new(
                                tonic::Code::Internal,
                                "tbl_host save err".to_string(),
                            ));
                        }
                    }
                }
                // 数据库里没有
                None => {
                    if let Some(system) = &host_req.system {
                        let tbl_host_am = tbl_host::ActiveModel {
                            agent_id: Set(agent_id.to_string()),
                            name: Set(system.name.clone()),
                            host_name: Set(system.host_name.clone()),
                            os_version: Set(system.os_version.clone()),
                            cpu_arch: Set(system.cpu_arch.clone()),
                            content: Set(host_req.encode_to_vec()),
                            ..Default::default()
                        };
                        if let Err(e) = tbl_host::Entity::insert(tbl_host_am)
                            .exec(&self.db_conn)
                            .await
                        {
                            log::error!("tbl_host insert err: {}", e);
                            return Err(tonic::Status::new(
                                tonic::Code::Internal,
                                "tbl_host insert err".to_string(),
                            ));
                        }
                    } else {
                        let tbl_host_am = tbl_host::ActiveModel {
                            agent_id: Set(agent_id.to_string()),
                            name: Set(None),
                            host_name: Set(None),
                            os_version: Set(None),
                            cpu_arch: Set("".to_string()),
                            content: Set(host_req.encode_to_vec()),
                            ..Default::default()
                        };
                        if let Err(e) = tbl_host::Entity::insert(tbl_host_am)
                            .exec(&self.db_conn)
                            .await
                        {
                            log::error!("tbl_host insert err: {}", e);
                            return Err(tonic::Status::new(
                                tonic::Code::Internal,
                                "tbl_host insert err".to_string(),
                            ));
                        }
                    }
                }
            },
            Err(e) => {
                log::error!("tbl_host find by id err: {}", e);
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "tbl_host find by id err".to_string(),
                ));
            }
        }
        log::info!("save host success");
        Ok(Response::new(Empty {}))
    }
}
