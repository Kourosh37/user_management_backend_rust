use crate::config::AppConfig;
use crate::domain::{DomainError, User};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub token_type: TokenType,
    pub exp: usize,
}

#[derive(Clone)]
pub struct JwtService {
    secret: String,
    access_token_minutes: i64,
    refresh_token_days: i64,
}

impl JwtService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            secret: config.jwt_secret.clone(),
            access_token_minutes: config.access_token_minutes,
            refresh_token_days: config.refresh_token_days,
        }
    }

    pub fn create_access_token(&self, user: &User) -> Result<String, DomainError> {
        self.create_token(user, TokenType::Access)
    }

    pub fn create_refresh_token(&self, user: &User) -> Result<String, DomainError> {
        self.create_token(user, TokenType::Refresh)
    }

    pub fn decode_token(&self, token: &str) -> Result<Claims, DomainError> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|err| DomainError::Unauthorized(err.to_string()))?;

        Ok(token_data.claims)
    }

    fn create_token(&self, user: &User, token_type: TokenType) -> Result<String, DomainError> {
        let expiration = match token_type {
            TokenType::Access => Utc::now() + Duration::minutes(self.access_token_minutes),
            TokenType::Refresh => Utc::now() + Duration::days(self.refresh_token_days),
        };

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.to_string(),
            token_type,
            exp: expiration.timestamp() as usize,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|err| DomainError::Internal(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Role;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn jwt_round_trip() {
        let config = AppConfig {
            app_host: "0.0.0.0".to_string(),
            app_port: 8080,
            database_url: "postgres://localhost".to_string(),
            jwt_secret: "secret".to_string(),
            access_token_minutes: 10,
            refresh_token_days: 7,
            cors_allowed_origins: vec!["http://localhost:3000".to_string()],
        };

        let service = JwtService::new(&config);
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "tester".to_string(),
            role: Role::User,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let token = service.create_access_token(&user).unwrap();
        let claims = service.decode_token(&token).unwrap();

        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.role, "user");
        assert_eq!(claims.token_type, TokenType::Access);
    }
}
