#!/bin/bash

# Simple script to add a client's Wix site to QuillSpace
# Usage: ./add-client-wix-site.sh <user_email> <wix_site_id> [display_name]

set -e

# Check arguments
if [ $# -lt 2 ]; then
    echo "Usage: $0 <user_email> <wix_site_id> [display_name]"
    echo ""
    echo "Examples:"
    echo "  $0 yasin@example.com 1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9"
    echo "  $0 yasin@example.com 1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9 'Yasin Kakande Website'"
    echo ""
    echo "The Wix site ID can be found in the Wix dashboard URL:"
    echo "  https://manage.wix.com/dashboard/1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9/..."
    echo "                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^"
    exit 1
fi

USER_EMAIL="$1"
WIX_SITE_ID="$2"
DISPLAY_NAME="${3:-}"

# Database connection (use your actual connection string)
DB_URL="${DATABASE_URL:-postgresql://quillspace:dev_password@localhost:5432/quillspace_dev}"

echo "🔍 Adding Wix site for user: $USER_EMAIL"
echo "📍 Wix Site ID: $WIX_SITE_ID"
echo "📝 Display Name: ${DISPLAY_NAME:-'(will use Wix site name)'}"
echo ""

# SQL to add the mapping
SQL="
DO \$\$
DECLARE
    target_user_id UUID;
    target_tenant_id UUID;
BEGIN
    -- Find user by email
    SELECT id, tenant_id INTO target_user_id, target_tenant_id 
    FROM users 
    WHERE email ILIKE '%$USER_EMAIL%'
    LIMIT 1;
    
    IF target_user_id IS NULL THEN
        RAISE EXCEPTION 'User not found with email: $USER_EMAIL';
    END IF;
    
    RAISE NOTICE 'Found user: % in tenant: %', target_user_id, target_tenant_id;
    
    -- Insert the mapping
    INSERT INTO user_wix_sites (
        wix_site_id,
        tenant_id,
        user_id,
        display_name,
        project_status,
        service_type,
        client_can_edit,
        client_can_publish,
        metadata
    ) VALUES (
        '$WIX_SITE_ID',
        target_tenant_id,
        target_user_id,
        NULLIF('$DISPLAY_NAME', ''),  -- NULL if empty string
        'active',
        'build_and_manage',
        TRUE,
        TRUE,
        jsonb_build_object(
            'created_by', 'script',
            'created_via', 'add-client-wix-site.sh',
            'service_start_date', NOW()
        )
    ) ON CONFLICT (wix_site_id) 
    DO UPDATE SET 
        user_id = EXCLUDED.user_id,
        tenant_id = EXCLUDED.tenant_id,
        display_name = COALESCE(EXCLUDED.display_name, user_wix_sites.display_name),
        updated_at = NOW();
    
    RAISE NOTICE '✅ Successfully mapped Wix site % to user %', '$WIX_SITE_ID', target_user_id;
    
END \$\$;

-- Verify the mapping
SELECT 
    '✅ Mapping created:' as status,
    uws.wix_site_id,
    uws.display_name,
    u.name as user_name,
    u.email as user_email,
    t.name as tenant_name
FROM user_wix_sites uws
JOIN users u ON uws.user_id = u.id
JOIN tenants t ON uws.tenant_id = t.id
WHERE uws.wix_site_id = '$WIX_SITE_ID';
"

# Execute the SQL
echo "🚀 Executing database update..."
echo "$SQL" | psql "$DB_URL"

echo ""
echo "🎉 Done! The user should now see their Wix website in QuillSpace."
echo ""
echo "Next steps:"
echo "1. User logs into QuillSpace"
echo "2. Goes to Websites page"
echo "3. Should see their Wix site in 'Connected Websites'"
echo "4. Can click 'Edit' to open QuillSpace editor"
echo ""
echo "To remove this mapping later:"
echo "  DELETE FROM user_wix_sites WHERE wix_site_id = '$WIX_SITE_ID';"
