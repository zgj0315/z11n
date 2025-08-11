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

static RESTFUL_APIS: Lazy<Vec<RestfulApi>> = Lazy::new(|| {
    let mut restful_apis = Vec::new();
    restful_apis.push(RestfulApi {
        method: "GET".to_string(),
        path: "api/agents".to_string(),
        name: "Agent查询".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "GET".to_string(),
        path: "api/agents/".to_string(),
        name: "Agent详情".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "DELETE".to_string(),
        path: "api/agents/".to_string(),
        name: "Agent删除".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "POST".to_string(),
        path: "api/login".to_string(),
        name: "登录".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "POST".to_string(),
        path: "api/logout".to_string(),
        name: "退出".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "GET".to_string(),
        path: "api/roles".to_string(),
        name: "角色查询".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "POST".to_string(),
        path: "api/roles".to_string(),
        name: "角色新增".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "PATCH".to_string(),
        path: "api/roles/".to_string(),
        name: "角色修改".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "GET".to_string(),
        path: "api/users".to_string(),
        name: "用户查询".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "POST".to_string(),
        path: "api/users".to_string(),
        name: "用户新增".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "PATCH".to_string(),
        path: "api/users/".to_string(),
        name: "用户修改".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "GET".to_string(),
        path: "api/hosts".to_string(),
        name: "主机查询".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "GET".to_string(),
        path: "api/hosts/".to_string(),
        name: "主机详情".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "DELETE".to_string(),
        path: "api/hosts/".to_string(),
        name: "主机删除".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "GET".to_string(),
        path: "api/llm_tasks".to_string(),
        name: "大语言模型任务查询".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "GET".to_string(),
        path: "api/llm_tasks/".to_string(),
        name: "大语言模型任务详情".to_string(),
    });
    restful_apis.push(RestfulApi {
        method: "DELETE".to_string(),
        path: "api/llm_tasks/".to_string(),
        name: "大语言模型任务删除".to_string(),
    });
    restful_apis
});

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
                    let mut owend_restful_apis = Vec::new();
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
                        for restful_api in restful_apis {
                            // 暴力插入，这里需要去重
                            owend_restful_apis.push(restful_api);
                        }
                    }
                    let token = uuid::Uuid::new_v4().to_string();

                    let token_value = TokenValue {
                        expired_time: chrono::Utc::now().timestamp(),
                        owend_restful_apis,
                    };
                    let encoded: Vec<u8> =
                        match bincode::encode_to_vec(&token_value, bincode::config::standard()) {
                            Ok(v) => v,
                            Err(e) => {
                                log::error!("bincode::encode_to_vec err: {}", e);
                                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                            }
                        };
                    if let Err(e) = app_state.sled_db.insert(token.clone(), &*encoded) {
                        log::error!("sled db insert err: {}", e);
                    }
                    (
                        StatusCode::OK,
                        Json(json!({
                            "token": token
                        })),
                    )
                        .into_response()
                } else {
                    StatusCode::UNAUTHORIZED.into_response()
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

                            let token_value = TokenValue {
                                expired_time: chrono::Utc::now().timestamp(),
                                owend_restful_apis: RESTFUL_APIS.clone(),
                            };
                            let encoded: Vec<u8> = match bincode::encode_to_vec(
                                &token_value,
                                bincode::config::standard(),
                            ) {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("bincode::encode_to_vec err: {}", e);
                                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                                }
                            };
                            if let Err(e) = app_state.sled_db.insert(token.clone(), &*encoded) {
                                log::error!("sled db insert err: {}", e);
                            }
                            return (
                                StatusCode::OK,
                                Json(json!({
                                    "token": token
                                })),
                            )
                                .into_response();
                        }
                        Err(e) => {
                            log::error!("tbl_auth_user insert err: {}", e);
                            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                        }
                    }
                }
                log::warn!("user {} not exists", login_input_dto.username);
                StatusCode::UNAUTHORIZED.into_response()
            }
        },
        Err(e) => {
            log::error!("tbl_auth_user find err: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
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
                match state.sled_db.get(token) {
                    Ok(op) => match op {
                        Some(v) => {
                            let (mut token_value, _len): (TokenValue, usize) =
                                match bincode::decode_from_slice(
                                    &v[..],
                                    bincode::config::standard(),
                                ) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        log::error!("bincode::decode_from_slice err: {}", e);
                                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                                    }
                                };
                            let mut is_auth = false;
                            for restful_api in &token_value.owend_restful_apis {
                                if restful_api.method.eq(&parts.method.to_string()) {
                                    if parts.uri.path().starts_with(&restful_api.path) {
                                        is_auth = true;
                                        break;
                                    }
                                }
                            }
                            if is_auth {
                                token_value.expired_time = chrono::Utc::now().timestamp();
                                let encoded: Vec<u8> = match bincode::encode_to_vec(
                                    &token_value,
                                    bincode::config::standard(),
                                ) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        log::error!("bincode::encode_to_vec err: {}", e);
                                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                                    }
                                };
                                if let Err(e) = state.sled_db.insert(token, &*encoded) {
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
                                log::warn!(
                                    "access denied {} {} {}",
                                    src_ip,
                                    parts.method,
                                    parts.uri.path()
                                );
                                return Err(StatusCode::FORBIDDEN);
                            }
                        }
                        None => {
                            log::warn!("sled db not contains token: {}", token);
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    },
                    Err(e) => {
                        log::error!("sled_db get {} err: {}", token, e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
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

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
struct TokenValue {
    expired_time: i64,
    owend_restful_apis: Vec<RestfulApi>,
}

pub async fn token_expired_task(sled_db: sled::Db) -> anyhow::Result<()> {
    let expired_time = 6 * 60 * 60; // 6小时
    tokio::spawn(async move {
        log::info!("token_expired_task running");
        loop {
            for (k, v) in sled_db.iter().flatten() {
                let (token_value, _len): (TokenValue, usize) =
                    match bincode::decode_from_slice(&v[..], bincode::config::standard()) {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("bincode::decode_from_slice err: {}", e);
                            log::warn!("forced sled_db.remove {:?}", k);
                            if let Err(e) = sled_db.remove(&k) {
                                log::error!("sled remove err: {}", e);
                            }
                            continue;
                        }
                    };

                if Utc::now().timestamp() - token_value.expired_time >= expired_time {
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
    restful_apis: Vec<RestfulApiWithAuth>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
struct RestfulApiWithAuth {
    restful_api: RestfulApi,
    is_owned: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone)]
struct RestfulApi {
    method: String,
    path: String,
    name: String,
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
        let (owned_restful_apis, _len): (Vec<RestfulApi>, usize) =
            match bincode::decode_from_slice(&encoded[..], bincode::config::standard()) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("bincode::decode_from_slice err: {}", e);
                    continue;
                }
            };
        let mut restful_apis_with_auth = Vec::new();
        for restful_api in RESTFUL_APIS.iter() {
            let mut is_owned = false;
            for owned_restful_api in &owned_restful_apis {
                if restful_api.method.eq(&owned_restful_api.method)
                    && restful_api.path.eq(&owned_restful_api.path)
                {
                    is_owned = true;
                    continue;
                }
            }
            restful_apis_with_auth.push(RestfulApiWithAuth {
                restful_api: restful_api.clone(),
                is_owned,
            });
        }

        roles.push(RoleQueryOutputDto {
            id: tbl_auth_role.id,
            name: tbl_auth_role.name,
            restful_apis: restful_apis_with_auth,
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
    restful_apis: Vec<RestfulApiWithAuth>,
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
    restful_apis: Vec<RestfulApiWithAuth>,
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
            let (owned_restful_apis, _len): (Vec<RestfulApi>, usize) =
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
            let mut restful_apis_with_auth = Vec::new();
            for restful_api in RESTFUL_APIS.iter() {
                let mut is_owned = false;
                for owned_restful_api in &owned_restful_apis {
                    if restful_api.method.eq(&owned_restful_api.method)
                        && restful_api.path.eq(&owned_restful_api.path)
                    {
                        is_owned = true;
                        continue;
                    }
                }
                restful_apis_with_auth.push(RestfulApiWithAuth {
                    restful_api: restful_api.clone(),
                    is_owned,
                });
            }
            let role = RoleQueryOutputDto {
                id: tbl_auth_role.id,
                name: tbl_auth_role.name,
                restful_apis: restful_apis_with_auth,
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
    restful_apis: Vec<RestfulApiWithAuth>,
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
    restful_apis: Vec<RestfulApiWithAuth>,
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
