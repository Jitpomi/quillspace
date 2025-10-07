-- QuillSpace Development Database Seeding
-- This script populates the development database with sample data for testing

-- Insert demo tenant (matching actual schema)
INSERT INTO tenants (id, name, slug, settings)
VALUES (
    '550e8400-e29b-41d4-a716-446655440000',
    'Demo Publishing House',
    'demo-publishing',
    '{"theme": "default", "features": {"analytics": true, "comments": true, "newsletter": true}}'
) ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    settings = EXCLUDED.settings,
    updated_at = NOW();

-- Insert demo users (matching actual schema)
INSERT INTO users (id, tenant_id, email, name, password_hash, role)
VALUES 
    (
        '550e8400-e29b-41d4-a716-446655440001',
        '550e8400-e29b-41d4-a716-446655440000',
        'admin@demo.com',
        'Admin User',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VJBzxqrxe', -- password: admin123
        'admin'
    ),
    (
        '550e8400-e29b-41d4-a716-446655440002',
        '550e8400-e29b-41d4-a716-446655440000',
        'yasin@demo.com',
        'Yasin',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VJBzxqrxe', -- password: admin123
        'editor'
    ),
    (
        '550e8400-e29b-41d4-a716-446655440003',
        '550e8400-e29b-41d4-a716-446655440000',
        'joe@demo.com',
        'Joe',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VJBzxqrxe', -- password: admin123
        'viewer'
    )
ON CONFLICT (tenant_id, email) DO UPDATE SET
    name = EXCLUDED.name,
    password_hash = EXCLUDED.password_hash,
    role = EXCLUDED.role,
    updated_at = NOW();

-- Insert demo content (matching actual schema)
INSERT INTO content (id, tenant_id, title, slug, body, status, author_id, published_at)
VALUES 
    (
        '550e8400-e29b-41d4-a716-446655440020',
        '550e8400-e29b-41d4-a716-446655440000',
        'Welcome to QuillSpace: Your Digital Writing Sanctuary',
        'welcome-to-quillspace',
        'Welcome to QuillSpace! This is your personal sanctuary for creativity, connection, and growth as a writer. QuillSpace provides you with a clean, distraction-free environment where your words can flourish. Connect with readers who truly appreciate your work and build lasting relationships with your audience.',
        'published',
        '550e8400-e29b-41d4-a716-446655440002',
        NOW() - INTERVAL '2 days'
    ),
    (
        '550e8400-e29b-41d4-a716-446655440021',
        '550e8400-e29b-41d4-a716-446655440000',
        'The Art of Storytelling in the Digital Age',
        'art-of-storytelling-digital-age',
        'In our hyperconnected world, the fundamentals of good storytelling remain unchanged, but the mediums and methods continue to evolve. Great stories start with compelling characters and emotional resonance. The best stories make us feel something and touch our hearts.',
        'published',
        '550e8400-e29b-41d4-a716-446655440002',
        NOW() - INTERVAL '5 days'
    ),
    (
        '550e8400-e29b-41d4-a716-446655440022',
        '550e8400-e29b-41d4-a716-446655440000',
        'Building Your Author Platform: A Complete Guide',
        'building-author-platform-complete-guide',
        'An author platform is more than just a website—it''s your entire online presence and the foundation of your writing career. Your platform consists of your website, social media presence, email newsletter, and professional networks.',
        'published',
        '550e8400-e29b-41d4-a716-446655440002',
        NOW() - INTERVAL '1 week'
    ),
    (
        '550e8400-e29b-41d4-a716-446655440023',
        '550e8400-e29b-41d4-a716-446655440000',
        'Draft: Overcoming Writer''s Block',
        'overcoming-writers-block',
        'Writer''s block is one of the most frustrating experiences for any writer. Here are some strategies that have helped many writers push through those difficult periods. Sometimes a simple change of scenery can unlock creativity.',
        'draft',
        '550e8400-e29b-41d4-a716-446655440002',
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
    content_count INTEGER;
    published_count INTEGER;
    draft_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO tenant_count FROM tenants;
    SELECT COUNT(*) INTO user_count FROM users WHERE tenant_id = '550e8400-e29b-41d4-a716-446655440000';
    SELECT COUNT(*) INTO content_count FROM content WHERE tenant_id = '550e8400-e29b-41d4-a716-446655440000';
    SELECT COUNT(*) INTO published_count FROM content WHERE tenant_id = '550e8400-e29b-41d4-a716-446655440000' AND status = 'published';
    SELECT COUNT(*) INTO draft_count FROM content WHERE tenant_id = '550e8400-e29b-41d4-a716-446655440000' AND status = 'draft';
    
    RAISE NOTICE '🌱 QuillSpace development database seeded successfully!';
    RAISE NOTICE '';
    RAISE NOTICE '📊 Seeding Summary:';
    RAISE NOTICE '   • Tenants: %', tenant_count;
    RAISE NOTICE '   • Users: %', user_count;
    RAISE NOTICE '   • Total Content: %', content_count;
    RAISE NOTICE '   • Published Articles: %', published_count;
    RAISE NOTICE '   • Draft Articles: %', draft_count;
    RAISE NOTICE '';
    RAISE NOTICE '🔑 Demo Credentials:';
    RAISE NOTICE '   • Admin: admin@demo.com / admin123';
    RAISE NOTICE '   • Yasin: yasin@demo.com / admin123';
    RAISE NOTICE '   • Joe: joe@demo.com / admin123';
    RAISE NOTICE '';
    RAISE NOTICE '🚀 Ready to test QuillSpace!';
END
$$;
