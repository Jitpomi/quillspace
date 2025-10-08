-- QuillSpace Database Seeding
-- Real user data for Yasin Kakande and Josephine Nakimuli

-- Insert Yasin's tenant (Yasin Kakande's publishing space)
INSERT INTO tenants (id, name, slug, settings)
VALUES (
    '11111111-1111-1111-1111-111111111111',
    'Yasin Kakande Publishing',
    'yasin-kakande',
    '{"theme": "professional", "features": {"analytics": true, "comments": true, "newsletter": true}}'
) ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    settings = EXCLUDED.settings,
    updated_at = NOW();

-- Insert Joey's tenant (Josephine Nakimuli's publishing space)
INSERT INTO tenants (id, name, slug, settings)
VALUES (
    '22222222-2222-2222-2222-222222222222',
    'Josephine Nakimuli Publishing',
    'josephine-nakimuli',
    '{"theme": "creative", "features": {"analytics": true, "comments": true, "newsletter": true}}'
) ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    settings = EXCLUDED.settings,
    updated_at = NOW();

-- Insert system tenant for admin
INSERT INTO tenants (id, name, slug, settings)
VALUES (
    '00000000-0000-0000-0000-000000000000',
    'QuillSpace System',
    'system-admin',
    '{"theme": "admin", "features": {"analytics": true, "comments": true, "newsletter": true, "system_admin": true}}'
) ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    settings = EXCLUDED.settings,
    updated_at = NOW();

-- Insert users with proper bcrypt hashes (password: "secret")
INSERT INTO users (id, tenant_id, email, name, password_hash, role)
VALUES 
    -- System Admin (can access all tenants)
    (
        'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
        '00000000-0000-0000-0000-000000000000',
        'admin@quillspace.com',
        'System Administrator',
        '$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW', -- password: secret
        'admin'
    ),
    -- Yasin Kakande (tenant admin)
    (
        'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
        '11111111-1111-1111-1111-111111111111',
        'yasinkak@gmail.com',
        'Yasin Kakande',
        '$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW', -- password: secret
        'admin'
    ),
    -- Josephine Nakimuli (tenant admin)
    (
        'cccccccc-cccc-cccc-cccc-cccccccccccc',
        '22222222-2222-2222-2222-222222222222',
        'joeykigozi@yahoo.co.uk',
        'Josephine Nakimuli',
        '$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW', -- password: secret
        'admin'
    )
ON CONFLICT (tenant_id, email) DO UPDATE SET
    name = EXCLUDED.name,
    password_hash = EXCLUDED.password_hash,
    role = EXCLUDED.role,
    updated_at = NOW();

-- Insert content for Yasin Kakande
INSERT INTO content (id, tenant_id, title, slug, body, status, author_id, published_at)
VALUES 
    (
        'aaaaa001-bbbb-cccc-dddd-eeeeeeeeeeee',
        '11111111-1111-1111-1111-111111111111',
        'Welcome to My Digital Space',
        'welcome-yasin-kakande',
        'Hello! I''m Yasin Kakande, and this is my digital publishing space. Here I share my thoughts, insights, and creative works. Welcome to my corner of the internet where ideas come to life.',
        'published',
        'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
        NOW() - INTERVAL '1 day'
    ),
    (
        'aaaaa002-bbbb-cccc-dddd-eeeeeeeeeeee',
        '11111111-1111-1111-1111-111111111111',
        'Thoughts on Digital Publishing',
        'digital-publishing-thoughts',
        'The landscape of digital publishing continues to evolve. As creators, we must adapt to new platforms while maintaining our authentic voice. This platform represents a step towards independent publishing.',
        'published',
        'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
        NOW() - INTERVAL '3 days'
    ),
    -- Insert content for Josephine Nakimuli
    (
        'bbbbb001-cccc-dddd-eeee-ffffffffffff',
        '22222222-2222-2222-2222-222222222222',
        'Creative Expression in the Digital Age',
        'creative-expression-digital',
        'As Josephine Nakimuli, I believe creativity knows no bounds. This digital space allows me to explore various forms of expression, from written word to multimedia storytelling.',
        'published',
        'cccccccc-cccc-cccc-cccc-cccccccccccc',
        NOW() - INTERVAL '2 days'
    ),
    (
        'bbbbb002-cccc-dddd-eeee-ffffffffffff',
        '22222222-2222-2222-2222-222222222222',
        'Building Authentic Connections',
        'authentic-connections',
        'In our interconnected world, authentic connections matter more than ever. Through this platform, I hope to build meaningful relationships with readers who resonate with my work.',
        'published',
        'cccccccc-cccc-cccc-cccc-cccccccccccc',
        NOW() - INTERVAL '4 days'
    ),
    (
        'bbbbb003-cccc-dddd-eeee-ffffffffffff',
        '22222222-2222-2222-2222-222222222222',
        'Work in Progress: Future Vision',
        'future-vision-draft',
        'This is a draft exploring my vision for the future of digital creativity and community building. Still developing these ideas...',
        'draft',
        'cccccccc-cccc-cccc-cccc-cccccccccccc',
        NULL
    )
ON CONFLICT (tenant_id, slug) DO UPDATE SET
    title = EXCLUDED.title,
    body = EXCLUDED.body,
    status = EXCLUDED.status,
    author_id = EXCLUDED.author_id,
    published_at = EXCLUDED.published_at,
    updated_at = NOW();

-- Display seeding summary
DO $$
DECLARE
    tenant_count INTEGER;
    user_count INTEGER;
    yasin_content INTEGER;
    joey_content INTEGER;
    total_content INTEGER;
    published_count INTEGER;
    draft_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO tenant_count FROM tenants;
    SELECT COUNT(*) INTO user_count FROM users;
    SELECT COUNT(*) INTO yasin_content FROM content WHERE tenant_id = '11111111-1111-1111-1111-111111111111';
    SELECT COUNT(*) INTO joey_content FROM content WHERE tenant_id = '22222222-2222-2222-2222-222222222222';
    SELECT COUNT(*) INTO total_content FROM content;
    SELECT COUNT(*) INTO published_count FROM content WHERE status = 'published';
    SELECT COUNT(*) INTO draft_count FROM content WHERE status = 'draft';
    
    RAISE NOTICE '🌱 QuillSpace database seeded successfully!';
    RAISE NOTICE '';
    RAISE NOTICE '📊 Seeding Summary:';
    RAISE NOTICE '   • Tenants: %', tenant_count;
    RAISE NOTICE '   • Users: %', user_count;
    RAISE NOTICE '   • Total Content: %', total_content;
    RAISE NOTICE '   • Yasin''s Content: %', yasin_content;
    RAISE NOTICE '   • Joey''s Content: %', joey_content;
    RAISE NOTICE '   • Published Articles: %', published_count;
    RAISE NOTICE '   • Draft Articles: %', draft_count;
    RAISE NOTICE '';
    RAISE NOTICE '🔑 User Credentials (password: secret):';
    RAISE NOTICE '   • System Admin: admin@quillspace.com';
    RAISE NOTICE '   • Yasin Kakande: yasinkak@gmail.com';
    RAISE NOTICE '   • Josephine Nakimuli: joeykigozi@yahoo.co.uk';
    RAISE NOTICE '';
    RAISE NOTICE '🏢 Tenants:';
    RAISE NOTICE '   • System Admin: system-admin';
    RAISE NOTICE '   • Yasin Kakande: yasin-kakande';
    RAISE NOTICE '   • Josephine Nakimuli: josephine-nakimuli';
    RAISE NOTICE '';
    RAISE NOTICE '🚀 Ready for production use!';
END
$$;
