use once_cell::sync::Lazy;
use serde::Deserialize;

pub static Z11N_AGENT_TOML: Lazy<ServerToml> = Lazy::new(|| {
    config::Config::builder()
        .add_source(config::File::with_name("./config/z11n_agent.toml"))
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
