use std::{collections::HashSet, net::SocketAddr, ops::Deref};

use axum::{
    Json, Router,
    extract::{ConnectInfo, FromRequestParts, Path, Query, State},
    http::{Method, StatusCode, header, request::Parts},
    response::IntoResponse,
    routing::{get, patch, post},
};
use bincode::{Decode, Encode};
use chrono::Utc;
use entity::{tbl_auth_role, tbl_auth_user, tbl_auth_user_role};
use once_cell::sync::Lazy;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::AppState;

pub fn routers(state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout/{token}", post(logout))
        .route("/roles", get(role_query).post(role_create))
        .route("/roles/{id}", patch(role_update))
        .route("/user", get(user_query).post(user_create))
        .route("/user/{id}", patch(user_update))
        .with_state(state)
}
#[derive(Deserialize, Debug, Validate)]
struct LoginInputDto {
    username: String,
    password: String,
}
async fn login(
    app_state: State<AppState>,
    Json(login_input_dto): Json<LoginInputDto>,
) -> impl IntoResponse {
    match tbl_auth_user::Entity::find()
        .filter(tbl_auth_user::Column::Username.eq(&login_input_dto.username))
        .one(&app_state.db_conn)
        .await
    {
        Ok(tbl_auth_user_op) => match tbl_auth_user_op {
            Some(tbl_auth_user) => {
                if tbl_auth_user.password.eq(&login_input_dto.password) {
                    let token = uuid::Uuid::new_v4().to_string();
                    if let Err(e) = app_state
                        .sled_db
                        .insert(token.clone(), &chrono::Utc::now().timestamp().to_be_bytes())
                    {
                        log::error!("sled db insert err: {}", e);
                    }
                    (
                        StatusCode::OK,
                        [("code", "200"), ("msg", "ok")],
                        Json(json!({
                            "token": token
                        })),
                    )
                } else {
                    (
                        StatusCode::UNAUTHORIZED,
                        [("code", "401"), ("msg", "UNAUTHORIZED")],
                        Json(json!({})),
                    )
                }
            }
            None => {
                // 初始化admin数据
                if login_input_dto.username.eq("admin")
                    && login_input_dto.password.eq("123qwe!@#QWE")
                {
                    let tbl_auth_user_am = tbl_auth_user::ActiveModel {
                        username: Set(login_input_dto.username),
                        password: Set(login_input_dto.password),
                        ..Default::default()
                    };
                    match tbl_auth_user::Entity::insert(tbl_auth_user_am)
                        .exec(&app_state.db_conn)
                        .await
                    {
                        Ok(_) => {
                            let token = uuid::Uuid::new_v4().to_string();
                            if let Err(e) = app_state.sled_db.insert(
                                token.clone(),
                                &chrono::Utc::now().timestamp().to_be_bytes(),
                            ) {
                                log::error!("sled db insert err: {}", e);
                            }
                            return (
                                StatusCode::OK,
                                [("code", "200"), ("msg", "ok")],
                                Json(json!({
                                    "token": token
                                })),
                            );
                        }
                        Err(e) => {
                            log::error!("tbl_auth_user insert err: {}", e);
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                [("code", "500"), ("msg", "tbl_auth_user insert err")],
                                Json(json!({})),
                            );
                        }
                    }
                }
                log::warn!("user {} not exists", login_input_dto.username);
                (
                    StatusCode::UNAUTHORIZED,
                    [("code", "401"), ("msg", "UNAUTHORIZED")],
                    Json(json!({})),
                )
            }
        },
        Err(e) => {
            log::error!("tbl_auth_user find err: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("code", "500"), ("msg", "tbl_auth_user find err")],
                Json(json!({})),
            )
        }
    }
}

async fn logout(Path(token): Path<String>, app_state: State<AppState>) -> impl IntoResponse {
    log::info!("{token} logout");
    match app_state.sled_db.remove(&token) {
        Ok(op) => match op {
            Some(_) => (StatusCode::OK, Json(json!({}))),
            None => {
                log::warn!("token {token} not exists");
                (StatusCode::OK, Json(json!({})))
            }
        },
        Err(e) => {
            log::error!("sled remove {token} err: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({})))
        }
    }
}

static WHITE_API_SET: Lazy<HashSet<(Method, &'static str)>> = Lazy::new(|| {
    HashSet::from([
        (Method::POST, "/api/login"),
        (Method::GET, "/api/agents"),
        (Method::GET, "/api/hosts"),
        (Method::POST, "/api/hosts"),
        (Method::GET, "/api/llm_tasks"),
    ])
});
pub struct RequireAuth;

impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync + Deref<Target = AppState>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let src_ip = match ConnectInfo::<SocketAddr>::from_request_parts(parts, state).await {
            Ok(ConnectInfo(socket_addr)) => {
                // log::info!("{} {} {}", socket_addr.ip(), parts.method, parts.uri);
                socket_addr.ip().to_string()
            }
            Err(e) => {
                log::error!("get source ip err: {}", e);
                "".to_string()
            }
        };

        if !parts.uri.path().starts_with("/api") {
            log::info!(
                "allow no api access: {} {} {}",
                src_ip,
                parts.method,
                parts.uri.path()
            );
            return Ok(Self);
        }
        for (method, api) in WHITE_API_SET.iter() {
            if method == parts.method && parts.uri.path().starts_with(api) {
                log::info!(
                    "white list api {} {} {}",
                    src_ip,
                    parts.method,
                    parts.uri.path()
                );
                return Ok(Self);
            }
        }

        log::info!("auth api {} {} {}", src_ip, parts.method, parts.uri.path());
        if let Some(authorization) = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        {
            if let Some((_, token)) = authorization.split_once(" ") {
                match state.sled_db.contains_key(token) {
                    Ok(is_contains) => {
                        if is_contains {
                            if let Err(e) = state
                                .sled_db
                                .insert(token, &chrono::Utc::now().timestamp().to_be_bytes())
                            {
                                log::error!("sled db insert err: {}", e);
                            }
                            log::info!(
                                "auth success {} {} {}",
                                src_ip,
                                parts.method,
                                parts.uri.path()
                            );
                            return Ok(Self);
                        } else {
                            log::warn!("sled db not contains token: {}", token);
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    }
                    Err(e) => {
                        log::error!("sled db contains key err: {}", e);
                    }
                }
            }
        }
        log::warn!(
            "not has auth info, api {} {} {}",
            src_ip,
            parts.method,
            parts.uri.path()
        );
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn token_expired_task(sled_db: sled::Db) -> anyhow::Result<()> {
    let expired_time = 6 * 60 * 60; // 6小时
    tokio::spawn(async move {
        log::info!("token_expired_task running");
        loop {
            for (k, v) in sled_db.iter().flatten() {
                let ts_now = Utc::now().timestamp();

                let ts = v
                    .as_ref()
                    .try_into()
                    .map(i64::from_be_bytes)
                    .unwrap_or_else(|e| {
                        log::error!("v.as_ref().try_into() err: {}", e);
                        0
                    });

                if ts_now - ts >= expired_time {
                    if let Err(e) = sled_db.remove(&k) {
                        log::error!("sled remove err: {}", e);
                    }
                    log::info!("token expired {}", String::from_utf8_lossy(&k));
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });
    Ok(())
}

#[derive(Deserialize, Debug, Validate)]
struct RoleQueryInputDto {
    name: Option<String>,
    size: u64,
    page: u64,
}

#[derive(Serialize, Debug)]
struct RoleQueryOutputDto {
    id: i32,
    name: String,
    restful_apis: Vec<RestfulApi>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
struct RestfulApi {
    method: String,
    path: String,
    name: String,
    is_owned: bool,
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
            restful_apis,
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
    id: i32,
    name: String,
    restful_apis: Vec<RestfulApi>,
}
async fn role_update(
    app_state: State<AppState>,
    Json(update_input_dto): Json<RoleUpdateInputDto>,
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
            .column(tbl_auth_role::Column::Apis)
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
