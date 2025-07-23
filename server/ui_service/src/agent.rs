use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use entity::tbl_agent;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::AppState;

pub fn routers(state: AppState) -> Router {
    Router::new()
        .route("/agents", get(query))
        .route("/agents/{id}", get(detail).delete(delete))
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
    id: String,
    version: String,
    created_at: i64,
    updated_at: i64,
}
async fn query(
    app_state: State<AppState>,
    Query(query_input_dto): Query<QueryInputDto>,
) -> impl IntoResponse {
    let mut select = tbl_agent::Entity::find();
    if let Some(ip) = query_input_dto.ip {
        if !ip.is_empty() {
            let like_pattern = format!("%{ip}%");
            select = select.filter(tbl_agent::Column::Id.like(like_pattern));
        }
    }

    let paginator = select
        .order_by_desc(tbl_agent::Column::CreatedAt)
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
    let tbl_agents = match paginator.fetch_page(query_input_dto.page).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("fetch_page err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut agents = Vec::new();
    for tbl_agent in tbl_agents {
        agents.push(QueryOutputDto {
            id: tbl_agent.id,
            version: tbl_agent.version,
            created_at: tbl_agent.created_at.and_utc().timestamp_millis(),
            updated_at: tbl_agent.created_at.and_utc().timestamp_millis(),
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
                "agent":agents
            }
           }
        )),
    )
        .into_response()
}

async fn detail(Path(id): Path<String>, State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_agent::Entity::find_by_id(&id)
        .one(&app_state.db_conn)
        .await
    {
        Ok(tbl_agent_op) => match tbl_agent_op {
            Some(tbl_agent) => {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "agent_id":tbl_agent.id,
                        "agent_version":tbl_agent.version,
                        "created_at":tbl_agent.created_at.and_utc().timestamp_millis()
                    })),
                )
                    .into_response();
            }
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
    match tbl_agent::Entity::delete_by_id(&id)
        .exec(&app_state.db_conn)
        .await
    {
        Ok(delete_result) => {
            if delete_result.rows_affected == 1 {
                log::info!("delete agent {id} success");
            } else {
                log::warn!(
                    "delete agent {id} success, affected row: {}",
                    delete_result.rows_affected
                );
            }
            return StatusCode::OK;
        }
        Err(e) => {
            log::error!("delete agent {id} db err: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
}
