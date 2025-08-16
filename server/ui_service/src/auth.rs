use std::{collections::HashSet, net::SocketAddr, ops::Deref, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{ConnectInfo, FromRequestParts, Path, State},
    http::{Method, StatusCode, header, request::Parts},
    response::IntoResponse,
    routing::{get, post},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use bincode::{Decode, Encode};
use captcha::{Captcha, filters::Noise};
use chrono::Utc;
use entity::{tbl_auth_role, tbl_auth_user, tbl_auth_user_role};
use moka::{notification::RemovalCause, sync::Cache};
use once_cell::sync::Lazy;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs1::EncodeRsaPublicKey};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::AppState;

pub static RESTFUL_APIS: Lazy<Vec<RestfulApi>> = Lazy::new(|| {
    vec![
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/agents".to_string(),
            name: "Agent查询".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/agents/".to_string(),
            name: "Agent详情".to_string(),
        },
        RestfulApi {
            method: "DELETE".to_string(),
            path: "/api/agents/".to_string(),
            name: "Agent删除".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/roles".to_string(),
            name: "角色查询".to_string(),
        },
        RestfulApi {
            method: "POST".to_string(),
            path: "/api/roles".to_string(),
            name: "角色新增".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/roles/".to_string(),
            name: "角色详情".to_string(),
        },
        RestfulApi {
            method: "PATCH".to_string(),
            path: "/api/roles/".to_string(),
            name: "角色修改".to_string(),
        },
        RestfulApi {
            method: "DELETE".to_string(),
            path: "/api/roles/".to_string(),
            name: "角色删除".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            name: "用户查询".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/users/".to_string(),
            name: "用户详情".to_string(),
        },
        RestfulApi {
            method: "POST".to_string(),
            path: "/api/users".to_string(),
            name: "用户新增".to_string(),
        },
        RestfulApi {
            method: "PATCH".to_string(),
            path: "/api/users/".to_string(),
            name: "用户修改".to_string(),
        },
        RestfulApi {
            method: "DELETE".to_string(),
            path: "/api/users/".to_string(),
            name: "用户删除".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/hosts".to_string(),
            name: "主机查询".to_string(),
        },
        RestfulApi {
            method: "POST".to_string(),
            path: "/api/hosts".to_string(),
            name: "主机更新".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/hosts/".to_string(),
            name: "主机详情".to_string(),
        },
        RestfulApi {
            method: "DELETE".to_string(),
            path: "/api/hosts/".to_string(),
            name: "主机删除".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/llm_tasks".to_string(),
            name: "大语言模型任务查询".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/llm_tasks/".to_string(),
            name: "大语言模型任务详情".to_string(),
        },
        RestfulApi {
            method: "DELETE".to_string(),
            path: "/api/llm_tasks/".to_string(),
            name: "大语言模型任务删除".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/restful_apis".to_string(),
            name: "接口列表".to_string(),
        },
        RestfulApi {
            method: "POST".to_string(),
            path: "/api/system/title".to_string(),
            name: "标题更新".to_string(),
        },
        RestfulApi {
            method: "POST".to_string(),
            path: "/api/system/icon".to_string(),
            name: "标题更新".to_string(),
        },
        RestfulApi {
            method: "POST".to_string(),
            path: "/api/system/logo".to_string(),
            name: "Logo更新".to_string(),
        },
        RestfulApi {
            method: "GET".to_string(),
            path: "/api/system".to_string(),
            name: "系统设置".to_string(),
        },
    ]
});
#[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone)]
pub struct RestfulApi {
    method: String,
    path: String,
    name: String,
}
pub fn routers(state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout/{token}", post(logout))
        .route("/captcha", get(generate_captcha))
        .with_state(state)
}
#[derive(Deserialize, Debug, Validate)]
struct LoginInputDto {
    username: String,
    password: String,
    uuid: String,
    captcha: String,
}
async fn login(
    app_state: State<AppState>,
    Json(login_input_dto): Json<LoginInputDto>,
) -> impl IntoResponse {
    let private_key = match app_state.captcha_cache.get(&login_input_dto.uuid) {
        Some(v) => {
            if !login_input_dto.captcha.eq(&v.captcha) {
                log::warn!(
                    "captcha not match {} vs {}",
                    login_input_dto.captcha,
                    v.captcha
                );
                return StatusCode::BAD_REQUEST.into_response();
            }
            v.private_key
        }
        None => {
            log::warn!("uuid exist {}", login_input_dto.uuid);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };
    let encrypted_bytes = match BASE64_STANDARD.decode(login_input_dto.password) {
        Ok(v) => v,
        Err(e) => {
            log::error!("BASE64_STANDARD.decode err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let password = match private_key.decrypt(Pkcs1v15Encrypt, &encrypted_bytes) {
        Ok(v) => match String::from_utf8(v) {
            Ok(v) => v,
            Err(e) => {
                log::error!("String::from_utf8 err: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        },
        Err(e) => {
            log::error!("private_key.decrypt err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match tbl_auth_user::Entity::find()
        .filter(tbl_auth_user::Column::Username.eq(&login_input_dto.username))
        .one(&app_state.db_conn)
        .await
    {
        Ok(tbl_auth_user_op) => match tbl_auth_user_op {
            Some(tbl_auth_user) => {
                if tbl_auth_user.password.eq(&password) {
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
                    let mut distinct_restful_apis: Vec<RestfulApi> = Vec::new();
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
                            let mut is_exist = false;
                            for owend_restful_api in &distinct_restful_apis {
                                if restful_api.method.eq(&owend_restful_api.method)
                                    && restful_api.path.eq(&owend_restful_api.path)
                                {
                                    is_exist = true;
                                    continue;
                                }
                            }
                            if !is_exist {
                                distinct_restful_apis.push(restful_api);
                            }
                        }
                    }
                    let token = uuid::Uuid::new_v4().to_string();

                    let token_value = TokenValue {
                        expired_time: chrono::Utc::now().timestamp(),
                        restful_apis: distinct_restful_apis.clone(),
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
                            "token": token,
                            "restful_apis":distinct_restful_apis,
                        })),
                    )
                        .into_response()
                } else {
                    StatusCode::UNAUTHORIZED.into_response()
                }
            }
            None => {
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
        (Method::POST, "/api/logout"),
        (Method::GET, "/api/captcha"),
        (Method::GET, "/api/system/title"),
        (Method::GET, "/api/system/icon"),
        (Method::GET, "/api/system/logo"),
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
                            // log::info!("{token_value:?}");
                            let mut is_auth = false;
                            for restful_api in &token_value.restful_apis {
                                // log::info!(
                                //     "{} vs {}, {} vs {}",
                                //     restful_api.method,
                                //     parts.method,
                                //     parts.uri.path(),
                                //     restful_api.path
                                // );
                                if restful_api.method.eq(&parts.method.to_string())
                                    && parts.uri.path().starts_with(&restful_api.path)
                                {
                                    is_auth = true;
                                    break;
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
    restful_apis: Vec<RestfulApi>,
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

pub async fn auth_init(db_conn: sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    // 初始化角色
    let super_role_name = "超管角色";
    let read_only_role_name = "只读角色";
    let super_admin_username = "sa";
    let guest_username = "guest";
    // 超管角色
    let encoded: Vec<u8> = bincode::encode_to_vec(&*RESTFUL_APIS, bincode::config::standard())?;
    let super_role_id = match tbl_auth_role::Entity::find()
        .filter(tbl_auth_role::Column::Name.eq(super_role_name))
        .one(&db_conn)
        .await?
    {
        Some(tbl_auth_role) => {
            let role_id = tbl_auth_role.id;
            let mut tbl_auth_role_am = tbl_auth_role.into_active_model();
            tbl_auth_role_am.apis = Set(encoded);
            tbl_auth_role_am.save(&db_conn).await?;
            role_id
        }
        None => {
            let tbl_auth_role_am = tbl_auth_role::ActiveModel {
                name: Set(super_role_name.to_string()),
                apis: Set(encoded),
                ..Default::default()
            };
            let insert_result = tbl_auth_role::Entity::insert(tbl_auth_role_am)
                .exec(&db_conn)
                .await?;
            insert_result.last_insert_id
        }
    };
    // 只读角色
    let mut read_only_restful_apis = Vec::new();
    for restful_api in RESTFUL_APIS.clone() {
        if restful_api.method.eq("GET") {
            read_only_restful_apis.push(restful_api);
        }
    }
    let encoded: Vec<u8> =
        bincode::encode_to_vec(read_only_restful_apis, bincode::config::standard())?;
    let read_only_role_id = match tbl_auth_role::Entity::find()
        .filter(tbl_auth_role::Column::Name.eq(read_only_role_name))
        .one(&db_conn)
        .await?
    {
        Some(tbl_auth_role) => {
            let role_id = tbl_auth_role.id;
            let mut tbl_auth_role_am = tbl_auth_role.into_active_model();
            tbl_auth_role_am.apis = Set(encoded);
            tbl_auth_role_am.save(&db_conn).await?;
            role_id
        }
        None => {
            let tbl_auth_role_am = tbl_auth_role::ActiveModel {
                name: Set(read_only_role_name.to_string()),
                apis: Set(encoded),
                ..Default::default()
            };
            let insert_result = tbl_auth_role::Entity::insert(tbl_auth_role_am)
                .exec(&db_conn)
                .await?;
            insert_result.last_insert_id
        }
    };
    // 初始化admin
    let super_admin_id = match tbl_auth_user::Entity::find()
        .filter(tbl_auth_user::Column::Username.eq(super_admin_username))
        .one(&db_conn)
        .await?
    {
        Some(tbl_auth_user) => tbl_auth_user.id,
        None => {
            let tbl_auth_user_am = tbl_auth_user::ActiveModel {
                username: Set(super_admin_username.to_string()),
                password: Set("sa".to_string()),
                ..Default::default()
            };
            let insert_result = tbl_auth_user::Entity::insert(tbl_auth_user_am)
                .exec(&db_conn)
                .await?;
            insert_result.last_insert_id
        }
    };
    // 初始化guest
    let guest_id = match tbl_auth_user::Entity::find()
        .filter(tbl_auth_user::Column::Username.eq(guest_username))
        .one(&db_conn)
        .await?
    {
        Some(tbl_auth_user) => tbl_auth_user.id,
        None => {
            let tbl_auth_user_am = tbl_auth_user::ActiveModel {
                username: Set(guest_username.to_string()),
                password: Set("guest".to_string()),
                ..Default::default()
            };
            let insert_result = tbl_auth_user::Entity::insert(tbl_auth_user_am)
                .exec(&db_conn)
                .await?;
            insert_result.last_insert_id
        }
    };

    // 初始化用户和角色关系表
    let count = tbl_auth_user_role::Entity::find()
        .filter(tbl_auth_user_role::Column::UserId.eq(super_admin_id))
        .filter(tbl_auth_user_role::Column::RoleId.eq(super_role_id))
        .count(&db_conn)
        .await?;
    if count < 1 {
        let tbl_auth_user_role_am = tbl_auth_user_role::ActiveModel {
            user_id: Set(super_admin_id),
            role_id: Set(super_role_id),
        };
        tbl_auth_user_role::Entity::insert(tbl_auth_user_role_am)
            .exec(&db_conn)
            .await?;
    }
    let count = tbl_auth_user_role::Entity::find()
        .filter(tbl_auth_user_role::Column::UserId.eq(guest_id))
        .filter(tbl_auth_user_role::Column::RoleId.eq(read_only_role_id))
        .count(&db_conn)
        .await?;
    if count < 1 {
        let tbl_auth_user_role_am = tbl_auth_user_role::ActiveModel {
            user_id: Set(guest_id),
            role_id: Set(read_only_role_id),
        };
        tbl_auth_user_role::Entity::insert(tbl_auth_user_role_am)
            .exec(&db_conn)
            .await?;
    }
    Ok(())
}

pub fn captcha_cache_init() -> anyhow::Result<Cache<String, CaptchaEntry>> {
    let eviction_listener =
        move |uuid: Arc<String>, _captcha_entry: CaptchaEntry, removal_cause| match removal_cause {
            RemovalCause::Expired => {
                log::info!("Expired: {uuid}");
            }
            RemovalCause::Explicit => {
                log::info!("Explicit: {uuid}");
            }
            RemovalCause::Replaced => {
                log::info!("Replaced: {uuid}");
            }
            RemovalCause::Size => {
                log::info!("Size: {uuid}");
            }
        };
    let cache = Cache::builder()
        .max_capacity(50_000)
        .time_to_live(Duration::from_secs(60 * 10))
        .eviction_listener(eviction_listener)
        .build();
    Ok(cache)
}

#[derive(Clone)]
pub struct CaptchaEntry {
    captcha: String,
    private_key: RsaPrivateKey,
}

async fn generate_captcha(app_state: State<AppState>) -> impl IntoResponse {
    let mut rng_captcha = Captcha::new();
    rng_captcha
        .set_chars(&['1', '2', '3', '4', '5', '6', '7', '8', '9'])
        .add_chars(5)
        .apply_filter(Noise::new(0.3))
        .view(180, 60);

    let base64_captcha = match rng_captcha.as_base64() {
        Some(v) => v,
        None => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut rng = rand::thread_rng(); // rand@0.8
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    let public_key = match public_key.to_pkcs1_pem(Default::default()) {
        Ok(v) => v,
        Err(e) => {
            log::error!("public_key.to_pkcs1_pem err: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let uuid = uuid::Uuid::new_v4().to_string();
    let captcha_entry = CaptchaEntry {
        captcha: rng_captcha.chars_as_string(),
        private_key,
    };
    app_state.captcha_cache.insert(uuid.clone(), captcha_entry);
    (
        StatusCode::OK,
        Json(json!({
            "uuid":uuid,
            "base64_captcha":base64_captcha,
            "public_key":public_key
        })),
    )
        .into_response()
}
