use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use moka::{notification::RemovalCause, sync::Cache};

use crate::config::CLIENT_SERVICE_TOML;

pub async fn init_cache(
    pg_conn: sea_orm::DatabaseConnection,
) -> anyhow::Result<Cache<String, DateTime<Utc>>> {
    let pg_conn_clone = pg_conn.clone();
    let eviction_listener = move |agent_id: Arc<String>, _dt: DateTime<Utc>, removal_cause| {
        let pg_conn_clone = pg_conn_clone.clone();
        match removal_cause {
            RemovalCause::Expired => {
                // cache 到达ttl时，会触发此逻辑
                tokio::spawn(async move {
                    if let Err(e) = agent_offline(&agent_id, pg_conn_clone.clone()).await {
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
    sync_pg_and_cache_task(pg_conn, cache.clone()).await?;
    Ok(cache)
}

async fn agent_offline(agent_id: &str, pg_conn: sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    // if let Some(kl_host) = kl_host::Entity::find()
    //     .filter(kl_host::Column::HostId.eq(agent_id))
    //     .one(&pg_conn)
    //     .await?
    // {
    //     let host_state = kl_host.host_state.clone().unwrap_or_default();
    //     if !host_state.eq(HostState::OFFLINE.value) {
    //         let mut kl_host_am = kl_host.into_active_model();
    //         kl_host_am.host_state = Set(Some(HostState::OFFLINE.value.to_string()));
    //         kl_host_am.modify_time = Set(Some(chrono::Utc::now().naive_utc()));
    //         kl_host_am.save(&pg_conn).await?;
    //         log::info!("{} offline in pg", agent_id);
    //         (kl_system_log::ActiveModel {
    //             log_id: Set(uuid::Uuid::new_v4().to_string()),
    //             log_type: Set(Some(SystemLogType::AGENT_STATE_ONLINE.value.to_string())),
    //             log_level: Set(Some(Level::LOW.value.to_string())),
    //             log_content: Set(Some(format!(
    //                 "主机下线，主机agent_id: {:?}",
    //                 agent_id.to_string()
    //             ))),
    //             create_time: Set(Some(chrono::Utc::now().naive_utc())),
    //         })
    //         .insert(&pg_conn)
    //         .await?;
    //     }
    // }
    Ok(())
}

async fn sync_pg_and_cache_task(
    pg_conn: sea_orm::DatabaseConnection,
    cache: Cache<String, DateTime<Utc>>,
) -> anyhow::Result<()> {
    // 将pg中的在线主机load到内存，然后启动任务
    sync_pg_and_cache(pg_conn.clone(), cache.clone()).await?;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600));
        loop {
            interval.tick().await;
            if let Err(e) = sync_pg_and_cache(pg_conn.clone(), cache.clone()).await {
                log::error!("sync_pg_and_cache err: {}", e);
            };
        }
    });
    Ok(())
}

async fn sync_pg_and_cache(
    pg_conn: sea_orm::DatabaseConnection,
    cache: Cache<String, DateTime<Utc>>,
) -> anyhow::Result<()> {
    log::info!("sync_pg_and_cache begin");
    // if let Ok(agent_ids) = kl_host::Entity::find()
    //     .select_only()
    //     .column(kl_host::Column::HostId)
    //     .filter(kl_host::Column::HostState.eq(HostState::ONLINE.value.to_string()))
    //     .into_tuple::<String>()
    //     .all(&pg_conn)
    //     .await
    // {
    //     for agent_id in &agent_ids {
    //         if !cache.contains_key(agent_id) {
    //             cache.insert(agent_id.to_string(), chrono::Utc::now());
    //             log::info!("{} online in cache", agent_id);
    //         }
    //     }
    //     for (agent_id, _) in cache.iter() {
    //         if !agent_ids.contains(&agent_id) {
    //             kl_host::Entity::update_many()
    //                 .col_expr(
    //                     kl_host::Column::HostState,
    //                     Expr::value(HostState::ONLINE.value.to_string()),
    //                 )
    //                 .col_expr(
    //                     kl_host::Column::ModifyTime,
    //                     Expr::value(chrono::Utc::now().naive_utc()),
    //                 )
    //                 .filter(kl_host::Column::HostId.eq(agent_id.as_str()))
    //                 .exec(&pg_conn)
    //                 .await?;
    //             log::info!("{} online in pg by sync_pg_and_cache", agent_id);
    //         }
    //     }
    // }
    log::info!("sync_pg_and_cache end");
    Ok(())
}
