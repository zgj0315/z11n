use crate::proto::{HeartbeatReq, HeartbeatRsp, z11n_service_server::Z11nService};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, service::Interceptor};

#[derive(Debug, Clone)]
pub struct Z11nInterceptor {}

impl Interceptor for Z11nInterceptor {
    fn call(&mut self, req: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        // // 出于性能考虑，这里尽可能少干活
        // match req.metadata().get("agent_id") {
        //     Some(v) => {
        //         if v.is_empty() {
        //             log::warn!("agent_id is empty");
        //             return Err(tonic::Status::new(
        //                 tonic::Code::Internal,
        //                 "agent_id is empty",
        //             ));
        //         }
        //         if let Err(e) = v.to_str() {
        //             log::warn!("agent_id to_str err: {}", e);
        //             return Err(tonic::Status::new(
        //                 tonic::Code::Internal,
        //                 "agent_id to_str err",
        //             ));
        //         }
        //     }
        //     None => {
        //         log::warn!("agent_id not exist");
        //         return Err(tonic::Status::new(
        //             tonic::Code::Internal,
        //             "agent_id not exist",
        //         ));
        //     }
        // }
        // match req.metadata().get("agent_version") {
        //     Some(v) => {
        //         if v.is_empty() {
        //             log::warn!("agent_version is empty");
        //             return Err(tonic::Status::new(
        //                 tonic::Code::Internal,
        //                 "agent_version is empty",
        //             ));
        //         }
        //         if let Err(e) = v.to_str() {
        //             log::warn!("agent_version to_str err: {}", e);
        //             return Err(tonic::Status::new(
        //                 tonic::Code::Internal,
        //                 "agent_version to_str err",
        //             ));
        //         }
        //     }
        //     None => {
        //         log::warn!("agent_version not exist");
        //         return Err(tonic::Status::new(
        //             tonic::Code::Internal,
        //             "agent_version not exist",
        //         ));
        //     }
        // };
        Ok(req)
    }
}

#[derive(Debug, Default)]
pub struct Z11nServer {}

#[tonic::async_trait]
impl Z11nService for Z11nServer {
    type HeartbeatStream = ReceiverStream<Result<HeartbeatRsp, Status>>;
    async fn heartbeat(
        &self,
        request: Request<HeartbeatReq>,
    ) -> Result<Response<Self::HeartbeatStream>, Status> {
        let (tx, rx) = mpsc::channel(10);
        let heartbeat_req = request.into_inner();
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
}
