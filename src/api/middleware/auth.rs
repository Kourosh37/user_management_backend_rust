use crate::api::error::AppError;
use crate::domain::{User, UserRepository};
use crate::infra::auth::jwt::{JwtService, TokenType};
use crate::infra::db::user_repo::SqlxUserRepository;
use crate::AppState;
use axum::extract::FromRequestParts;
use axum::http::{header, request::Parts};
use axum::extract::FromRef;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CurrentUser(pub User);

#[derive(Debug, Clone)]
pub struct AdminGuard(pub User);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing authorization".to_string()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("invalid authorization".to_string()))?;

        let state = AppState::from_ref(state);
        let jwt = JwtService::new(&state.config);
        let claims = jwt.decode_token(token)?;

        if claims.token_type != TokenType::Access {
            return Err(AppError::Unauthorized("invalid token".to_string()));
        }

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("invalid token".to_string()))?;

        let repo = SqlxUserRepository::new(state.db.clone());
        let user_with_password = repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid token".to_string()))?;

        if !user_with_password.user.is_active() {
            return Err(AppError::Unauthorized("user is inactive".to_string()));
        }

        Ok(CurrentUser(user_with_password.user))
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AdminGuard
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;

        if !user.role.can_manage_users() {
            return Err(AppError::Forbidden("admin access required".to_string()));
        }

        Ok(AdminGuard(user))
    }
}
