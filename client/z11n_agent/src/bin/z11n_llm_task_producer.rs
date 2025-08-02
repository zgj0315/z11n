use z11n_agent::{
    agent_register, build_client,
    config::Z11N_AGENT_TOML,
    heartbeat,
    proto::{Empty, LlmTaskQuestionReq},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("./config/log4rs.yml", Default::default())?;
    log::info!("llm task producer starting");
    agent_register().await?;
    tokio::spawn(async move {
        loop {
            if let Err(e) = push_llm_task_question().await {
                log::error!("push_llm_task_question err: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });
    tokio::spawn(async move {
        loop {
            if let Err(e) = pull_llm_task_answer().await {
                log::error!("pull_llm_task_answer err: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
    });
    heartbeat().await?;
    Ok(())
}

async fn pull_llm_task_answer() -> anyhow::Result<()> {
    let mut client = build_client(&Z11N_AGENT_TOML.server.addr).await?;
    let empty = Empty {};
    match client.pull_llm_task_answer(empty).await {
        Ok(rsp) => {
            for llm_task_answer in &rsp.get_ref().items {
                log::info!(
                    "get task_id: {} answer: {}",
                    llm_task_answer.id,
                    llm_task_answer.content
                );
            }
        }
        Err(e) => {
            log::error!("client.pull_llm_task_answer err: {}", e);
        }
    }
    Ok(())
}
async fn push_llm_task_question() -> anyhow::Result<()> {
    let llm_task_question_req = LlmTaskQuestionReq {
        model: "gemma3".to_string(),
        prompt: "你是一个资深的Rust程序员专家".to_string(),
        content: "如何遍历一个enum".to_string(),
    };
    let mut client = build_client(&Z11N_AGENT_TOML.server.addr).await?;
    let rsp = client.push_llm_task_question(llm_task_question_req).await?;
    let task_id = rsp.get_ref().id.clone();
    log::info!("push_llm_task_question task id: {task_id}");
    Ok(())
}
