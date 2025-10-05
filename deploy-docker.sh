#!/bin/bash

# QuillSpace Docker Deployment Script
# Complete containerized deployment with Docker Compose

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

# Main deployment function
main() {
    log_info "🐳 Starting QuillSpace Docker Deployment"
    
    # Step 1: Check prerequisites
    log_info "📋 Checking prerequisites..."
    
    if ! command_exists docker; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! command_exists docker-compose && ! docker compose version >/dev/null 2>&1; then
        log_error "Docker Compose is not available. Please install Docker Compose."
        exit 1
    fi
    
    # Determine docker-compose command
    if command_exists docker-compose; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD="docker compose"
    fi
    
    # Step 2: Check Docker is running
    log_info "🐳 Ensuring Docker is running..."
    if ! docker ps >/dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker first."
        exit 1
    fi
    
    # Step 3: Load environment variables
    if [ -f .env.docker ]; then
        log_info "📄 Loading environment configuration..."
        export $(cat .env.docker | grep -v '^#' | xargs)
    else
        log_warning "No .env.docker file found, using defaults"
    fi
    
    # Step 4: Stop any existing deployment
    log_info "🛑 Stopping existing deployment..."
    $COMPOSE_CMD down --remove-orphans >/dev/null 2>&1 || true
    
    # Step 5: Code Quality Checks
    log_info "🔍 Running code quality checks..."
    
    # Frontend linting
    log_info "Checking frontend code quality..."
    cd frontend
    if ! pnpm run lint; then
        log_error "Frontend linting failed! Please fix errors before deploying."
        exit 1
    fi
    log_success "Frontend linting passed!"
    cd ..
    
    # Backend checks
    log_info "Checking backend code quality..."
    cd quillspace-core
    if ! cargo check; then
        log_error "Backend compilation check failed!"
        exit 1
    fi
    log_success "Backend checks passed!"
    cd ..
    
    # Step 6: Build and start services
    log_info "🏗️ Building and starting services..."
    log_info "This may take a few minutes on first run..."
    
    # Build images
    log_info "Building backend image..."
    $COMPOSE_CMD build backend
    
    log_info "Building frontend image..."
    $COMPOSE_CMD build frontend
    
    # Start services
    log_info "Starting all services..."
    $COMPOSE_CMD up -d
    
    # Step 6: Wait for services to be healthy
    log_info "⏳ Waiting for services to be ready..."
    
    # Wait for PostgreSQL
    log_info "Waiting for PostgreSQL..."
    timeout=60
    while [ $timeout -gt 0 ]; do
        if $COMPOSE_CMD exec -T postgres pg_isready -U quillspace -d quillspace_dev >/dev/null 2>&1; then
            log_success "PostgreSQL is ready!"
            break
        fi
        echo -n "."
        sleep 2
        timeout=$((timeout - 2))
    done
    
    if [ $timeout -le 0 ]; then
        log_error "PostgreSQL failed to start"
        exit 1
    fi
    
    # Wait for ClickHouse
    log_info "Waiting for ClickHouse..."
    timeout=60
    while [ $timeout -gt 0 ]; do
        if curl -s http://localhost:8123/ping >/dev/null 2>&1; then
            log_success "ClickHouse is ready!"
            break
        fi
        echo -n "."
        sleep 2
        timeout=$((timeout - 2))
    done
    
    # Wait for Backend
    log_info "Waiting for Backend API..."
    timeout=120
    while [ $timeout -gt 0 ]; do
        if curl -s http://localhost:3000/health >/dev/null 2>&1; then
            log_success "Backend API is ready!"
            break
        fi
        echo -n "."
        sleep 2
        timeout=$((timeout - 2))
    done
    
    if [ $timeout -le 0 ]; then
        log_error "Backend API failed to start"
        log_info "Checking backend logs..."
        $COMPOSE_CMD logs backend
        exit 1
    fi
    
    # Wait for Frontend
    log_info "Waiting for Frontend..."
    timeout=60
    while [ $timeout -gt 0 ]; do
        if curl -s http://localhost/health >/dev/null 2>&1; then
            log_success "Frontend is ready!"
            break
        fi
        echo -n "."
        sleep 2
        timeout=$((timeout - 2))
    done
    
    # Step 7: Success message
    log_success "🎉 QuillSpace Docker deployment complete!"
    echo ""
    echo "🌐 Application URL: http://localhost"
    echo "🔧 Backend API: http://localhost:3000"
    echo "🗄️ PostgreSQL: localhost:5432"
    echo "📊 ClickHouse: http://localhost:8123"
    echo ""
    echo "Demo credentials:"
    echo "  Email: admin@demo.com"
    echo "  Password: admin123"
    echo ""
    echo "📋 Useful commands:"
    echo "  View logs: $COMPOSE_CMD logs -f"
    echo "  Stop services: $COMPOSE_CMD down"
    echo "  Restart: $COMPOSE_CMD restart"
    echo "  View status: $COMPOSE_CMD ps"
    echo ""
    log_success "All services are running in Docker containers!"
}

# Handle cleanup on script exit
cleanup() {
    log_info "To stop all services, run: $COMPOSE_CMD down"
}

trap cleanup EXIT

# Run main function
main "$@"
