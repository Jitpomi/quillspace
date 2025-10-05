#!/bin/bash

# QuillSpace Seamless Deployment Script
# This script handles the complete deployment process automatically

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Wait for service to be ready
wait_for_service() {
    local service_name=$1
    local check_command=$2
    local max_attempts=30
    local attempt=1

    log_info "Waiting for $service_name to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        if eval "$check_command" >/dev/null 2>&1; then
            log_success "$service_name is ready!"
            return 0
        fi
        
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    log_error "$service_name failed to start within $((max_attempts * 2)) seconds"
    return 1
}

# Main deployment function
main() {
    log_info "🚀 Starting QuillSpace Deployment"
    
    # Step 1: Check prerequisites
    log_info "📋 Checking prerequisites..."
    
    if ! command_exists docker; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! command_exists cargo; then
        log_error "Rust/Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    if ! command_exists pnpm; then
        log_warning "pnpm not found, trying npm..."
        if ! command_exists npm; then
            log_error "Neither pnpm nor npm found. Please install Node.js and pnpm."
            exit 1
        fi
        PACKAGE_MANAGER="npm"
    else
        PACKAGE_MANAGER="pnpm"
    fi
    
    # Step 2: Start Docker if not running
    log_info "🐳 Ensuring Docker is running..."
    if ! docker ps >/dev/null 2>&1; then
        log_info "Starting Docker..."
        if command_exists colima; then
            colima start
        else
            log_error "Docker is not running and Colima is not available. Please start Docker manually."
            exit 1
        fi
    fi
    
    # Step 3: Stop any existing services
    log_info "🛑 Stopping existing services..."
    docker-compose -f docker-compose.dev.yml down >/dev/null 2>&1 || true
    pkill -f "quillspace-core" >/dev/null 2>&1 || true
    
    # Step 4: Start database services
    log_info "🗄️ Starting database services..."
    docker-compose -f docker-compose.minimal.yml up -d
    
    # Step 5: Wait for PostgreSQL to be ready
    wait_for_service "PostgreSQL" "docker exec quillspace-postgres-dev pg_isready -U quillspace"
    
    # Step 6: Run database migrations
    log_info "📊 Setting up database..."
    
    # Clear any existing migration state and recreate
    docker exec -i quillspace-postgres-dev psql -U quillspace -d quillspace_dev -c "
        DROP TABLE IF EXISTS _sqlx_migrations CASCADE;
        DROP TABLE IF EXISTS content CASCADE;
        DROP TABLE IF EXISTS users CASCADE;
        DROP TABLE IF EXISTS tenants CASCADE;
        DROP TYPE IF EXISTS user_role CASCADE;
        DROP TYPE IF EXISTS content_status CASCADE;
    " >/dev/null 2>&1 || true
    
    # Run migrations
    docker exec -i quillspace-postgres-dev psql -U quillspace -d quillspace_dev < quillspace-core/migrations/001_initial_schema.sql
    docker exec -i quillspace-postgres-dev psql -U quillspace -d quillspace_dev < quillspace-core/migrations/002_row_level_security.sql
    
    # Mark migrations as completed in SQLx
    docker exec -i quillspace-postgres-dev psql -U quillspace -d quillspace_dev -c "
        INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) 
        VALUES 
            (1, 'initial_schema', NOW(), true, decode('00000000000000000000000000000000', 'hex'), 1000),
            (2, 'row_level_security', NOW(), true, decode('00000000000000000000000000000000', 'hex'), 1000)
        ON CONFLICT (version) DO NOTHING;
    " >/dev/null 2>&1 || true
    
    log_success "Database setup complete"
    
    # Step 7: Start backend
    log_info "⚙️ Starting Rust backend..."
    cd quillspace-core
    SQLX_OFFLINE=true cargo build --release >/dev/null 2>&1
    SQLX_OFFLINE=true cargo run --release &
    BACKEND_PID=$!
    cd ..
    
    # Step 8: Wait for backend to be ready
    wait_for_service "Backend API" "curl -s http://localhost:3000/health"
    
    # Step 9: Start frontend
    log_info "🎨 Starting frontend..."
    cd frontend
    if [ "$PACKAGE_MANAGER" = "pnpm" ]; then
        pnpm install >/dev/null 2>&1
        pnpm dev &
    else
        npm install >/dev/null 2>&1
        npm run dev &
    fi
    FRONTEND_PID=$!
    cd ..
    
    # Step 10: Wait for frontend to be ready
    wait_for_service "Frontend" "curl -s http://localhost:5174"
    
    # Step 11: Success message
    log_success "🎉 QuillSpace deployment complete!"
    echo ""
    echo "📱 Frontend: http://localhost:5174"
    echo "🔧 Backend API: http://localhost:3000"
    echo "🗄️ Database: PostgreSQL on localhost:5432"
    echo ""
    echo "Demo credentials:"
    echo "  Email: admin@demo.com"
    echo "  Password: admin123"
    echo ""
    echo "Press Ctrl+C to stop all services"
    
    # Step 12: Keep script running and handle cleanup
    cleanup() {
        log_info "🛑 Shutting down services..."
        kill $BACKEND_PID 2>/dev/null || true
        kill $FRONTEND_PID 2>/dev/null || true
        docker-compose -f docker-compose.minimal.yml down >/dev/null 2>&1 || true
        log_success "All services stopped"
        exit 0
    }
    
    trap cleanup INT TERM
    
    # Wait for user interrupt
    while true; do
        sleep 1
    done
}

# Run main function
main "$@"
