#!/bin/bash
set -e

echo "🚀 Starting QuillSpace Backend..."

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
until pg_isready -h postgres -p 5432 -U quillspace; do
  echo "PostgreSQL is unavailable - sleeping"
  sleep 2
done

echo "✅ PostgreSQL is ready!"

# Run database migrations
echo "🔄 Running database migrations..."
export PGPASSWORD="$POSTGRES_PASSWORD"

# Run migrations in order
for migration_file in /app/migrations/*.sql; do
  if [ -f "$migration_file" ]; then
    echo "Running migration: $(basename "$migration_file")"
    psql -h postgres -U quillspace -d quillspace_dev -f "$migration_file" || {
      echo "⚠️  Migration $(basename "$migration_file") failed or already applied"
    }
  fi
done

echo "✅ Database migrations completed!"

# Start the application
echo "🚀 Starting QuillSpace Core..."
exec ./quillspace-core
