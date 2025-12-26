use crate::api::dto::auth::{LoginRequest, LoginResponse, RefreshRequest, RegisterRequest};
use crate::api::dto::user::UserResponse;
use crate::api::error::AppError;
use crate::app::services::auth_service::{AuthService, LoginInput, RegisterInput};
use crate::api::middleware::auth::CurrentUser;
use crate::infra::auth::jwt::JwtService;
use crate::infra::db::user_repo::SqlxUserRepository;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Conflict")
    ),
    tag = "auth"
)]
pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let repo = SqlxUserRepository::new(state.db.clone());
    let service = AuthService::new(repo, JwtService::new(&state.config));

    let user = service
        .register_user(RegisterInput {
            email: payload.email,
            username: payload.username,
            password: payload.password,
        })
        .await?;

    let response = UserResponse::from(user);
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, body = LoginResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "auth"
)]
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let repo = SqlxUserRepository::new(state.db.clone());
    let service = AuthService::new(repo, JwtService::new(&state.config));

    let response = service
        .login(LoginInput {
            email: payload.email,
            password: payload.password,
        })
        .await?;

    let body = LoginResponse {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        user: UserResponse::from(response.user),
    };

    Ok(Json(body))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, body = LoginResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "auth"
)]
pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let repo = SqlxUserRepository::new(state.db.clone());
    let service = AuthService::new(repo, JwtService::new(&state.config));

    let response = service.refresh_tokens(payload.refresh_token).await?;

    let body = LoginResponse {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        user: UserResponse::from(response.user),
    };

    Ok(Json(body))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 204, description = "Logged out")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "auth"
)]
pub async fn logout_handler(CurrentUser(_user): CurrentUser) -> Result<impl IntoResponse, AppError> {
    Ok(StatusCode::NO_CONTENT)
}
