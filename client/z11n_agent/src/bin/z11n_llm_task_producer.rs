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
    let llm_task_question_req = LlmTaskQuestionReq {
        model: "this is model".to_string(),
        prompt: "this is prompt".to_string(),
        content: "this is content".to_string(),
    };
    let mut client = build_client(&Z11N_AGENT_TOML.server.addr).await?;
    let rsp = client.push_llm_task_question(llm_task_question_req).await?;
    let task_id = rsp.get_ref().id.clone();
    tokio::spawn(async move {
        loop {
            let empty = Empty {};
            match client.pull_llm_task_answer(empty).await {
                Ok(rsp) => {
                    for llm_task_answer in &rsp.get_ref().items {
                        log::info!("get task_id: {task_id} answer: {}", llm_task_answer.content);
                        break;
                    }
                }
                Err(e) => {
                    log::error!("client.pull_llm_task_answer err: {}", e);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
    });
    heartbeat().await?;
    Ok(())
}
