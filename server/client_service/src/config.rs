use once_cell::sync::Lazy;
use serde::Deserialize;

pub static CLIENT_SERVICE_TOML: Lazy<ServerToml> = Lazy::new(|| {
    config::Config::builder()
        .add_source(config::File::with_name("./config/client_service.toml"))
        .build()
        .unwrap()
        .try_deserialize::<ServerToml>()
        .unwrap()
});

#[derive(Debug, Deserialize)]
pub struct ServerToml {
    pub server: Server,
    pub agent: Agent,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub addr: String,
}

#[derive(Debug, Deserialize)]
pub struct Agent {
    pub heartbeat_delay: i32,
    pub offline_ex: i64,
}
