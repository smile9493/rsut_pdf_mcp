# Development Dockerfile - 从源码构建
FROM rust:latest-bookworm as builder

WORKDIR /app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制源码
COPY pdf-module-rs/Cargo.toml Cargo.toml
COPY pdf-module-rs/Cargo.lock Cargo.lock
COPY pdf-module-rs/crates ./crates

# 构建项目
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 pdfuser

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/pdf-mcp /usr/local/bin/pdf-mcp
COPY --from=builder /app/target/release/pdf-rest /usr/local/bin/pdf-rest

# 设置权限
RUN chmod +x /usr/local/bin/pdf-mcp /usr/local/bin/pdf-rest

# 创建必要目录
RUN mkdir -p /app/data /app/logs/audit /app/cache && \
    chown -R pdfuser:pdfuser /app

# 切换到非 root 用户
USER pdfuser

# 环境变量
ENV RUST_LOG=info
ENV STORAGE_TYPE=local
ENV STORAGE_LOCAL_DIR=/app/data
ENV CACHE_ENABLED=true
ENV CACHE_MAX_SIZE=1000
ENV AUDIT_ENABLED=true
ENV AUDIT_LOG_DIR=/app/logs/audit
ENV MAX_FILE_SIZE_MB=100

# 暴露端口
EXPOSE 8000 8001

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/api/v1/x2text/health || exit 1

# 默认启动 REST API 服务
CMD ["pdf-rest"]
