use crate::types::{TenantId, User, UserRole, UserId};
use anyhow::Result;
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

/// User management service
#[derive(Clone)]
pub struct UserService {
    db: PgPool,
}

impl UserService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        tenant_id: &TenantId,
        email: String,
        name: String,
        role: UserRole,
    ) -> Result<User> {
        let user_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, tenant_id, email, name, role, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(tenant_id.as_uuid())
        .bind(&email)
        .bind(&name)
        .bind(role)
        .bind(now)
        .bind(now)
        .bind(true)
        .fetch_one(&self.db)
        .await?;

        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND tenant_id = $2"
        )
        .bind(user_id.as_uuid())
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    /// Get user by email
    pub async fn get_user_by_email(
        &self,
        tenant_id: &TenantId,
        email: &str,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND tenant_id = $2 AND is_active = true"
        )
        .bind(email)
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    /// Update user
    pub async fn update_user(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        name: Option<String>,
        email: Option<String>,
    ) -> Result<Option<User>> {
        let now = chrono::Utc::now();

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET name = COALESCE($3, name),
                email = COALESCE($4, email),
                updated_at = $5
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#
        )
        .bind(user_id.as_uuid())
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(email)
        .bind(now)
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    /// Update user role
    pub async fn update_user_role(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        role: UserRole,
    ) -> Result<Option<User>> {
        let now = chrono::Utc::now();

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET role = $3, updated_at = $4
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#
        )
        .bind(user_id.as_uuid())
        .bind(tenant_id.as_uuid())
        .bind(role)
        .bind(now)
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    /// List users in tenant
    pub async fn list_users(
        &self,
        tenant_id: &TenantId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users 
            WHERE tenant_id = $1 AND is_active = true 
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(tenant_id.as_uuid())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// Deactivate user (soft delete)
    pub async fn deactivate_user(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<bool> {
        let now = chrono::Utc::now();

        let result = sqlx::query!(
            "UPDATE users SET is_active = false, updated_at = $3 WHERE id = $1 AND tenant_id = $2",
            user_id.as_uuid(),
            tenant_id.as_uuid(),
            now
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Count users in tenant
    pub async fn count_users(&self, tenant_id: &TenantId) -> Result<i64> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE tenant_id = $1 AND is_active = true",
            tenant_id.as_uuid()
        )
        .fetch_one(&self.db)
        .await?;

        Ok(count.unwrap_or(0))
    }
}
