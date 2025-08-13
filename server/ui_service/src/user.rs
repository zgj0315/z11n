use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
};
use entity::{tbl_auth_role, tbl_auth_user, tbl_auth_user_role};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::role::RoleQueryOutputDto;
use crate::{AppState, auth::RestfulApi};

pub fn routers(state: AppState) -> Router {
    Router::new()
        .route("/users", get(user_query).post(user_create))
        .route("/users/{id}", patch(user_update).get(user_detail))
        .with_state(state)
}

#[derive(Deserialize, Debug, Validate)]
struct UserQueryInputDto {
    username: Option<String>,
    size: u64,
    page: u64,
}

#[derive(Serialize, Debug)]
struct UserQueryOutputDto {
    id: i32,
    username: String,
    roles: Vec<RoleQueryOutputDto>,
}

async fn user_query(
    app_state: State<AppState>,
    Query(query_input_dto): Query<UserQueryInputDto>,
) -> impl IntoResponse {
    let mut select = tbl_auth_user::Entity::find();
    if let Some(username) = query_input_dto.username {
        if !username.is_empty() {
            let like_pattern = format!("%{username}%");
            select = select.filter(tbl_auth_user::Column::Username.like(like_pattern));
        }
    }

    let paginator = select
        .order_by_desc(tbl_auth_user::Column::Id)
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
    let tbl_auth_users = match paginator.fetch_page(query_input_dto.page).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("fetch_page err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut users = Vec::new();
    for tbl_auth_user in tbl_auth_users {
        let role_ids = match tbl_auth_user_role::Entity::find()
            .select_only()
            .column(tbl_auth_user_role::Column::RoleId)
            .filter(tbl_auth_user_role::Column::UserId.eq(tbl_auth_user.id))
            .into_tuple::<i32>()
            .all(&app_state.db_conn)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                log::error!("tbl_auth_user_role find err: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        let tbl_auth_roles = match tbl_auth_role::Entity::find()
            .filter(tbl_auth_role::Column::Id.is_in(role_ids))
            .all(&app_state.db_conn)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                log::error!("tbl_auth_role find err: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        let mut roles = Vec::new();
        for tbl_auth_role in tbl_auth_roles {
            let (restful_apis, _len): (Vec<RestfulApi>, usize) = match bincode::decode_from_slice(
                &tbl_auth_role.apis[..],
                bincode::config::standard(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("bincode::decode_from_slice err: {}", e);
                    continue;
                }
            };

            let role = RoleQueryOutputDto {
                id: tbl_auth_role.id,
                name: tbl_auth_role.name,
                restful_apis,
            };
            roles.push(role);
        }
        users.push(UserQueryOutputDto {
            id: tbl_auth_user.id,
            username: tbl_auth_user.username,
            roles,
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
                "user":users
            }
           }
        )),
    )
        .into_response()
}

#[derive(Deserialize, Debug, Validate)]
struct UserCreateInputDto {
    name: String,
    restful_apis: Vec<RestfulApi>,
}
async fn user_create(
    app_state: State<AppState>,
    Json(create_input_dto): Json<UserCreateInputDto>,
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
struct UserUpdateInputDto {
    id: i32,
    name: String,
    restful_apis: Vec<RestfulApi>,
}
async fn user_update(
    app_state: State<AppState>,
    Json(update_input_dto): Json<UserUpdateInputDto>,
) -> impl IntoResponse {
    let tbl_auth_role = match tbl_auth_role::Entity::find_by_id(update_input_dto.id)
        .one(&app_state.db_conn)
        .await
    {
        Ok(op) => match op {
            Some(v) => v,
            None => {
                log::error!("tbl_auth_role {} not exist", update_input_dto.id);
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

async fn user_detail(Path(id): Path<i32>, State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_auth_user::Entity::find_by_id(id)
        .one(&app_state.db_conn)
        .await
    {
        Ok(op) => match op {
            Some(tbl_auth_user) => {
                let role_ids = match tbl_auth_user_role::Entity::find()
                    .select_only()
                    .column(tbl_auth_user_role::Column::RoleId)
                    .filter(tbl_auth_user_role::Column::UserId.eq(tbl_auth_user.id))
                    .into_tuple::<i32>()
                    .all(&app_state.db_conn)
                    .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("tbl_auth_user_role find err: {}", e);
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };
                let tbl_auth_roles = match tbl_auth_role::Entity::find()
                    .filter(tbl_auth_role::Column::Id.is_in(role_ids))
                    .all(&app_state.db_conn)
                    .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("tbl_auth_role find err: {}", e);
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };
                let mut roles = Vec::new();
                for tbl_auth_role in tbl_auth_roles {
                    let (restful_apis, _len): (Vec<RestfulApi>, usize) =
                        match bincode::decode_from_slice(
                            &tbl_auth_role.apis[..],
                            bincode::config::standard(),
                        ) {
                            Ok(v) => v,
                            Err(e) => {
                                log::error!("bincode::decode_from_slice err: {}", e);
                                continue;
                            }
                        };

                    let role = RoleQueryOutputDto {
                        id: tbl_auth_role.id,
                        name: tbl_auth_role.name,
                        restful_apis,
                    };
                    roles.push(role);
                }

                (
                    StatusCode::OK,
                    Json(json!({
                        "id":tbl_auth_user.id,
                        "name":tbl_auth_user.username,
                        "roles":roles,
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
