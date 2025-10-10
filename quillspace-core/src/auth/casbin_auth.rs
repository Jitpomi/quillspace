use casbin::{prelude::*, DefaultModel};
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::http::StatusCode;
use crate::types::UserRole;
use tracing::{info, warn};

/// Casbin-based authorization manager
#[derive(Clone)]
pub struct CasbinAuthorizer {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl CasbinAuthorizer {
    /// Initialize Casbin enforcer with RBAC model following official docs
    pub async fn new() -> anyhow::Result<Self> {
        // Create RBAC model with tenant isolation
        const MODEL_CONF: &str = r#"
[request_definition]
r = sub, obj, act, tenant

[policy_definition]
p = sub, obj, act, tenant

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act && (r.tenant == p.tenant || p.tenant == "*")
"#;

        // Create model from string using DefaultModel::from_str as shown in docs
        let model = DefaultModel::from_str(MODEL_CONF).await?;
        let mut enforcer = Enforcer::new(model, MemoryAdapter::default()).await?;

        // Define role hierarchy (admin inherits editor, editor inherits viewer)
        enforcer.add_grouping_policy(vec!["admin".to_string(), "editor".to_string()]).await?;
        enforcer.add_grouping_policy(vec!["editor".to_string(), "viewer".to_string()]).await?;

        // Define comprehensive permissions for each role with wildcard tenant
        // VIEWER PERMISSIONS (read-only access)
        let viewer_permissions = vec![
            ("content", "read"), ("sites", "read"), ("pages", "read"), 
            ("templates", "read"), ("assets", "read"), ("analytics", "read")
        ];
        for (resource, action) in viewer_permissions {
            enforcer.add_policy(vec!["viewer".to_string(), resource.to_string(), action.to_string(), "*".to_string()]).await?;
        }

        // EDITOR PERMISSIONS (inherits viewer + content creation/editing)
        let editor_permissions = vec![
            ("content", "write"), ("content", "update"), ("content", "publish"),
            ("sites", "write"), ("sites", "update"), ("sites", "publish"),
            ("pages", "write"), ("pages", "update"), ("pages", "publish"),
            ("templates", "write"), ("templates", "update"),
            ("assets", "write"), ("assets", "update")
        ];
        for (resource, action) in editor_permissions {
            enforcer.add_policy(vec!["editor".to_string(), resource.to_string(), action.to_string(), "*".to_string()]).await?;
        }

        // ADMIN PERMISSIONS (inherits editor + administrative operations)
        let admin_permissions = vec![
            ("users", "read"), ("users", "write"), ("users", "update"), ("users", "delete"),
            ("tenants", "read"), ("tenants", "update"), ("tenants", "configure"),
            ("sites", "delete"), ("pages", "delete"), ("content", "delete"),
            ("templates", "delete"), ("assets", "delete"),
            ("analytics", "admin"), ("security", "admin")
        ];
        for (resource, action) in admin_permissions {
            enforcer.add_policy(vec!["admin".to_string(), resource.to_string(), action.to_string(), "*".to_string()]).await?;
        }

        info!("Casbin authorizer initialized with comprehensive RBAC policies");

        Ok(Self {
            enforcer: Arc::new(RwLock::new(enforcer)),
        })
    }

    /// Check if a user has permission to perform an action on a resource within a tenant
    pub async fn enforce(&self, user_role: &UserRole, resource: &str, action: &str, tenant_id: &str) -> anyhow::Result<bool> {
        let role_str = match user_role {
            UserRole::Admin => "admin",
            UserRole::Editor => "editor", 
            UserRole::Viewer => "viewer",
        };

        let enforcer = self.enforcer.read().await;
        let result = enforcer.enforce(vec![role_str, resource, action, tenant_id])?;
        Ok(result)
    }

    /// Require permission, returning 403 if not authorized
    pub async fn require_permission(&self, user_role: &UserRole, resource: &str, action: &str, tenant_id: &str) -> std::result::Result<(), StatusCode> {
        match self.enforce(user_role, resource, action, tenant_id).await {
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

/// Resource schemas for authorization
pub enum Resource {
    Content,
    Users,
    Tenants,
    Analytics,
    Sites,
    Pages,
    Templates,
    Assets,
    Security,
}

impl Resource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Resource::Content => "content",
            Resource::Users => "users", 
            Resource::Tenants => "tenants",
            Resource::Analytics => "analytics",
            Resource::Sites => "sites",
            Resource::Pages => "pages",
            Resource::Templates => "templates",
            Resource::Assets => "assets",
            Resource::Security => "security",
        }
    }
}

/// Action schemas for authorization
pub enum Action {
    Read,
    Write,
    Update,
    Delete,
    Publish,
    Archive,
    Configure,
    Admin,
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
            Action::Configure => "configure",
            Action::Admin => "admin",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_casbin_permissions() {
        let auth = CasbinAuthorizer::new().await.expect("Failed to create Casbin authorizer");
        let test_tenant = "11111111-1111-1111-1111-111111111111";

        // Test viewer permissions
        assert!(auth.enforce(&UserRole::Viewer, "content", "read", test_tenant).await.expect("Viewer read test failed"));
        assert!(!auth.enforce(&UserRole::Viewer, "content", "write", test_tenant).await.expect("Viewer write test failed"));
        assert!(!auth.enforce(&UserRole::Viewer, "users", "write", test_tenant).await.expect("Viewer user write test failed"));

        // Test editor permissions  
        assert!(auth.enforce(&UserRole::Editor, "content", "read", test_tenant).await.expect("Editor read test failed"));
        assert!(auth.enforce(&UserRole::Editor, "content", "write", test_tenant).await.expect("Editor write test failed"));
        assert!(!auth.enforce(&UserRole::Editor, "users", "write", test_tenant).await.expect("Editor user write test failed"));

        // Test admin permissions
        assert!(auth.enforce(&UserRole::Admin, "content", "read", test_tenant).await.expect("Admin content read test failed"));
        assert!(auth.enforce(&UserRole::Admin, "content", "write", test_tenant).await.expect("Admin content write test failed"));
        assert!(auth.enforce(&UserRole::Admin, "users", "write", test_tenant).await.expect("Admin user write test failed"));
        assert!(auth.enforce(&UserRole::Admin, "security", "admin", test_tenant).await.expect("Admin security test failed"));

        // Test new resources
        assert!(auth.enforce(&UserRole::Editor, "templates", "write", test_tenant).await.expect("Editor templates test failed"));
        assert!(auth.enforce(&UserRole::Admin, "templates", "delete", test_tenant).await.expect("Admin templates delete test failed"));
        assert!(!auth.enforce(&UserRole::Viewer, "templates", "write", test_tenant).await.expect("Viewer templates write test failed"));
    }
}
