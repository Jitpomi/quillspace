# Multi-stage build for Rust backend using stable Rust
FROM rust:1.82-slim-bookworm as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files from quillspace-core directory
COPY quillspace-core/Cargo.toml ./
COPY quillspace-core/Cargo.loc[k] ./

# Copy source code
COPY quillspace-core/src ./src
COPY quillspace-core/migrations ./migrations

# Build the application (no SQLx offline mode needed)
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false quillspace

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/quillspace-core /app/quillspace-core

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Copy config files
COPY config /app/config

# Change ownership
RUN chown -R quillspace:quillspace /app

# Switch to app user
USER quillspace

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["./quillspace-core"]
