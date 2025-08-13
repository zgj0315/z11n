use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
};
use entity::tbl_auth_role;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::{
    AppState,
    auth::{RESTFUL_APIS, RestfulApi},
};

pub fn routers(state: AppState) -> Router {
    Router::new()
        .route("/roles", get(role_query).post(role_create))
        .route(
            "/roles/{id}",
            patch(role_update).get(role_detail).delete(role_delete),
        )
        .route("/restful_apis", get(restful_apis))
        .with_state(state)
}

#[derive(Deserialize, Debug, Validate)]
struct RoleQueryInputDto {
    name: Option<String>,
    size: u64,
    page: u64,
}

#[derive(Serialize, Debug)]
pub struct RoleQueryOutputDto {
    pub id: i32,
    pub name: String,
    pub restful_apis: Vec<RestfulApi>,
}

async fn role_query(
    app_state: State<AppState>,
    Query(query_input_dto): Query<RoleQueryInputDto>,
) -> impl IntoResponse {
    let mut select = tbl_auth_role::Entity::find();
    if let Some(name) = query_input_dto.name {
        if !name.is_empty() {
            let like_pattern = format!("%{name}%");
            select = select.filter(tbl_auth_role::Column::Name.like(like_pattern));
        }
    }

    let paginator = select
        .order_by_desc(tbl_auth_role::Column::Id)
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
    let tbl_auth_roles = match paginator.fetch_page(query_input_dto.page).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("fetch_page err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut roles = Vec::new();
    for tbl_auth_role in tbl_auth_roles {
        let encoded = tbl_auth_role.apis;
        let (restful_apis, _len): (Vec<RestfulApi>, usize) =
            match bincode::decode_from_slice(&encoded[..], bincode::config::standard()) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("bincode::decode_from_slice err: {}", e);
                    continue;
                }
            };

        roles.push(RoleQueryOutputDto {
            id: tbl_auth_role.id,
            name: tbl_auth_role.name,
            restful_apis: restful_apis,
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
                "role":roles
            }
           }
        )),
    )
        .into_response()
}

#[derive(Deserialize, Debug, Validate)]
struct RoleCreateInputDto {
    name: String,
    restful_apis: Vec<RestfulApi>,
}
async fn role_create(
    app_state: State<AppState>,
    Json(create_input_dto): Json<RoleCreateInputDto>,
) -> impl IntoResponse {
    let encoded: Vec<u8> =
        match bincode::encode_to_vec(&create_input_dto.restful_apis, bincode::config::standard()) {
            Ok(v) => v,
            Err(e) => {
                log::error!("bincode::encode_to_vec err: {}", e);
                return StatusCode::BAD_REQUEST.into_response();
            }
        };
    let tbl_auth_role_am = tbl_auth_role::ActiveModel {
        name: Set(create_input_dto.name),
        apis: Set(encoded),
        ..Default::default()
    };
    match tbl_auth_role::Entity::insert(tbl_auth_role_am)
        .exec(&app_state.db_conn)
        .await
    {
        Ok(_) => {
            return StatusCode::OK.into_response();
        }
        Err(e) => {
            log::error!("tbl_auth_role insert err: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize, Debug, Validate)]
struct RoleUpdateInputDto {
    name: String,
    restful_apis: Vec<RestfulApi>,
}
async fn role_update(
    Path(id): Path<i32>,
    app_state: State<AppState>,
    Json(update_input_dto): Json<RoleUpdateInputDto>,
) -> impl IntoResponse {
    let tbl_auth_role = match tbl_auth_role::Entity::find_by_id(id)
        .one(&app_state.db_conn)
        .await
    {
        Ok(op) => match op {
            Some(v) => v,
            None => {
                log::error!("tbl_auth_role {} not exist", id);
                return StatusCode::BAD_REQUEST.into_response();
            }
        },
        Err(e) => {
            log::error!("tbl_auth_role find by id err: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };
    let encoded: Vec<u8> =
        match bincode::encode_to_vec(&update_input_dto.restful_apis, bincode::config::standard()) {
            Ok(v) => v,
            Err(e) => {
                log::error!("bincode::encode_to_vec err: {}", e);
                return StatusCode::BAD_REQUEST.into_response();
            }
        };
    let mut tbl_auth_role_am = tbl_auth_role.into_active_model();
    tbl_auth_role_am.name = Set(update_input_dto.name);
    tbl_auth_role_am.apis = Set(encoded);

    match tbl_auth_role_am.save(&app_state.db_conn).await {
        Ok(_) => {
            return StatusCode::OK.into_response();
        }
        Err(e) => {
            log::error!("tbl_auth_role save err: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn role_detail(Path(id): Path<i32>, State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_auth_role::Entity::find_by_id(id)
        .one(&app_state.db_conn)
        .await
    {
        Ok(tbl_agent_op) => match tbl_agent_op {
            Some(tbl_auth_role) => {
                let (restful_apis, _len): (Vec<RestfulApi>, usize) =
                    match bincode::decode_from_slice(
                        &tbl_auth_role.apis[..],
                        bincode::config::standard(),
                    ) {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("bincode::decode_from_slice err: {}", e);
                            return StatusCode::BAD_REQUEST.into_response();
                        }
                    };

                (
                    StatusCode::OK,
                    Json(json!({
                        "id":tbl_auth_role.id,
                        "name":tbl_auth_role.name,
                        "restful_apis":restful_apis,
                    })),
                )
                    .into_response()
            }
            None => StatusCode::BAD_REQUEST.into_response(),
        },
        Err(e) => {
            log::error!("find agent {} db err: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn role_delete(Path(id): Path<i32>, State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_auth_role::Entity::delete_by_id(id)
        .exec(&app_state.db_conn)
        .await
    {
        Ok(delete_result) => {
            if delete_result.rows_affected == 1 {
                log::info!("tbl_auth_role {} delete success", id)
            } else {
                log::warn!(
                    "tbl_auth_role {} delete success, but affect row: {}",
                    id,
                    delete_result.rows_affected
                )
            }
            StatusCode::OK.into_response()
        }
        Err(e) => {
            log::error!("find agent {} db err: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn restful_apis() -> impl IntoResponse {
    let restful_apis = RESTFUL_APIS.clone();
    match serde_json::to_value(&restful_apis) {
        Ok(v) => {
            return (StatusCode::OK, Json(v)).into_response();
        }
        Err(e) => {
            log::error!("RESTFUL_APIS to value err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
}
