# PDF Module Rust

高性能 PDF 文本提取服务，支持 REST API 和 MCP (Model Context Protocol) 双接口。

## 特性

- **多引擎支持**: lopdf、pdf-extract、pdfium 三种提取引擎
- **智能路由**: 根据文档特征自动选择最优引擎
- **熔断降级**: 自动故障转移，保证服务可用性
- **双接口**: REST API + MCP (stdio/SSE)
- **高性能**: Rust 实现，内存占用低，速度快

## 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp

# 构建
cargo build --release
```

### REST API 服务

```bash
# 启动服务
./target/release/pdf-rest --host 0.0.0.0 --port 8000

# 健康检查
curl http://localhost:8000/api/v1/x2text/health

# 提取文本
curl -X POST http://localhost:8000/api/v1/x2text/extract \
  -F "file=@document.pdf"

# 指定引擎
curl -X POST http://localhost:8000/api/v1/x2text/extract \
  -F "file=@document.pdf" \
  -F "adapter=pdfium"

# 获取结构化数据
curl -X POST http://localhost:8000/api/v1/x2text/extract-json \
  -F "file=@document.pdf"

# 列出可用引擎
curl http://localhost:8000/api/v1/x2text/adapters
```

### MCP Server

```bash
# stdio 模式 (用于 Claude Desktop 等)
./target/release/pdf-mcp serve --transport stdio

# SSE 模式 (HTTP)
./target/release/pdf-mcp serve --transport sse --port 8001
```

**Claude Desktop 配置** (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "pdf": {
      "command": "/path/to/pdf-mcp",
      "args": ["serve", "--transport", "stdio"]
    }
  }
}
```

## API 端点

### REST API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/v1/x2text/health` | GET | 健康检查 |
| `/api/v1/x2text/extract` | POST | 提取纯文本 |
| `/api/v1/x2text/extract-json` | POST | 提取结构化 JSON |
| `/api/v1/x2text/info` | POST | 获取 PDF 信息 |
| `/api/v1/x2text/adapters` | GET | 列出可用引擎 |
| `/api/v1/x2text/cache/stats` | GET | 缓存统计 |

### MCP Tools

| 工具 | 说明 |
|------|------|
| `extract_text` | 提取纯文本 |
| `extract_structured` | 提取结构化数据 |
| `get_page_count` | 获取页数 |
| `search_keywords` | 关键词搜索 |
| `extract_keywords` | 自动提取高频词 |
| `list_adapters` | 列出可用引擎 |
| `cache_stats` | 缓存统计 |

## PDF 引擎

| 引擎 | 特点 | 适用场景 |
|------|------|----------|
| `lopdf` | 布局感知 | 通用文档 |
| `pdf-extract` | 最快 | 简单文档 |
| `pdfium` | 最兼容 | 复杂/特殊编码 |

**别名支持**:
- `pymupdf`, `fitz` → lopdf
- `pdfplumber` → pdf-extract

## 智能路由

系统根据文档特征自动选择最优引擎：

1. **小文档 + 简单布局** → `pdf-extract` (最快)
2. **特殊编码 (CIDFont/Type3)** → `pdfium` (最兼容)
3. **默认** → `lopdf` (布局感知)

## 熔断降级

- 连续 5 次失败后熔断
- 60 秒冷却后尝试恢复
- 自动降级到 `pdfium` 备用引擎

## Docker 部署

```bash
# 构建镜像
docker build -t pdf-module .

# 运行
docker run -p 8000:8000 pdf-module

# 或使用 docker-compose
docker-compose up -d
```

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `PDF_SERVER_HOST` | `127.0.0.1` | 监听地址 |
| `PDF_SERVER_PORT` | `8000` | 监听端口 |
| `PDF_DEFAULT_ADAPTER` | `lopdf` | 默认引擎 |
| `PDF_CACHE_ENABLED` | `true` | 启用缓存 |
| `PDF_CACHE_MAX_SIZE` | `1000` | 缓存最大条目 |
| `PDF_CACHE_TTL_SECONDS` | `3600` | 缓存 TTL |
| `PDF_MAX_FILE_SIZE_MB` | `200` | 最大文件大小 |

## 项目结构

```
pdf-module-rs/
├── Cargo.toml              # Workspace 配置
├── Dockerfile              # Docker 构建文件
├── docker-compose.yml      # Docker Compose
├── .env.example            # 环境变量示例
└── crates/
    ├── pdf-core/           # 核心库
    │   ├── src/
    │   │   ├── engine/     # PDF 引擎实现
    │   │   ├── cache.rs    # 缓存
    │   │   ├── config.rs   # 配置
    │   │   ├── dto.rs      # 数据模型
    │   │   ├── error.rs    # 错误处理
    │   │   ├── extractor.rs # 服务编排 + 智能路由 + 熔断
    │   │   ├── keyword.rs  # 关键词提取
    │   │   ├── metrics.rs  # Prometheus 指标
    │   │   └── validator.rs # 文件验证
    │   └── Cargo.toml
    ├── pdf-rest/           # REST API 服务
    │   ├── src/
    │   │   ├── main.rs     # 入口
    │   │   └── routes.rs   # 路由
    │   └── Cargo.toml
    ├── pdf-mcp/            # MCP Server
    │   ├── src/
    │   │   ├── main.rs     # 入口
    │   │   ├── server.rs   # stdio 传输
    │   │   └── sse.rs      # SSE 传输
    │   └── Cargo.toml
    └── pdf-python/         # PyO3 绑定 (可选)
        └── Cargo.toml
```

## 性能

| 指标 | 值 |
|------|-----|
| 二进制大小 (pdf-rest) | 12MB |
| 二进制大小 (pdf-mcp) | 16MB |
| 内存占用 | ~10-50MB (取决于文件) |
| 提取速度 | 10-50x vs Python |

## 许可证

MIT
