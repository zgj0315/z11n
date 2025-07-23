use crate::{AppState, z11n::HostReq};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use entity::tbl_host;
use prost::Message;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

pub fn routers(state: AppState) -> Router {
    Router::new()
        .route("/hosts", get(query))
        .route("/hosts/{id}", get(detail).delete(delete))
        .with_state(state)
}

#[derive(Deserialize, Debug, Validate)]
struct QueryInputDto {
    ip: Option<String>,
    size: u64,
    page: u64,
}

#[derive(Serialize, Debug)]
struct QueryOutputDto {
    agent_id: String,
    name: Option<String>,
    host_name: Option<String>,
    os_version: Option<String>,
    cpu_arch: String,
    created_at: i64,
    updated_at: i64,
}
async fn query(
    app_state: State<AppState>,
    Query(query_input_dto): Query<QueryInputDto>,
) -> impl IntoResponse {
    let mut select = tbl_host::Entity::find();
    if let Some(ip) = query_input_dto.ip {
        if !ip.is_empty() {
            let like_pattern = format!("%{ip}%");
            select = select.filter(tbl_host::Column::AgentId.like(like_pattern));
        }
    }

    let paginator = select
        .order_by_desc(tbl_host::Column::CreatedAt)
        .paginate(&app_state.db_conn, query_input_dto.size);
    let num_pages = match paginator.num_pages().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("num_pages err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let num_items = match paginator.num_items().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("num_items err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let tbl_hosts = match paginator.fetch_page(query_input_dto.page).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("fetch_page err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut hosts = Vec::new();
    for tbl_host in tbl_hosts {
        hosts.push(QueryOutputDto {
            agent_id: tbl_host.agent_id,
            name: tbl_host.name,
            host_name: tbl_host.host_name,
            os_version: tbl_host.os_version,
            cpu_arch: tbl_host.cpu_arch,
            created_at: tbl_host.created_at.and_utc().timestamp_millis(),
            updated_at: tbl_host.updated_at.and_utc().timestamp_millis(),
        });
    }
    (
        StatusCode::OK,
        Json(json!(
            {
            "page":{
              "size":query_input_dto.size,
              "total_elements":num_items,
              "total_pages":num_pages
            },
            "_embedded":{
                "host":hosts
            }
           }
        )),
    )
        .into_response()
}

async fn detail(Path(id): Path<String>, State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_host::Entity::find_by_id(&id)
        .one(&app_state.db_conn)
        .await
    {
        Ok(tbl_host_op) => match tbl_host_op {
            Some(tbl_host) => match HostReq::decode(&*tbl_host.content) {
                Ok(host_req) => {
                    if let Some(system) = host_req.system {
                        return (
                            StatusCode::OK,
                            Json(json!({
                                "agent_id":id,
                                "name":system.name,
                                "kernel_version":system.kernel_version,
                                "name":system.name,
                                "os_version":system.os_version,
                                "host_name":system.host_name,
                                "cpu_arch":system.cpu_arch,
                                "kernel_long_version":system.kernel_long_version,
                                "total_memory":system.total_memory,
                                "total_swap":system.total_swap,
                                "created_at":tbl_host.created_at.and_utc().timestamp_millis()
                            })),
                        )
                            .into_response();
                    } else {
                        log::warn!("host detail not exist");
                        return StatusCode::BAD_REQUEST.into_response();
                    }
                }
                Err(e) => {
                    log::error!("HostReq decode err: {}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            },
            None => {
                return StatusCode::BAD_REQUEST.into_response();
            }
        },
        Err(e) => {
            log::error!("find agent {} db err: {}", id, e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
}

async fn delete(Path(id): Path<String>, State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_host::Entity::delete_by_id(&id)
        .exec(&app_state.db_conn)
        .await
    {
        Ok(delete_result) => {
            if delete_result.rows_affected == 1 {
                log::info!("delete host {id} success");
            } else {
                log::warn!(
                    "delete host {id} success, affected row: {}",
                    delete_result.rows_affected
                );
            }
            return StatusCode::OK;
        }
        Err(e) => {
            log::error!("delete host {id} db err: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
}
