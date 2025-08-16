use axum::{
    Json, Router,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use entity::tbl_system_config;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::AppState;

pub fn routers(state: AppState) -> Router {
    Router::new()
        .route("/system/title", get(get_title).post(update_title))
        .route("/system/icon", get(get_icon).post(update_icon))
        .route("/system/logo", get(get_logo).post(update_logo))
        .with_state(state)
}

#[derive(
    Debug, PartialEq, strum_macros::EnumString, strum_macros::Display, strum_macros::EnumIter,
)]
enum ConfigKey {
    Title,
    Icon,
    Logo,
}

async fn get_title(State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_system_config::Entity::find()
        .filter(tbl_system_config::Column::Key.eq(ConfigKey::Title.to_string()))
        .one(&app_state.db_conn)
        .await
    {
        Ok(op) => match op {
            Some(tbl_system_config) => {
                let title = match String::from_utf8(tbl_system_config.value) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("String::from_utf8 err: {}", e);
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };
                (
                    StatusCode::OK,
                    Json(json!({
                        "title":title,
                    })),
                )
                    .into_response()
            }
            None => StatusCode::GONE.into_response(),
        },
        Err(e) => {
            log::error!(
                "find tbl_system_config {} db err: {}",
                ConfigKey::Title,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize, Debug, Validate)]
struct TitleUpdateInputDto {
    title: String,
}
async fn update_title(
    State(app_state): State<AppState>,
    Json(input): Json<TitleUpdateInputDto>,
) -> impl IntoResponse {
    match tbl_system_config::Entity::find()
        .filter(tbl_system_config::Column::Key.eq(ConfigKey::Title.to_string()))
        .one(&app_state.db_conn)
        .await
    {
        Ok(op) => match op {
            Some(tbl_system_config) => {
                let mut tbl_system_config_am = tbl_system_config.into_active_model();
                tbl_system_config_am.value = Set(input.title.as_bytes().to_vec());
                match tbl_system_config_am.save(&app_state.db_conn).await {
                    Ok(_) => {
                        StatusCode::OK.into_response()
                    }
                    Err(e) => {
                        log::error!("tbl_system_config save err: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            }
            None => {
                let tbl_system_config_am = tbl_system_config::ActiveModel {
                    key: Set(ConfigKey::Title.to_string()),
                    value: Set(input.title.as_bytes().to_vec()),
                };
                match tbl_system_config::Entity::insert(tbl_system_config_am)
                    .exec(&app_state.db_conn)
                    .await
                {
                    Ok(_) => {
                        StatusCode::OK.into_response()
                    }
                    Err(e) => {
                        log::error!("tbl_system_config insert err: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            }
        },
        Err(e) => {
            log::error!("tbl_system_config find err: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_icon(State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_system_config::Entity::find()
        .filter(tbl_system_config::Column::Key.eq(ConfigKey::Icon.to_string()))
        .one(&app_state.db_conn)
        .await
    {
        Ok(op) => match op {
            Some(tbl_system_config) => (
                StatusCode::OK,
                Json(json!({
                    "base64_icon":BASE64_STANDARD.encode(tbl_system_config.value),
                })),
            )
                .into_response(),
            None => StatusCode::GONE.into_response(),
        },
        Err(e) => {
            log::error!(
                "find tbl_system_config {} db err: {}",
                ConfigKey::Icon,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update_icon(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(field) = match multipart.next_field().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("multipart.next_field err: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    } {
        if let Some(name) = field.name() {
            log::info!("name: {name}");
        }
        log::info!("file_name: {:?}", field.file_name());

        if let Some(content_type) = field.content_type() {
            log::info!("content_type: {content_type}");
            // if !content_type.eq("application/pdf") {
            //     log::warn!("content-type should be application/pdf");
            //     return StatusCode::BAD_REQUEST.into_response();
            // }
            let content_bytes = match field.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(e_bytes) => {
                    log::error!("get field bytes err: {}", e_bytes);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            };
            match tbl_system_config::Entity::find()
                .filter(tbl_system_config::Column::Key.eq(ConfigKey::Icon.to_string()))
                .one(&app_state.db_conn)
                .await
            {
                Ok(op) => match op {
                    Some(tbl_system_config) => {
                        let mut tbl_system_config_am = tbl_system_config.into_active_model();
                        tbl_system_config_am.value = Set(content_bytes);
                        match tbl_system_config_am.save(&app_state.db_conn).await {
                            Ok(_) => {
                                return StatusCode::OK.into_response();
                            }
                            Err(e) => {
                                log::error!("tbl_system_config save err: {}", e);
                                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                            }
                        }
                    }
                    None => {
                        let tbl_system_config_am = tbl_system_config::ActiveModel {
                            key: Set(ConfigKey::Icon.to_string()),
                            value: Set(content_bytes),
                        };
                        match tbl_system_config::Entity::insert(tbl_system_config_am)
                            .exec(&app_state.db_conn)
                            .await
                        {
                            Ok(_) => {
                                return StatusCode::OK.into_response();
                            }
                            Err(e) => {
                                log::error!("tbl_system_config insert err: {}", e);
                                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                            }
                        }
                    }
                },
                Err(e) => {
                    log::error!("tbl_system_config find err: {}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
    }

    StatusCode::OK.into_response()
}

async fn get_logo(State(app_state): State<AppState>) -> impl IntoResponse {
    match tbl_system_config::Entity::find()
        .filter(tbl_system_config::Column::Key.eq(ConfigKey::Logo.to_string()))
        .one(&app_state.db_conn)
        .await
    {
        Ok(op) => match op {
            Some(tbl_system_config) => (
                StatusCode::OK,
                Json(json!({
                    "base64_logo":BASE64_STANDARD.encode(tbl_system_config.value),
                })),
            )
                .into_response(),
            None => StatusCode::GONE.into_response(),
        },
        Err(e) => {
            log::error!(
                "find tbl_system_config {} db err: {}",
                ConfigKey::Logo,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update_logo(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(field) = match multipart.next_field().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("multipart.next_field err: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    } {
        if let Some(name) = field.name() {
            log::info!("name: {name}");
        }
        log::info!("file_name: {:?}", field.file_name());

        if let Some(content_type) = field.content_type() {
            log::info!("content_type: {content_type}");
            // if !content_type.eq("application/pdf") {
            //     log::warn!("content-type should be application/pdf");
            //     return StatusCode::BAD_REQUEST.into_response();
            // }
            let content_bytes = match field.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(e_bytes) => {
                    log::error!("get field bytes err: {}", e_bytes);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            };
            match tbl_system_config::Entity::find()
                .filter(tbl_system_config::Column::Key.eq(ConfigKey::Logo.to_string()))
                .one(&app_state.db_conn)
                .await
            {
                Ok(op) => match op {
                    Some(tbl_system_config) => {
                        let mut tbl_system_config_am = tbl_system_config.into_active_model();
                        tbl_system_config_am.value = Set(content_bytes);
                        match tbl_system_config_am.save(&app_state.db_conn).await {
                            Ok(_) => {
                                return StatusCode::OK.into_response();
                            }
                            Err(e) => {
                                log::error!("tbl_system_config save err: {}", e);
                                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                            }
                        }
                    }
                    None => {
                        let tbl_system_config_am = tbl_system_config::ActiveModel {
                            key: Set(ConfigKey::Logo.to_string()),
                            value: Set(content_bytes),
                        };
                        match tbl_system_config::Entity::insert(tbl_system_config_am)
                            .exec(&app_state.db_conn)
                            .await
                        {
                            Ok(_) => {
                                return StatusCode::OK.into_response();
                            }
                            Err(e) => {
                                log::error!("tbl_system_config insert err: {}", e);
                                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                            }
                        }
                    }
                },
                Err(e) => {
                    log::error!("tbl_system_config find err: {}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
    }

    StatusCode::OK.into_response()
}
