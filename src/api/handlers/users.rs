use crate::api::dto::user::{PaginationQuery, UpdateProfileRequest, UpdateUserRequest, UserResponse};
use crate::api::error::AppError;
use crate::app::services::user_service::UserService;
use crate::domain::{AdminUpdateUser, UpdateProfile};
use crate::infra::db::user_repo::SqlxUserRepository;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;
use validator::Validate;

use crate::api::middleware::auth::{AdminGuard, CurrentUser};

#[utoipa::path(
    get,
    path = "/users/me",
    responses(
        (status = 200, body = UserResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_me_handler(
    CurrentUser(current_user): CurrentUser,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(UserResponse::from(current_user)))
}

#[utoipa::path(
    patch,
    path = "/users/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn update_me_handler(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let repo = SqlxUserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let updated = service
        .update_profile(
            current_user.id,
            UpdateProfile {
                email: payload.email,
                username: payload.username,
            },
        )
        .await?;

    Ok(Json(UserResponse::from(updated)))
}

#[utoipa::path(
    get,
    path = "/users",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, body = [UserResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn list_users_handler(
    State(state): State<AppState>,
    AdminGuard(_admin): AdminGuard,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = pagination.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let repo = SqlxUserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let users = service.list_users(per_page, offset).await?;
    let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "User id")
    ),
    responses(
        (status = 200, body = UserResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_user_handler(
    State(state): State<AppState>,
    AdminGuard(_admin): AdminGuard,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("invalid user id".to_string()))?;

    let repo = SqlxUserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let user = service.get_profile(user_id).await?;

    Ok(Json(UserResponse::from(user)))
}

#[utoipa::path(
    patch,
    path = "/users/{id}",
    request_body = UpdateUserRequest,
    params(
        ("id" = String, Path, description = "User id")
    ),
    responses(
        (status = 200, body = UserResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn update_user_handler(
    State(state): State<AppState>,
    AdminGuard(_admin): AdminGuard,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let user_id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("invalid user id".to_string()))?;

    let repo = SqlxUserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let user = service
        .update_user(
            user_id,
            AdminUpdateUser {
                email: payload.email,
                username: payload.username,
                role: payload.role,
                is_active: payload.is_active,
            },
        )
        .await?;

    Ok(Json(UserResponse::from(user)))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "User id")
    ),
    responses(
        (status = 204, description = "Deactivated"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn deactivate_user_handler(
    State(state): State<AppState>,
    AdminGuard(_admin): AdminGuard,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("invalid user id".to_string()))?;

    let repo = SqlxUserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    service.deactivate_user(user_id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
