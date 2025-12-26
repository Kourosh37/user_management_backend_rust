use crate::domain::errors::DomainError;
use crate::domain::role::Role;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub role: Role,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

#[derive(Debug, Clone)]
pub struct UserWithPassword {
    pub user: User,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub is_active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateProfile {
    pub email: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AdminUpdateUser {
    pub email: Option<String>,
    pub username: Option<String>,
    pub role: Option<Role>,
    pub is_active: Option<bool>,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<UserWithPassword>, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserWithPassword>, DomainError>;
    async fn create(&self, new_user: NewUser) -> Result<User, DomainError>;
    async fn update_profile(&self, id: Uuid, input: UpdateProfile) -> Result<User, DomainError>;
    async fn update_user(&self, id: Uuid, input: AdminUpdateUser) -> Result<User, DomainError>;
    async fn set_role(&self, id: Uuid, role: Role) -> Result<User, DomainError>;
    async fn set_active(&self, id: Uuid, is_active: bool) -> Result<(), DomainError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>, DomainError>;
}