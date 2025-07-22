use crate::config::CLIENT_SERVICE_TOML;
use entity::tbl_agent;
use moka::{notification::RemovalCause, sync::Cache};
use pub_lib::AgentState;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QuerySelect, prelude::Expr,
};
use std::{collections::HashSet, sync::Arc, time::Duration};

pub async fn init_cache(
    db_conn: &sea_orm::DatabaseConnection,
) -> anyhow::Result<Cache<String, String>> {
    let db_conn_clone = db_conn.clone();
    let eviction_listener = move |agent_id: Arc<String>, _token: String, removal_cause| {
        let db_conn_clone = db_conn_clone.clone();
        match removal_cause {
            RemovalCause::Expired => {
                // cache 到达ttl时，会触发此逻辑
                tokio::spawn(async move {
                    if let Err(e) = agent_offline(&agent_id, db_conn_clone.clone()).await {
                        log::error!("{} agent_offline err: {}", agent_id, e);
                    }
                });
            }
            RemovalCause::Explicit => {
                log::warn!("explicit");
            }
            RemovalCause::Replaced => {
                // log::info!("{:?} is alive", app_info);
            }
            RemovalCause::Size => {
                log::warn!("size");
            }
        }
    };
    let cache = Cache::builder()
        .max_capacity(50_000)
        .time_to_live(Duration::from_secs(
            CLIENT_SERVICE_TOML.agent.offline_ex as u64,
        ))
        .eviction_listener(eviction_listener)
        .build();
    sync_pg_and_cache_task(db_conn.clone(), cache.clone()).await?;
    Ok(cache)
}

async fn agent_offline(agent_id: &str, db_conn: sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    if let Some(tbl_agent) = tbl_agent::Entity::find()
        .filter(tbl_agent::Column::Id.eq(agent_id))
        .one(&db_conn)
        .await?
    {
        let state = tbl_agent.state.clone();
        if !state.eq(&AgentState::Online.to_string()) {
            let mut tbl_agent_am = tbl_agent.into_active_model();
            tbl_agent_am.state = Set(AgentState::Offline.to_string());
            tbl_agent_am.save(&db_conn).await?;
            log::info!("{} offline in db", agent_id);
        }
    }
    Ok(())
}

async fn sync_pg_and_cache_task(
    db_conn: sea_orm::DatabaseConnection,
    cache: Cache<String, String>,
) -> anyhow::Result<()> {
    // 将pg中的在线主机load到内存，然后启动任务
    sync_pg_and_cache(db_conn.clone(), cache.clone()).await?;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600));
        loop {
            interval.tick().await;
            if let Err(e) = sync_pg_and_cache(db_conn.clone(), cache.clone()).await {
                log::error!("sync_pg_and_cache err: {}", e);
            };
        }
    });
    Ok(())
}

async fn sync_pg_and_cache(
    db_conn: sea_orm::DatabaseConnection,
    cache: Cache<String, String>,
) -> anyhow::Result<()> {
    log::info!("sync_db_and_cache begin");
    if let Ok(agent_id_tokens) = tbl_agent::Entity::find()
        .select_only()
        .column(tbl_agent::Column::Id)
        .column(tbl_agent::Column::Token)
        .filter(tbl_agent::Column::State.eq(AgentState::Online.to_string()))
        .into_tuple::<(String, String)>()
        .all(&db_conn)
        .await
    {
        let mut agent_id_set = HashSet::new();
        for (agent_id, token) in &agent_id_tokens {
            agent_id_set.insert(agent_id.clone());
            if !cache.contains_key(agent_id) {
                cache.insert(agent_id.to_string(), token.clone());
                log::info!("{} online in cache", agent_id);
            }
        }
        for (agent_id, _) in cache.iter() {
            if !agent_id_set.contains(agent_id.as_str()) {
                tbl_agent::Entity::update_many()
                    .col_expr(
                        tbl_agent::Column::State,
                        Expr::value(AgentState::Online.to_string()),
                    )
                    .filter(tbl_agent::Column::Id.eq(agent_id.as_str()))
                    .exec(&db_conn)
                    .await?;
                log::info!("{} online in db by sync_pg_and_cache", agent_id);
            }
        }
    }
    log::info!("sync_db_and_cache end");
    Ok(())
}
