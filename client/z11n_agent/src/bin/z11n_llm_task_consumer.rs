use z11n_agent::{
    agent_register, build_client,
    config::Z11N_AGENT_TOML,
    heartbeat,
    proto::{Empty, LlmTaskAnswer, LlmTaskQuestion},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("llm task consumer starting");
    agent_register().await?;
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    tokio::spawn(async move {
        loop {
            let tx_clone = tx.clone();
            if let Err(e) = pull_llm_task_question(tx_clone).await {
                log::error!("pull_llm_task_question err: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
    tokio::spawn(async move {
        if let Err(e) = push_llm_task_answer(rx).await {
            log::error!("push_llm_task_answer err: {}", e);
        }
    });
    heartbeat().await?;
    Ok(())
}

async fn push_llm_task_answer(
    mut rx: tokio::sync::mpsc::Receiver<LlmTaskQuestion>,
) -> anyhow::Result<()> {
    while let Some(llm_task_question) = rx.recv().await {
        let mut client = build_client(&Z11N_AGENT_TOML.server.addr).await?;
        let llm_task_answer = LlmTaskAnswer {
            id: llm_task_question.id.clone(),
            content: "this is answer".to_string(),
        };
        log::info!("push_llm_task_answer task: {}", llm_task_question.id);
        let rsp = client.push_llm_task_answer(llm_task_answer).await?;
        log::info!("rsp: {rsp:?}");
    }
    Ok(())
}

async fn pull_llm_task_question(
    tx: tokio::sync::mpsc::Sender<LlmTaskQuestion>,
) -> anyhow::Result<()> {
    let mut client = build_client(&Z11N_AGENT_TOML.server.addr).await?;
    let rsp = client.pull_llm_task_question(Empty {}).await?;
    if let Some(llm_task_question) = &rsp.get_ref().llm_task_question {
        log::info!("llm_task_question: {llm_task_question:?}");
        tx.send(llm_task_question.clone()).await?;
    }
    Ok(())
}
