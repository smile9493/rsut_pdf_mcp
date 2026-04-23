# Simplified Dockerfile for PDF Module MCP Server
FROM rust:1.83-slim as builder

# Set working directory
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY pdf-module-rs/Cargo.toml pdf-module-rs/ ./
COPY pdf-module-rs/Cargo.lock pdf-module-rs/ ./

# Copy source code
COPY pdf-module-rs/crates ./crates

# Update Cargo.lock to match current Rust version
RUN cargo update

# Build the project (may take several minutes)
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 pdfuser

# Set working directory
WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/pdf-mcp /usr/local/bin/
COPY --from=builder /app/target/release/pdf-rest /usr/local/bin/

# Create necessary directories
RUN mkdir -p /app/data /app/logs/audit /app/cache && \
    chown -R pdfuser:pdfuser /app

# Switch to non-root user
USER pdfuser

# Set environment variables
ENV RUST_LOG=info
ENV STORAGE_TYPE=local
ENV STORAGE_LOCAL_DIR=/app/data
ENV CACHE_ENABLED=true
ENV CACHE_MAX_SIZE_MB=100
ENV AUDIT_ENABLED=true
ENV AUDIT_LOG_DIR=/app/logs/audit
ENV AUDIT_RETENTION_DAYS=30
ENV LOG_LEVEL=info
ENV LOG_FORMAT=text
ENV ENABLE_CORS=true
ENV ALLOWED_ORIGINS=*
ENV MAX_FILE_SIZE_MB=100
ENV ALLOWED_EXTENSIONS=pdf

# Expose ports
EXPOSE 8000 8001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/api/v1/x2text/health || exit 1

# Default command
CMD ["pdf-mcp", "serve", "--transport", "stdio"]