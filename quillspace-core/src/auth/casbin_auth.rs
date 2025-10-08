use casbin::{prelude::*, DefaultModel};
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::http::StatusCode;
use crate::types::UserRole;

/// Casbin-based authorization manager
#[derive(Clone)]
pub struct CasbinAuthorizer {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl CasbinAuthorizer {
    /// Initialize Casbin enforcer with RBAC model following official docs
    pub async fn new() -> anyhow::Result<Self> {
        // Create RBAC model exactly as shown in Casbin docs
        const MODEL_CONF: &str = r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
"#;

        // Create model from string using DefaultModel::from_str as shown in docs
        let model = DefaultModel::from_str(MODEL_CONF).await?;
        let mut enforcer = Enforcer::new(model, MemoryAdapter::default()).await?;

        // Define role hierarchy (admin inherits editor, editor inherits viewer)
        enforcer.add_grouping_policy(vec!["admin".to_string(), "editor".to_string()]).await?;
        enforcer.add_grouping_policy(vec!["editor".to_string(), "viewer".to_string()]).await?;

        // Define permissions for each role
        // Viewer permissions
        enforcer.add_policy(vec!["viewer".to_string(), "content".to_string(), "read".to_string()]).await?;
        enforcer.add_policy(vec!["viewer".to_string(), "users".to_string(), "read".to_string()]).await?;
        enforcer.add_policy(vec!["viewer".to_string(), "analytics".to_string(), "read".to_string()]).await?;

        // Editor permissions (inherits viewer + write content)
        enforcer.add_policy(vec!["editor".to_string(), "content".to_string(), "write".to_string()]).await?;
        enforcer.add_policy(vec!["editor".to_string(), "content".to_string(), "update".to_string()]).await?;
        enforcer.add_policy(vec!["editor".to_string(), "content".to_string(), "publish".to_string()]).await?;
        enforcer.add_policy(vec!["editor".to_string(), "content".to_string(), "archive".to_string()]).await?;

        // Admin permissions (inherits editor + admin operations)
        enforcer.add_policy(vec!["admin".to_string(), "users".to_string(), "write".to_string()]).await?;
        enforcer.add_policy(vec!["admin".to_string(), "users".to_string(), "update".to_string()]).await?;
        enforcer.add_policy(vec!["admin".to_string(), "users".to_string(), "delete".to_string()]).await?;
        enforcer.add_policy(vec!["admin".to_string(), "tenants".to_string(), "read".to_string()]).await?;
        enforcer.add_policy(vec!["admin".to_string(), "tenants".to_string(), "write".to_string()]).await?;
        enforcer.add_policy(vec!["admin".to_string(), "tenants".to_string(), "update".to_string()]).await?;
        enforcer.add_policy(vec!["admin".to_string(), "content".to_string(), "delete".to_string()]).await?;

        Ok(Self {
            enforcer: Arc::new(RwLock::new(enforcer)),
        })
    }

    /// Check if a user has permission to perform an action on a resource
    pub async fn enforce(&self, user_role: &UserRole, resource: &str, action: &str) -> anyhow::Result<bool> {
        let role_str = match user_role {
            UserRole::Admin => "admin",
            UserRole::Editor => "editor", 
            UserRole::Viewer => "viewer",
        };

        let enforcer = self.enforcer.read().await;
        let result = enforcer.enforce(vec![role_str, resource, action])?;
        Ok(result)
    }

    /// Require permission, returning 403 if not authorized
    pub async fn require_permission(&self, user_role: &UserRole, resource: &str, action: &str) -> std::result::Result<(), StatusCode> {
        match self.enforce(user_role, resource, action).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(StatusCode::FORBIDDEN),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    /// Add a custom policy (for dynamic permissions)
    pub async fn add_policy(&self, subject: &str, object: &str, action: &str) -> anyhow::Result<bool> {
        let mut enforcer = self.enforcer.write().await;
        let result = enforcer.add_policy(vec![subject.to_string(), object.to_string(), action.to_string()]).await?;
        Ok(result)
    }

    /// Remove a policy
    pub async fn remove_policy(&self, subject: &str, object: &str, action: &str) -> anyhow::Result<bool> {
        let mut enforcer = self.enforcer.write().await;
        let result = enforcer.remove_policy(vec![subject.to_string(), object.to_string(), action.to_string()]).await?;
        Ok(result)
    }

    /// Get all policies for a subject
    pub async fn get_permissions_for_user(&self, user_role: &UserRole) -> Vec<Vec<String>> {
        let role_str = match user_role {
            UserRole::Admin => "admin",
            UserRole::Editor => "editor",
            UserRole::Viewer => "viewer",
        };

        let enforcer = self.enforcer.read().await;
        enforcer.get_permissions_for_user(role_str, None)
    }
}

/// Resource types for authorization
pub enum Resource {
    Content,
    Users,
    Tenants,
    Analytics,
}

impl Resource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Resource::Content => "content",
            Resource::Users => "users", 
            Resource::Tenants => "tenants",
            Resource::Analytics => "analytics",
        }
    }
}

/// Action types for authorization
pub enum Action {
    Read,
    Write,
    Update,
    Delete,
    Publish,
    Archive,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Write => "write",
            Action::Update => "update", 
            Action::Delete => "delete",
            Action::Publish => "publish",
            Action::Archive => "archive",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_casbin_permissions() {
        let auth = CasbinAuthorizer::new().await.unwrap();

        // Test viewer permissions
        assert!(auth.enforce(&UserRole::Viewer, "content", "read").await.unwrap());
        assert!(!auth.enforce(&UserRole::Viewer, "content", "write").await.unwrap());
        assert!(!auth.enforce(&UserRole::Viewer, "users", "write").await.unwrap());

        // Test editor permissions  
        assert!(auth.enforce(&UserRole::Editor, "content", "read").await.unwrap());
        assert!(auth.enforce(&UserRole::Editor, "content", "write").await.unwrap());
        assert!(!auth.enforce(&UserRole::Editor, "users", "write").await.unwrap());

        // Test admin permissions
        assert!(auth.enforce(&UserRole::Admin, "content", "read").await.unwrap());
        assert!(auth.enforce(&UserRole::Admin, "content", "write").await.unwrap());
        assert!(auth.enforce(&UserRole::Admin, "users", "write").await.unwrap());
        assert!(auth.enforce(&UserRole::Admin, "tenants", "write").await.unwrap());
    }
}
