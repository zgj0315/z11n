pub mod agent;
pub mod config;
pub mod server;
pub mod uds;
pub mod proto {
    tonic::include_proto!("z11n");
}
