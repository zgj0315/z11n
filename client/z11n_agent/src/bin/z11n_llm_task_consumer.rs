use z11n_agent::{agent_register, heartbeat};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("llm task consumer starting");
    agent_register().await?;

    heartbeat().await?;
    Ok(())
}
