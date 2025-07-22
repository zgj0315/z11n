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
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub addr: String,
}
