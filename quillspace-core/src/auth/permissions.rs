use crate::types::UserRole;
use axum::http::StatusCode;

/// Permission levels for different operations
#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    /// Can read content and basic operations
    Read,
    /// Can create and edit content
    Write,
    /// Can manage users and settings
    Admin,
}

/// Check if a user role has the required permission
pub fn has_permission(user_role: &UserRole, required_permission: Permission) -> bool {
    match (user_role, required_permission) {
        // Admin has all permissions
        (UserRole::Admin, _) => true,
        
        // Editor has read and write permissions
        (UserRole::Editor, Permission::Read) => true,
        (UserRole::Editor, Permission::Write) => true,
        (UserRole::Editor, Permission::Admin) => false,
        
        // Viewer only has read permission
        (UserRole::Viewer, Permission::Read) => true,
        (UserRole::Viewer, Permission::Write) => false,
        (UserRole::Viewer, Permission::Admin) => false,
    }
}

/// Require a specific permission, returning 403 if not authorized
pub fn require_permission(user_role: &UserRole, required_permission: Permission) -> Result<(), StatusCode> {
    if has_permission(user_role, required_permission) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// Extract user role from JWT claims for permission checking
pub fn extract_user_role_from_jwt(claims: &crate::auth::jwt::Claims) -> Result<UserRole, StatusCode> {
    match claims.role.as_str() {
        "admin" => Ok(UserRole::Admin),
        "editor" => Ok(UserRole::Editor),
        "viewer" => Ok(UserRole::Viewer),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_permissions() {
        assert!(has_permission(&UserRole::Admin, Permission::Read));
        assert!(has_permission(&UserRole::Admin, Permission::Write));
        assert!(has_permission(&UserRole::Admin, Permission::Admin));
    }

    #[test]
    fn test_editor_permissions() {
        assert!(has_permission(&UserRole::Editor, Permission::Read));
        assert!(has_permission(&UserRole::Editor, Permission::Write));
        assert!(!has_permission(&UserRole::Editor, Permission::Admin));
    }

    #[test]
    fn test_viewer_permissions() {
        assert!(has_permission(&UserRole::Viewer, Permission::Read));
        assert!(!has_permission(&UserRole::Viewer, Permission::Write));
        assert!(!has_permission(&UserRole::Viewer, Permission::Admin));
    }
}
