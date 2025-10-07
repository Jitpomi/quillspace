#!/bin/bash
set -e

echo "🚀 Starting QuillSpace backend..."

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
while ! pg_isready -h postgres -p 5432 -U quillspace; do
    echo "   PostgreSQL is not ready yet. Waiting..."
    sleep 2
done
echo "✅ PostgreSQL is ready!"

# Run database seeding
echo "🌱 Seeding database with demo data..."
if [ -f "/app/scripts/seed-dev.sql" ]; then
    # Clear existing demo data first
    echo "🧹 Clearing existing demo data..."
    PGPASSWORD=dev_password psql -h postgres -U quillspace -d quillspace_dev -c "
        DELETE FROM content WHERE tenant_id IN (
            SELECT id FROM tenants WHERE slug IN ('demo', 'demo-publishing')
        );
        DELETE FROM users WHERE tenant_id IN (
            SELECT id FROM tenants WHERE slug IN ('demo', 'demo-publishing')
        );
        DELETE FROM tenants WHERE slug IN ('demo', 'demo-publishing');
    "
    
    # Run seeding script
    PGPASSWORD=dev_password psql -h postgres -U quillspace -d quillspace_dev -f /app/scripts/seed-dev.sql
    echo "✅ Database seeded successfully!"
else
    echo "⚠️  Seed file not found, skipping seeding"
fi

# Start the application
echo "🎯 Starting QuillSpace application..."
exec ./quillspace-core
