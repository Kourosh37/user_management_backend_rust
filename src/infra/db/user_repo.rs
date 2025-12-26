use crate::domain::{AdminUpdateUser, DomainError, NewUser, Role, UpdateProfile, User, UserRepository, UserWithPassword};
use crate::infra::db::models::DbUser;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxUserRepository {
    pool: PgPool,
}

impl SqlxUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_db_user(db_user: DbUser) -> Result<User, DomainError> {
        User::try_from(db_user).map_err(DomainError::Internal)
    }

    fn map_db_user_with_password(db_user: DbUser) -> Result<UserWithPassword, DomainError> {
        let password_hash = db_user.password_hash.clone();
        let user = Self::map_db_user(db_user)?;
        Ok(UserWithPassword { user, password_hash })
    }

    fn map_db_error(error: sqlx::Error) -> DomainError {
        if let sqlx::Error::Database(db_error) = &error {
            if db_error.code().as_deref() == Some("23505") {
                return DomainError::Conflict("resource already exists".to_string());
            }
        }
        DomainError::Internal(error.to_string())
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<UserWithPassword>, DomainError> {
        let result = sqlx::query_as::<_, DbUser>(
            "SELECT id, email, username, password_hash, role, is_active, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_db_error)?;

        match result {
            Some(row) => Ok(Some(Self::map_db_user_with_password(row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let result = sqlx::query_as::<_, DbUser>(
            "SELECT id, email, username, password_hash, role, is_active, created_at, updated_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_db_error)?;

        match result {
            Some(row) => Ok(Some(Self::map_db_user(row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserWithPassword>, DomainError> {
        let result = sqlx::query_as::<_, DbUser>(
            "SELECT id, email, username, password_hash, role, is_active, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_db_error)?;

        match result {
            Some(row) => Ok(Some(Self::map_db_user_with_password(row)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, new_user: NewUser) -> Result<User, DomainError> {
        let result = sqlx::query_as::<_, DbUser>(
            "INSERT INTO users (id, email, username, password_hash, role, is_active) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, email, username, password_hash, role, is_active, created_at, updated_at",
        )
        .bind(Uuid::new_v4())
        .bind(new_user.email)
        .bind(new_user.username)
        .bind(new_user.password_hash)
        .bind(new_user.role.to_string())
        .bind(new_user.is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_db_error)?;

        Self::map_db_user(result)
    }

    async fn update_profile(&self, id: Uuid, input: UpdateProfile) -> Result<User, DomainError> {
        let result = sqlx::query_as::<_, DbUser>(
            "UPDATE users SET email = COALESCE($1, email), username = COALESCE($2, username), updated_at = NOW() WHERE id = $3 RETURNING id, email, username, password_hash, role, is_active, created_at, updated_at",
        )
        .bind(input.email)
        .bind(input.username)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_db_error)?;

        match result {
            Some(row) => Self::map_db_user(row),
            None => Err(DomainError::NotFound("user not found".to_string())),
        }
    }

    async fn update_user(&self, id: Uuid, input: AdminUpdateUser) -> Result<User, DomainError> {
        let role_value = input.role.map(|role| role.to_string());
        let result = sqlx::query_as::<_, DbUser>(
            "UPDATE users SET email = COALESCE($1, email), username = COALESCE($2, username), role = COALESCE($3, role), is_active = COALESCE($4, is_active), updated_at = NOW() WHERE id = $5 RETURNING id, email, username, password_hash, role, is_active, created_at, updated_at",
        )
        .bind(input.email)
        .bind(input.username)
        .bind(role_value)
        .bind(input.is_active)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_db_error)?;

        match result {
            Some(row) => Self::map_db_user(row),
            None => Err(DomainError::NotFound("user not found".to_string())),
        }
    }

    async fn set_role(&self, id: Uuid, role: Role) -> Result<User, DomainError> {
        let result = sqlx::query_as::<_, DbUser>(
            "UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2 RETURNING id, email, username, password_hash, role, is_active, created_at, updated_at",
        )
        .bind(role.to_string())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_db_error)?;

        match result {
            Some(row) => Self::map_db_user(row),
            None => Err(DomainError::NotFound("user not found".to_string())),
        }
    }

    async fn set_active(&self, id: Uuid, is_active: bool) -> Result<(), DomainError> {
        let affected = sqlx::query("UPDATE users SET is_active = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_active)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_db_error)?
            .rows_affected();

        if affected == 0 {
            return Err(DomainError::NotFound("user not found".to_string()));
        }

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>, DomainError> {
        let rows = sqlx::query_as::<_, DbUser>(
            "SELECT id, email, username, password_hash, role, is_active, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_db_error)?;

        rows.into_iter()
            .map(Self::map_db_user)
            .collect::<Result<Vec<_>, _>>()
    }
}