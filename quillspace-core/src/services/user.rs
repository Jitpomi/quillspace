use crate::types::{TenantId, User, UserRole, UserId};
use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio_postgres::{Row, Error as PgError};
use uuid::Uuid;

/// Helper function to convert a tokio-postgres Row to User
fn row_to_user(row: &Row) -> Result<User, PgError> {
    let role_str: String = row.try_get("role")?;
    let role = match role_str.as_str() {
        "Admin" => UserRole::Admin,
        "Editor" => UserRole::Editor,
        "Viewer" => UserRole::Viewer,
        _ => UserRole::Viewer, // Default fallback
    };

    Ok(User {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        role,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        is_active: row.try_get("is_active")?,
    })
}

/// Helper function to convert UserRole to string for database
fn user_role_to_string(role: &UserRole) -> &'static str {
    match role {
        UserRole::Admin => "Admin",
        UserRole::Editor => "Editor",
        UserRole::Viewer => "Viewer",
    }
}

/// User management service
#[derive(Clone)]
pub struct UserService {
    db: Pool,
}

impl UserService {
    pub fn new(db: Pool) -> Self {
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

        // Get database connection
        let client = self.db.get().await?;

        let query = r#"
            INSERT INTO users (id, tenant_id, email, name, role, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#;

        let role_str = user_role_to_string(&role);
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            &user_id,
            tenant_id.as_uuid(),
            &email,
            &name,
            &role_str,
            &now,
            &now,
            &true,
        ];

        let row = client.query_one(query, &params).await?;
        let user = row_to_user(&row)?;

        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<Option<User>> {
        let client = self.db.get().await?;

        let query = "SELECT * FROM users WHERE id = $1 AND tenant_id = $2";
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![user_id.as_uuid(), tenant_id.as_uuid()];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let user = row_to_user(&row)?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    /// Get user by email
    pub async fn get_user_by_email(
        &self,
        tenant_id: &TenantId,
        email: &str,
    ) -> Result<Option<User>> {
        let client = self.db.get().await?;

        let query = "SELECT * FROM users WHERE email = $1 AND tenant_id = $2 AND is_active = true";
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&email, tenant_id.as_uuid()];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let user = row_to_user(&row)?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
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
        let client = self.db.get().await?;

        let query = r#"
            UPDATE users 
            SET name = COALESCE($3, name),
                email = COALESCE($4, email),
                updated_at = $5
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#;

        let name_ref = name.as_deref();
        let email_ref = email.as_deref();
        
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            user_id.as_uuid(),
            tenant_id.as_uuid(),
            &name_ref,
            &email_ref,
            &now,
        ];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let user = row_to_user(&row)?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    /// Update user role
    pub async fn update_user_role(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        role: UserRole,
    ) -> Result<Option<User>> {
        let now = chrono::Utc::now();
        let client = self.db.get().await?;

        let query = r#"
            UPDATE users 
            SET role = $3, updated_at = $4
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#;

        let role_str = user_role_to_string(&role);
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            user_id.as_uuid(),
            tenant_id.as_uuid(),
            &role_str,
            &now,
        ];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let user = row_to_user(&row)?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    /// List users in tenant
    pub async fn list_users(
        &self,
        tenant_id: &TenantId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>> {
        let client = self.db.get().await?;

        let query = r#"
            SELECT * FROM users 
            WHERE tenant_id = $1 AND is_active = true 
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#;

        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            tenant_id.as_uuid(),
            &limit,
            &offset,
        ];

        let rows = client.query(query, &params).await?;
        let users: Result<Vec<User>, _> = rows.iter().map(row_to_user).collect();
        
        Ok(users?)
    }

    /// Deactivate user (soft delete)
    pub async fn deactivate_user(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<bool> {
        let now = chrono::Utc::now();
        let client = self.db.get().await?;

        let query = "UPDATE users SET is_active = false, updated_at = $3 WHERE id = $1 AND tenant_id = $2";
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            user_id.as_uuid(),
            tenant_id.as_uuid(),
            &now,
        ];

        let rows_affected = client.execute(query, &params).await?;
        
        Ok(rows_affected > 0)
    }

    /// Count users in tenant
    pub async fn count_users(&self, tenant_id: &TenantId) -> Result<i64> {
        let client = self.db.get().await?;

        let query = "SELECT COUNT(*) FROM users WHERE tenant_id = $1 AND is_active = true";
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![tenant_id.as_uuid()];

        let row = client.query_one(query, &params).await?;
        let count: i64 = row.get(0);
        
        Ok(count)
    }
}
