use crate::domain::{Role, User};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct DbUser {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<DbUser> for User {
    type Error = String;

    fn try_from(value: DbUser) -> Result<Self, Self::Error> {
        let role = Role::from_str(&value.role)?;
        Ok(User {
            id: value.id,
            email: value.email,
            username: value.username,
            role,
            is_active: value.is_active,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}
