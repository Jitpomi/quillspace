# QuillSpace RLS Quick Reference

## Essential Commands

### Set Security Context
```sql
-- REQUIRED: Set tenant context (always needed)
SELECT set_config('app.current_tenant_id', 'your-tenant-uuid', true);

-- REQUIRED: Set user context (needed for user-level isolation)
SELECT set_config('rls.user_id', 'your-user-uuid', true);
```

### Check Current Context
```sql
SELECT current_tenant_id(), current_user_id();
```

### Isolation Mode Management
```sql
-- Check current mode
SELECT get_tenant_isolation_mode();

-- Change mode (admin only)
SELECT set_tenant_user_isolation('collaborative'); -- or 'isolated' or 'role_based'

-- View settings
SELECT * FROM tenant_isolation_settings;
```

## Isolation Modes Cheat Sheet

| Mode | Users See | Content Access | Use Case |
|------|-----------|----------------|----------|
| `collaborative` | All users in tenant | All content in tenant | Team workspace |
| `isolated` | Only themselves (admins see all) | Only own content (admins see all) | Individual accounts |
| `role_based` | Based on role | Based on role + status | Publishing workflow |

## Role Permissions

| Role | Admin Functions | Cross-User Access | Content Creation |
|------|----------------|-------------------|------------------|
| `admin` | ✅ Full access | ✅ Always | ✅ For anyone |
| `editor` | ❌ No admin | 📋 Mode dependent | ✅ Own content |
| `author` | ❌ No admin | 📋 Mode dependent | ✅ Own content |
| `user` | ❌ No admin | 📋 Mode dependent | ✅ Own content |

## Security Testing

### Test Cross-Tenant Isolation
```sql
-- Should return 0
SELECT COUNT(*) FROM content WHERE tenant_id != current_tenant_id();
```

### Test User Isolation (in isolated mode)
```sql
-- Should return 0 for non-admins
SELECT COUNT(*) FROM content WHERE author_id != current_user_id();
```

### Test RLS Bypass Prevention
```sql
-- Should fail with permission error
ALTER TABLE content DISABLE ROW LEVEL SECURITY;
```

## Troubleshooting

### No Data Visible
1. Check context: `SELECT current_tenant_id(), current_user_id();`
2. Verify user exists: `SELECT * FROM users WHERE id = current_user_id();`
3. Check isolation mode: `SELECT get_tenant_isolation_mode();`

### Permission Denied
1. Check user role: `SELECT role FROM users WHERE id = current_user_id();`
2. Verify tenant membership: `SELECT tenant_id FROM users WHERE id = current_user_id();`
3. Check isolation settings: `SELECT * FROM tenant_isolation_settings;`

## Application Integration

### Required for Every Request
```rust
// Rust example
async fn set_rls_context(pool: &PgPool, tenant_id: Uuid, user_id: Uuid) -> Result<()> {
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(pool)
        .await?;
        
    sqlx::query("SELECT set_config('rls.user_id', $1, true)")
        .bind(user_id.to_string())
        .execute(pool)
        .await?;
        
    Ok(())
}
```

### Context Validation
```sql
-- Verify user belongs to tenant
SELECT EXISTS(
    SELECT 1 FROM users 
    WHERE id = $1 AND tenant_id = $2
) as valid_context;
```

## Security Checklist

- [ ] Context set for every database connection
- [ ] User authenticated before setting context
- [ ] Tenant membership verified
- [ ] Appropriate isolation mode configured
- [ ] Admin actions logged
- [ ] Cross-tenant access tests passing
- [ ] User isolation tests passing (if enabled)
- [ ] RLS bypass prevention tests passing
