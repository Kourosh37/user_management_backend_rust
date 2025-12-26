use crate::domain::{DomainError, NewUser, Role, User, UserRepository};
use crate::infra::auth::jwt::{Claims, JwtService, TokenType};
use crate::infra::security::password;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RegisterInput {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
}

pub struct AuthService<R> {
    repo: R,
    jwt: JwtService,
}

impl<R> AuthService<R>
where
    R: UserRepository,
{
    pub fn new(repo: R, jwt: JwtService) -> Self {
        Self { repo, jwt }
    }

    pub async fn register_user(&self, input: RegisterInput) -> Result<User, DomainError> {
        if self.repo.find_by_email(&input.email).await?.is_some() {
            return Err(DomainError::Conflict("email already exists".to_string()));
        }

        if self
            .repo
            .find_by_username(&input.username)
            .await?
            .is_some()
        {
            return Err(DomainError::Conflict("username already exists".to_string()));
        }

        let password_hash = password::hash_password(&input.password)?;
        let new_user = NewUser {
            email: input.email,
            username: input.username,
            password_hash,
            role: Role::User,
            is_active: true,
        };

        self.repo.create(new_user).await
    }

    pub async fn login(&self, input: LoginInput) -> Result<LoginResponse, DomainError> {
        let user_with_password = self
            .repo
            .find_by_email(&input.email)
            .await?
            .ok_or_else(|| DomainError::Unauthorized("invalid credentials".to_string()))?;

        if !password::verify_password(&user_with_password.password_hash, &input.password)? {
            return Err(DomainError::Unauthorized("invalid credentials".to_string()));
        }

        if !user_with_password.user.is_active() {
            return Err(DomainError::Unauthorized("user is inactive".to_string()));
        }

        let access_token = self.jwt.create_access_token(&user_with_password.user)?;
        let refresh_token = self.jwt.create_refresh_token(&user_with_password.user)?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: user_with_password.user,
        })
    }

    pub async fn refresh_tokens(&self, refresh_token: String) -> Result<LoginResponse, DomainError> {
        let claims = self.jwt.decode_token(&refresh_token)?;
        Self::validate_refresh(&claims)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| DomainError::Unauthorized("invalid token".to_string()))?;

        let user_with_password = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| DomainError::Unauthorized("invalid token".to_string()))?;

        if !user_with_password.user.is_active() {
            return Err(DomainError::Unauthorized("user is inactive".to_string()));
        }

        let access_token = self.jwt.create_access_token(&user_with_password.user)?;
        let refresh_token = self.jwt.create_refresh_token(&user_with_password.user)?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: user_with_password.user,
        })
    }

    fn validate_refresh(claims: &Claims) -> Result<(), DomainError> {
        if claims.token_type != TokenType::Refresh {
            return Err(DomainError::Unauthorized("invalid token".to_string()));
        }

        Ok(())
    }
}