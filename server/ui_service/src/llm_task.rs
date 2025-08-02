use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use entity::tbl_llm_task;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::AppState;

pub fn routers(state: AppState) -> Router {
    Router::new()
        .route("/llm_tasks", get(query))
        .route("/llm_tasks/{id}", get(detail))
        .with_state(state)
}

#[derive(Deserialize, Debug, Validate)]
struct QueryInputDto {
    model: Option<String>,
    prompt: Option<String>,
    req_content: Option<String>,
    rsp_content: Option<String>,
    size: u64,
    page: u64,
}

#[derive(Serialize, Debug)]
struct QueryOutputDto {
    id: String,
    model: String,
    prompt: String,
    req_content: String,
    req_push_at: i64,
    req_pull_at: Option<i64>,
    rsp_content: Option<String>,
    rsp_push_at: Option<i64>,
    rsp_pull_at: Option<i64>,
}
async fn query(
    app_state: State<AppState>,
    Query(query_input_dto): Query<QueryInputDto>,
) -> impl IntoResponse {
    let mut select = tbl_llm_task::Entity::find();
    if let Some(v) = query_input_dto.model {
        if !v.is_empty() {
            let like_pattern = format!("%{v}%");
            select = select.filter(tbl_llm_task::Column::Model.like(like_pattern));
        }
    }
    if let Some(v) = query_input_dto.prompt {
        if !v.is_empty() {
            let like_pattern = format!("%{v}%");
            select = select.filter(tbl_llm_task::Column::Prompt.like(like_pattern));
        }
    }
    if let Some(v) = query_input_dto.req_content {
        if !v.is_empty() {
            let like_pattern = format!("%{v}%");
            select = select.filter(tbl_llm_task::Column::ReqContent.like(like_pattern));
        }
    }
    if let Some(v) = query_input_dto.rsp_content {
        if !v.is_empty() {
            let like_pattern = format!("%{v}%");
            select = select.filter(tbl_llm_task::Column::RspContent.like(like_pattern));
        }
    }
    let paginator = select
        .order_by_desc(tbl_llm_task::Column::ReqPushAt)
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
    let tbl_llm_tasks = match paginator.fetch_page(query_input_dto.page).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("fetch_page err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut llm_tasks = Vec::new();
    for tbl_llm_task in tbl_llm_tasks {
        let req_pull_at = match tbl_llm_task.req_pull_at {
            Some(v) => Some(v.and_utc().timestamp_millis()),
            None => None,
        };
        let rsp_push_at = match tbl_llm_task.rsp_push_at {
            Some(v) => Some(v.and_utc().timestamp_millis()),
            None => None,
        };
        let rsp_pull_at = match tbl_llm_task.rsp_pull_at {
            Some(v) => Some(v.and_utc().timestamp_millis()),
            None => None,
        };
        let rsp_content = match tbl_llm_task.rsp_content {
            Some(v) => Some(v),
            None => None,
        };
        llm_tasks.push(QueryOutputDto {
            id: tbl_llm_task.id,
            model: tbl_llm_task.model,
            prompt: tbl_llm_task.prompt,
            req_content: tbl_llm_task.req_content,
            req_push_at: tbl_llm_task.req_push_at.and_utc().timestamp_millis(),
            req_pull_at,
            rsp_content,
            rsp_push_at,
            rsp_pull_at,
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
                "llm_task":llm_tasks
            }
           }
        )),
    )
        .into_response()
}

async fn detail(Path(id): Path<String>, State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_llm_task::Entity::find_by_id(&id)
        .one(&app_state.db_conn)
        .await
    {
        Ok(op) => match op {
            Some(tbl_llm_task) => {
                let req_pull_at = match tbl_llm_task.req_pull_at {
                    Some(v) => Some(v.and_utc().timestamp_millis()),
                    None => None,
                };
                let rsp_push_at = match tbl_llm_task.rsp_push_at {
                    Some(v) => Some(v.and_utc().timestamp_millis()),
                    None => None,
                };
                let rsp_pull_at = match tbl_llm_task.rsp_pull_at {
                    Some(v) => Some(v.and_utc().timestamp_millis()),
                    None => None,
                };
                (
                    StatusCode::OK,
                    Json(json!({
                        "id":tbl_llm_task.id,
                        "model":tbl_llm_task.model,
                        "prompt":tbl_llm_task.prompt,
                        "req_content":tbl_llm_task.req_content,
                        "req_push_at":tbl_llm_task.req_push_at,
                        "req_pull_at":req_pull_at,
                        "rsp_content":tbl_llm_task.rsp_content,
                        "rsp_push_at":rsp_push_at,
                        "rsp_pull_at":rsp_pull_at
                    })),
                )
                    .into_response()
            }
            None => StatusCode::BAD_REQUEST.into_response(),
        },
        Err(e) => {
            log::error!("find llm_task {} db err: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
