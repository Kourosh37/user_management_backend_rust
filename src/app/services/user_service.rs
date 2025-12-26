use crate::domain::{AdminUpdateUser, DomainError, Role, UpdateProfile, User, UserRepository};
use uuid::Uuid;

pub struct UserService<R> {
    repo: R,
}

impl<R> UserService<R>
where
    R: UserRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<User, DomainError> {
        let user_with_password = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("user not found".to_string()))?;
        Ok(user_with_password.user)
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        input: UpdateProfile,
    ) -> Result<User, DomainError> {
        if let Some(ref email) = input.email {
            if let Some(existing) = self.repo.find_by_email(email).await? {
                if existing.user.id != user_id {
                    return Err(DomainError::Conflict("email already exists".to_string()));
                }
            }
        }

        if let Some(ref username) = input.username {
            if let Some(existing) = self.repo.find_by_username(username).await? {
                if existing.id != user_id {
                    return Err(DomainError::Conflict("username already exists".to_string()));
                }
            }
        }

        self.repo.update_profile(user_id, input).await
    }

    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>, DomainError> {
        self.repo.list(limit, offset).await
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        input: AdminUpdateUser,
    ) -> Result<User, DomainError> {
        if let Some(ref email) = input.email {
            if let Some(existing) = self.repo.find_by_email(email).await? {
                if existing.user.id != user_id {
                    return Err(DomainError::Conflict("email already exists".to_string()));
                }
            }
        }

        if let Some(ref username) = input.username {
            if let Some(existing) = self.repo.find_by_username(username).await? {
                if existing.id != user_id {
                    return Err(DomainError::Conflict("username already exists".to_string()));
                }
            }
        }

        self.repo.update_user(user_id, input).await
    }

    pub async fn set_user_role(&self, user_id: Uuid, role: Role) -> Result<User, DomainError> {
        self.repo.set_role(user_id, role).await
    }

    pub async fn deactivate_user(&self, user_id: Uuid) -> Result<(), DomainError> {
        self.repo.set_active(user_id, false).await
    }
}