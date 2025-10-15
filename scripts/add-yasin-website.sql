-- Script to add Yasin Kakande's existing Wix website to QuillSpace
-- Run this after the main migration to connect his existing site

-- First, let's find Yasin's user ID (replace email with actual email)
-- You'll need to update this with the correct email address
DO $$
DECLARE
    yasin_user_id UUID;
    yasin_tenant_id UUID;
    credentials_id UUID;
BEGIN
    -- Find Yasin's user record (update email as needed)
    SELECT id, tenant_id INTO yasin_user_id, yasin_tenant_id 
    FROM users 
    WHERE email = 'yasin@blackwritersink.com' -- Update this email
    LIMIT 1;
    
    IF yasin_user_id IS NULL THEN
        RAISE NOTICE 'User not found. Please update the email address in this script.';
        RETURN;
    END IF;
    
    RAISE NOTICE 'Found user: % in tenant: %', yasin_user_id, yasin_tenant_id;
    
    -- Create credentials record for manual connection
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
        'manual_connection_yasin_wix', -- Placeholder for manual connection
        TRUE,
        NOW(),
        NOW(),
        NOW()
    ) ON CONFLICT (user_id, builder_type) 
    DO UPDATE SET updated_at = NOW()
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
        '1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9', -- From the Wix URL in screenshot
        'Yasin Kakande',
        'https://yasinkakande.wixsite.com/yasin-kakande',
        'yasinkakande.wixsite.com',
        'active',
        NOW(),
        jsonb_build_object(
            'wix_site_id', '1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9',
            'plan_type', 'free',
            'template_id', 'unknown',
            'last_published', NOW(),
            'manually_added', true
        ),
        NOW(),
        NOW()
    ) ON CONFLICT (user_id, builder_type, external_site_id) 
    DO UPDATE SET 
        name = EXCLUDED.name,
        url = EXCLUDED.url,
        domain = EXCLUDED.domain,
        status = EXCLUDED.status,
        last_sync = EXCLUDED.last_sync,
        metadata = EXCLUDED.metadata,
        updated_at = NOW();
    
    RAISE NOTICE 'Successfully added Yasin''s Wix website!';
    
END $$;

-- Verify the insertion
SELECT 
    cw.name,
    cw.url,
    cw.domain,
    cw.status,
    cw.builder_type,
    u.email as user_email,
    t.name as tenant_name
FROM connected_websites cw
JOIN users u ON cw.user_id = u.id
JOIN tenants t ON cw.tenant_id = t.id
WHERE cw.builder_type = 'wix'
AND u.email LIKE '%yasin%' -- Adjust this filter as needed
ORDER BY cw.created_at DESC;
