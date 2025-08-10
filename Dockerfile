# Terra Siaga - Dockerfile for Production Deployment
# Multi-stage build for optimized container size

# Build stage
FROM rust:1.70-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r terrasiaga && useradd -r -g terrasiaga terrasiaga

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/terra-siaga /app/terra-siaga

# Copy configuration files
COPY config/ ./config/

# Create directories for logs and data
RUN mkdir -p /app/logs /app/data && \
    chown -R terrasiaga:terrasiaga /app

# Switch to non-root user
USER terrasiaga

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV TERRA_ENV=production

# Run the application
CMD ["./terra-siaga"]
