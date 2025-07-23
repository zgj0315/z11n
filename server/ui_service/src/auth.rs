use std::{collections::HashSet, net::SocketAddr, ops::Deref};

use axum::{
    Json, Router,
    extract::{ConnectInfo, FromRequestParts, Path, State},
    http::{Method, StatusCode, header, request::Parts},
    response::IntoResponse,
    routing::post,
};
use chrono::Utc;
use entity::tbl_auth_user;
use once_cell::sync::Lazy;
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::AppState;
pub fn routers(state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout/{token}", post(logout))
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
