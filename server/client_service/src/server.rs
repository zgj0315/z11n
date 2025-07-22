use crate::proto::{
    HeartbeatReq, HeartbeatRsp, RegisterReq, RegisterRsp, z11n_service_server::Z11nService,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status, metadata::MetadataMap, service::Interceptor};

#[derive(Debug, Clone)]
pub struct Z11nInterceptor {}

impl Interceptor for Z11nInterceptor {
    fn call(&mut self, req: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        let agent_id = extract_metadata_value(req.metadata(), "agent_id")?;
        let agent_version = extract_metadata_value(req.metadata(), "agent_version")?;
        log::info!("agent_id: {agent_id}, agent_version: {agent_version}");
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

#[derive(Debug, Default)]
pub struct Z11nServer {}

#[tonic::async_trait]
impl Z11nService for Z11nServer {
    type HeartbeatStream = ReceiverStream<Result<HeartbeatRsp, Status>>;
    async fn heartbeat(
        &self,
        req: Request<HeartbeatReq>,
    ) -> Result<Response<Self::HeartbeatStream>, Status> {
        let token = extract_metadata_value(req.metadata(), "token")?;
        let (tx, rx) = mpsc::channel(10);
        let heartbeat_req = req.into_inner();
        log::info!("heartbeat_req: {:?}", heartbeat_req);
        let cmd_content = format!(
            "agent_id: {}, agent_type: {}",
            heartbeat_req.agent_id.to_owned(),
            heartbeat_req.agent_type.to_owned()
        );
        tokio::spawn(async move {
            for i in 0..2 {
                let response = HeartbeatRsp {
                    cmd_type: i % 2,
                    cmd_content: cmd_content.clone(),
                };
                log::info!("send response: {:?}", response);
                tx.send(Ok(response)).await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn register(&self, req: Request<RegisterReq>) -> Result<Response<RegisterRsp>, Status> {
        let token = uuid::Uuid::new_v4().to_string();
        let register_rsp = RegisterRsp { token };
        Ok(Response::new(register_rsp))
    }
}
