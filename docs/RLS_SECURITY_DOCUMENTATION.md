# QuillSpace Row-Level Security (RLS) Documentation

## Overview

QuillSpace implements a sophisticated multi-level Row-Level Security system that provides:
1. **Tenant-level isolation** (always enforced)
2. **Configurable user-level isolation** (controlled by tenant owners)
3. **Role-based access controls** within tenants
4. **Admin override capabilities** with audit logging

## Security Architecture

### Multi-Level Security Model

```
┌─────────────────────────────────────────────────────────────┐
│                    SYSTEM LEVEL                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                TENANT LEVEL                         │    │
│  │  ┌─────────────────────────────────────────────┐    │    │
│  │  │              USER LEVEL                     │    │    │
│  │  │  ┌─────────────────────────────────────┐    │    │    │
│  │  │  │           RESOURCE LEVEL            │    │    │    │
│  │  │  │   (content, sites, pages, etc.)     │    │    │    │
│  │  │  └─────────────────────────────────────┘    │    │    │
│  │  └─────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## RLS Context Variables

### Required Session Variables

| Variable | Purpose | Format | Example |
|----------|---------|--------|---------|
| `app.current_tenant_id` | Tenant isolation | UUID | `11111111-1111-1111-1111-111111111111` |
| `rls.user_id` | User isolation | UUID | `bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb` |

### Setting Context Variables

```sql
-- Set tenant context (required for all operations)
SELECT set_config('app.current_tenant_id', 'tenant-uuid-here', true);

-- Set user context (required for user-level isolation)
SELECT set_config('rls.user_id', 'user-uuid-here', true);
```

## Isolation Modes

### 1. Collaborative Mode (Default)
- **Behavior**: All users within tenant can see each other's data
- **Use Case**: Team workspaces, collaborative environments
- **Security**: Tenant-level isolation only

```sql
-- Enable collaborative mode
SELECT set_tenant_user_isolation('collaborative');
```

### 2. Isolated Mode
- **Behavior**: Users can only see their own data (admins see all)
- **Use Case**: Individual workspaces, strict user separation
- **Security**: Tenant + user-level isolation

```sql
-- Enable isolated mode
SELECT set_tenant_user_isolation('isolated');
```

### 3. Role-Based Mode
- **Behavior**: Access based on user roles
  - **Admins**: See all data in tenant
  - **Editors**: See published content + own content
  - **Authors**: See only own content
- **Use Case**: Publishing workflows, hierarchical organizations

```sql
-- Enable role-based mode
SELECT set_tenant_user_isolation('role_based');
```

## Table-Specific RLS Policies

### 1. Users Table

**Policy Name**: `configurable_user_isolation_users`

**Read Access**:
- Users can always see themselves
- Additional visibility based on isolation mode:
  - **Collaborative**: See all users in tenant
  - **Isolated**: Admins see all, others see only themselves
  - **Role-based**: Admins and editors see all, authors see only themselves

**Write Access**:
- Users can modify their own records
- Admins can modify any user in their tenant

```sql
-- Policy Implementation
CREATE POLICY configurable_user_isolation_users ON users
    FOR ALL
    USING (
        tenant_id = current_tenant_id() AND (
            id = current_user_id() OR
            (get_tenant_isolation_mode() = 'collaborative') OR
            (get_tenant_isolation_mode() = 'isolated' AND EXISTS (
                SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role = 'admin'
            )) OR
            (get_tenant_isolation_mode() = 'role_based' AND EXISTS (
                SELECT 1 FROM users cu WHERE cu.id = current_user_id() AND cu.role IN ('admin', 'editor')
            ))
        )
    );
```

### 2. Content Table

**Policy Name**: `configurable_user_isolation_content`

**Read Access**:
- Content owners can always see their content
- Additional access based on isolation mode and user roles

**Write Access**:
- Users can create content as themselves
- Admins can create content for others

```sql
-- Policy Implementation
CREATE POLICY configurable_user_isolation_content ON content
    FOR ALL
    USING (
        tenant_id = current_tenant_id() AND (
            author_id = current_user_id() OR
            user_can_access_resource(author_id, tenant_id)
        )
    );
```

### 3. Sites Table

**Policy Name**: `configurable_user_isolation_sites`

**Read Access**:
- Follows tenant isolation rules
- In collaborative mode: all users see all sites
- In isolated/role-based: admin override applies

### 4. Pages Table

**Policy Name**: `tenant_isolation_pages`

**Access Rules**:
- Pages inherit access from their parent site
- Must belong to user's tenant
- User-level access determined by site ownership

### 5. Assets Table

**Policy Name**: `tenant_isolation_assets`

**Access Rules**:
- Assets scoped to tenant
- User-level access based on isolation mode

### 6. Templates Table

**Policy Name**: `tenant_isolation_templates`

**Access Rules**:
- Templates scoped to tenant
- Public templates visible across tenants

### 7. Widgets Table

**Policy Names**: 
- `public_widgets_read`: Public approved widgets visible to all
- `widget_owner_policy`: Users can manage their own widgets

**Access Rules**:
- Public widgets: visible to all tenants
- Private widgets: only visible to creator and tenant admins

## Helper Functions

### Core Functions

#### `current_tenant_id()`
Returns the current tenant UUID from session context.

```sql
SELECT current_tenant_id();
-- Returns: 11111111-1111-1111-1111-111111111111
```

#### `current_user_id()`
Returns the current user UUID from session context.

```sql  
SELECT current_user_id();
-- Returns: bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb
```

#### `get_tenant_isolation_mode(tenant_id)`
Returns the isolation mode for specified tenant (or current tenant).

```sql
SELECT get_tenant_isolation_mode();
-- Returns: 'collaborative' | 'isolated' | 'role_based'
```

#### `user_can_access_resource(owner_id, tenant_id)`
Determines if current user can access resource owned by another user.

```sql
SELECT user_can_access_resource('owner-uuid', 'tenant-uuid');
-- Returns: true | false
```

### Management Functions

#### `set_tenant_user_isolation(mode)`
Changes tenant isolation mode (admin only).

```sql
-- Only tenant admins can execute
SELECT set_tenant_user_isolation('isolated');
-- Returns: 'Tenant isolation mode updated to: isolated'
```

#### `admin_set_tenant_context(tenant_id)`
Allows admin users to set arbitrary tenant contexts (with audit logging).

```sql
-- Only admin users can execute
SELECT admin_set_tenant_context('11111111-1111-1111-1111-111111111111');
```

## Security Enforcement

### Database Level

1. **Table Ownership**: All tables owned by `postgres` user
2. **Force RLS**: `FORCE ROW LEVEL SECURITY` enabled on all multi-tenant tables
3. **User Privileges**: Application users have limited `SELECT/INSERT/UPDATE/DELETE` only
4. **Admin Restrictions**: Even admin users cannot bypass RLS without proper context

### Application Level Requirements

1. **Context Setting**: Applications MUST set both tenant and user context
2. **Session Management**: Context variables are session-scoped
3. **Authentication**: User identity must be verified before setting context
4. **Authorization**: User permissions must be checked before setting context

## User Roles and Permissions

### Role Hierarchy

| Role | Tenant Access | User Access | Content Access | Admin Functions |
|------|---------------|-------------|----------------|-----------------|
| `admin` | All data in tenant | All users in tenant | All content in tenant | ✅ Can change isolation |
| `editor` | Tenant data only | Based on isolation mode | Published + own content | ❌ No admin functions |
| `author` | Tenant data only | Based on isolation mode | Own content only | ❌ No admin functions |
| `user` | Tenant data only | Based on isolation mode | Based on isolation mode | ❌ No admin functions |

### Permission Matrix by Isolation Mode

#### Collaborative Mode
| Role | See Other Users | See Other Content | Edit Other Content |
|------|----------------|-------------------|-------------------|
| admin | ✅ | ✅ | ✅ |
| editor | ✅ | ✅ | ✅ |
| author | ✅ | ✅ | ❌ |
| user | ✅ | ✅ | ❌ |

#### Isolated Mode
| Role | See Other Users | See Other Content | Edit Other Content |
|------|----------------|-------------------|-------------------|
| admin | ✅ | ✅ | ✅ |
| editor | ❌ | ❌ | ❌ |
| author | ❌ | ❌ | ❌ |
| user | ❌ | ❌ | ❌ |

#### Role-Based Mode
| Role | See Other Users | See Other Content | Edit Other Content |
|------|----------------|-------------------|-------------------|
| admin | ✅ | ✅ | ✅ |
| editor | ✅ | ✅ Published | ✅ Published |
| author | ❌ | ❌ | ❌ |
| user | ❌ | ❌ | ❌ |

## Monitoring and Auditing

### Audit Tables

#### `admin_audit_log`
Tracks all administrative actions including:
- Schema changes
- RLS policy modifications  
- Isolation mode changes
- Cross-tenant context switches

```sql
-- View recent admin actions
SELECT * FROM admin_audit_log 
ORDER BY timestamp DESC 
LIMIT 10;
```

#### `audit_logs`
General audit trail for application actions.

### Monitoring Views

#### `tenant_isolation_settings`
Shows current isolation configuration for tenant.

```sql
-- View current tenant isolation settings
SELECT * FROM tenant_isolation_settings;
```

#### `tenant_stats`
Provides overview of tenant data and user counts.

```sql
-- View tenant statistics
SELECT * FROM tenant_stats;
```

## Security Testing

### Test Scenarios

#### 1. Cross-Tenant Access Prevention
```sql
-- Set Tenant A context
SELECT set_config('app.current_tenant_id', 'tenant-a-uuid', true);
SELECT set_config('rls.user_id', 'user-a-uuid', true);

-- Try to access Tenant B data (should return 0 results)
SELECT COUNT(*) FROM content WHERE tenant_id = 'tenant-b-uuid';
-- Expected: 0
```

#### 2. User Isolation Testing
```sql
-- Test isolated mode
SELECT set_tenant_user_isolation('isolated');

-- Set non-admin user context
SELECT set_config('rls.user_id', 'regular-user-uuid', true);

-- Should only see own content
SELECT COUNT(*) FROM content WHERE author_id != current_user_id();
-- Expected: 0 (unless user is admin)
```

#### 3. RLS Bypass Prevention
```sql
-- Attempt to disable RLS (should fail)
ALTER TABLE content DISABLE ROW LEVEL SECURITY;
-- Expected: ERROR: must be owner of table content
```

### Automated Testing

#### Test Functions
```sql
-- Test tenant isolation
SELECT test_tenant_isolation();

-- Test user isolation modes
SELECT test_user_isolation_modes();

-- Test admin override capabilities
SELECT test_admin_overrides();
```

## Troubleshooting

### Common Issues

#### 1. No Data Visible
**Cause**: Missing or incorrect context variables
**Solution**: 
```sql
-- Check current context
SELECT current_tenant_id(), current_user_id();

-- Set correct context
SELECT set_config('app.current_tenant_id', 'correct-tenant-uuid', true);
SELECT set_config('rls.user_id', 'correct-user-uuid', true);
```

#### 2. Permission Denied Errors
**Cause**: User lacks necessary role or isolation mode prevents access
**Solution**:
```sql
-- Check user role
SELECT role FROM users WHERE id = current_user_id();

-- Check isolation mode
SELECT get_tenant_isolation_mode();

-- Verify user permissions
SELECT user_can_access_resource('target-user-uuid', current_tenant_id());
```

#### 3. Admin Cannot Change Settings
**Cause**: User is not admin or context not set
**Solution**:
```sql
-- Verify admin status
SELECT role FROM users 
WHERE id = current_user_id() 
AND tenant_id = current_tenant_id();

-- Ensure both contexts are set
SELECT current_tenant_id(), current_user_id();
```

### Debugging Queries

#### Check RLS Policy Status
```sql
SELECT 
    schemaname,
    tablename,
    policyname,
    cmd,
    qual
FROM pg_policies 
WHERE tablename IN ('users', 'content', 'sites', 'pages')
ORDER BY tablename, policyname;
```

#### Verify Table Security
```sql
SELECT 
    tablename,
    tableowner,
    rowsecurity as rls_enabled
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY tablename;
```

#### Check User Privileges
```sql
SELECT 
    rolname,
    rolsuper,
    rolbypassrls,
    rolcreaterole,
    rolcreatedb
FROM pg_roles 
WHERE rolname IN ('quillspace', 'quillspace_admin', 'postgres');
```

## Best Practices

### Application Development

1. **Always Set Context**: Set both tenant and user context for every request
2. **Validate Context**: Verify user belongs to tenant before setting context
3. **Handle Errors**: Gracefully handle RLS policy violations
4. **Audit Actions**: Log all context changes and admin actions
5. **Test Isolation**: Regularly test cross-tenant and cross-user access prevention

### Database Administration

1. **Monitor Policies**: Regular review of RLS policies and their effectiveness
2. **Audit Logs**: Monitor admin_audit_log for suspicious activities
3. **Performance**: Monitor query performance impact of RLS policies
4. **Backup Security**: Ensure backups maintain RLS policy integrity
5. **Updates**: Test RLS policies after any schema changes

### Security Operations

1. **Regular Testing**: Automated testing of isolation boundaries
2. **Penetration Testing**: Regular security audits of RLS implementation
3. **Incident Response**: Procedures for handling RLS policy violations
4. **Access Reviews**: Regular review of user roles and permissions
5. **Compliance**: Ensure RLS implementation meets regulatory requirements

## Migration and Deployment

### Initial Setup
```sql
-- Run in order:
1. /migrations/001_initial_schema.sql
2. /migrations/002_row_level_security.sql  
3. /migrations/004_web_builder_schema.sql
4. /migrations/999_security_hardening.sql
5. /scripts/configurable-user-isolation.sql
6. /scripts/admin-security-hardening.sql
```

### Production Deployment Checklist

- [ ] All RLS policies created and enabled
- [ ] FORCE RLS enabled on all multi-tenant tables
- [ ] Application user privileges restricted
- [ ] Admin audit logging configured
- [ ] Default isolation mode set for all tenants
- [ ] Context setting implemented in application
- [ ] Security testing completed
- [ ] Performance testing with RLS completed
- [ ] Backup and recovery procedures tested
- [ ] Monitoring and alerting configured

## Compliance and Standards

### Security Standards Met
- **Multi-tenancy**: Complete tenant data isolation
- **Principle of Least Privilege**: Users have minimal necessary access
- **Defense in Depth**: Multiple security layers (database + application)
- **Audit Trail**: Complete logging of administrative actions
- **Access Control**: Role-based and configurable access controls

### Regulatory Compliance
- **GDPR**: Data isolation supports data protection requirements
- **SOC 2**: Access controls and audit logging support compliance
- **HIPAA**: Data isolation suitable for healthcare applications
- **PCI DSS**: Security controls support payment card data protection

---

**Document Version**: 1.0  
**Last Updated**: 2025-10-09  
**Reviewed By**: Security Team  
**Next Review**: 2025-11-09
