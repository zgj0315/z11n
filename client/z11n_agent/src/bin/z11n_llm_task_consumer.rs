use z11n_agent::{
    agent_register, build_client,
    config::Z11N_AGENT_TOML,
    heartbeat,
    proto::{Empty, LlmTaskAnswer},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("llm task consumer starting");
    agent_register().await?;

    let mut client = build_client(&Z11N_AGENT_TOML.server.addr).await?;
    let rsp = client.pull_llm_task_question(Empty {}).await?;
    while let Some(llm_task_question) = &rsp.get_ref().llm_task_question {
        log::info!("llm_task_question: {llm_task_question:?}");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let llm_task_answer = LlmTaskAnswer {
            id: llm_task_question.id.clone(),
            content: "this is answer".to_string(),
        };
        let rsp = client.push_llm_task_answer(llm_task_answer).await?;
        log::info!("rsp: {rsp:?}");
    }

    heartbeat().await?;
    Ok(())
}
