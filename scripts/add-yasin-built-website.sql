-- Add Yasin's QuillSpace-built website to his connected websites
-- This website was built by the QuillSpace team using option 2 "Build and manage for me"

-- First, let's find Yasin's user record
DO $$
DECLARE
    yasin_user_id UUID;
    yasin_tenant_id UUID;
    credentials_id UUID;
    website_id UUID;
BEGIN
    -- Find Yasin's user record (update email as needed)
    SELECT id, tenant_id INTO yasin_user_id, yasin_tenant_id 
    FROM users 
    WHERE email ILIKE '%yasin%' OR name ILIKE '%yasin%'
    LIMIT 1;
    
    IF yasin_user_id IS NULL THEN
        RAISE NOTICE 'Yasin not found. Trying to find by tenant name...';
        
        -- Try to find by tenant
        SELECT u.id, u.tenant_id INTO yasin_user_id, yasin_tenant_id
        FROM users u
        JOIN tenants t ON u.tenant_id = t.id
        WHERE t.name = 'black-writers-ink'
        LIMIT 1;
    END IF;
    
    IF yasin_user_id IS NULL THEN
        RAISE NOTICE 'User not found. Please update the search criteria in this script.';
        RETURN;
    END IF;
    
    RAISE NOTICE 'Found user: % in tenant: %', yasin_user_id, yasin_tenant_id;
    
    -- Create credentials record for QuillSpace-managed website
    INSERT INTO website_builder_credentials (
        tenant_id, 
        user_id, 
        builder_type, 
        encrypted_credentials,
        is_active,
        last_sync,
        created_at,
        updated_at
    ) VALUES (
        yasin_tenant_id,
        yasin_user_id,
        'wix',
        'quillspace_managed_website', -- Special marker for QuillSpace-built sites
        TRUE,
        NOW(),
        NOW(),
        NOW()
    ) ON CONFLICT (user_id, builder_type) 
    DO UPDATE SET 
        updated_at = NOW(),
        is_active = TRUE
    RETURNING id INTO credentials_id;
    
    RAISE NOTICE 'Created/updated credentials: %', credentials_id;
    
    -- Insert the connected website
    INSERT INTO connected_websites (
        tenant_id,
        user_id, 
        credentials_id,
        builder_type,
        external_site_id,
        name,
        url,
        domain,
        status,
        last_sync,
        metadata,
        created_at,
        updated_at
    ) VALUES (
        yasin_tenant_id,
        yasin_user_id,
        credentials_id,
        'wix',
        '1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9', -- From the Wix dashboard URL
        'Yasin Kakande',
        'https://yasinkakande.wixsite.com/yasin-kakande',
        'yasinkakande.wixsite.com',
        'active',
        NOW(),
        jsonb_build_object(
            'wix_site_id', '1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9',
            'built_by', 'quillspace_team',
            'service_type', 'build_and_manage',
            'plan_type', 'free',
            'editable_by_client', true,
            'managed_by_quillspace', true,
            'last_published', NOW()
        ),
        NOW(),
        NOW()
    ) ON CONFLICT (user_id, builder_type, external_site_id) 
    DO UPDATE SET 
        name = EXCLUDED.name,
        url = EXCLUDED.url,
        domain = EXCLUDED.domain,
        status = 'active',
        last_sync = NOW(),
        metadata = EXCLUDED.metadata,
        updated_at = NOW()
    RETURNING id INTO website_id;
    
    RAISE NOTICE 'Successfully added Yasin''s QuillSpace-built website! Website ID: %', website_id;
    
END $$;

-- Verify the insertion
SELECT 
    cw.name,
    cw.url,
    cw.domain,
    cw.status,
    cw.builder_type,
    cw.metadata->>'built_by' as built_by,
    cw.metadata->>'service_type' as service_type,
    u.email as user_email,
    u.name as user_name,
    t.name as tenant_name
FROM connected_websites cw
JOIN users u ON cw.user_id = u.id
JOIN tenants t ON cw.tenant_id = t.id
WHERE cw.builder_type = 'wix'
AND (u.email ILIKE '%yasin%' OR u.name ILIKE '%yasin%' OR t.name = 'black-writers-ink')
ORDER BY cw.created_at DESC;
