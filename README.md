# PDF Module MCP Server

高性能 PDF 文本提取 MCP (Model Context Protocol) 服务器，支持多种提取引擎、智能路由、缓存优化，可直接集成到 Cursor、Claude Desktop 等 AI Agent。

---

## 目录

- [特性](#特性)
- [快速开始](#快速开始)
- [Docker 部署](#docker-部署)
- [1Panel 编排](#1panel-编排)
- [Agent 集成](#agent-集成)
- [Python SDK](#python-sdk)
- [MCP 工具参考](#mcp-工具参考)
- [REST API 参考](#rest-api-参考)
- [提取引擎](#提取引擎)
- [配置参考](#配置参考)
- [性能优化](#性能优化)
- [安全机制](#安全机制)
- [架构设计](#架构设计)
- [开发指南](#开发指南)

---

## 特性

| 类别 | 特性 |
|------|------|
| **协议** | MCP stdio/SSE 双传输、REST API |
| **引擎** | lopdf (布局感知)、pdf-extract (快速)、pdfium (高兼容) |
| **智能路由** | 根据文档特征自动选择最优引擎 |
| **容错** | 熔断器 (Circuit Breaker)、自动回退 |
| **缓存** | Moka 并发缓存、正则缓存、Jieba 缓存、智能缓存键 |
| **安全** | 路径遍历防护、文件类型深度检测、大小限制 |
| **可观测** | Prometheus 指标、JSON 审计日志、结构化日志 |
| **SDK** | Python SDK (同步/异步/REST) |
| **部署** | Docker、1Panel、docker-compose |

---

## 快速开始

### 环境要求

- Rust 1.83+ (构建)
- Docker (部署)

### 本地构建

```bash
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp/pdf-module-rs

# 构建
cargo build --release

# 运行 MCP 服务 (stdio 模式)
cargo run --release --bin pdf-mcp -- serve --transport stdio

# 运行 MCP 服务 (SSE 模式)
cargo run --release --bin pdf-mcp -- serve --transport sse --port 8001

# 运行 REST 服务
cargo run --release --bin pdf-rest -- --host 0.0.0.0 --port 8000
```

### 测试

```bash
cargo test                    # 全部测试
cargo test -p pdf-core        # 核心模块
cargo test -p pdf-core keyword # 关键词模块
cargo test -p pdf-core cache   # 缓存模块
```

---

## Docker 部署

### 镜像托管

镜像托管在 Docker Hub：`smile9493/pdf-mcp:latest`

```bash
# 拉取镜像
docker pull smile9493/pdf-mcp:latest

# 运行 MCP 服务
docker run -d --name pdf-mcp-server \
  -p 8001:8001 \
  -v pdf_data:/app/data \
  -v pdf_logs:/app/logs/audit \
  smile9493/pdf-mcp:latest \
  pdf-mcp serve --transport sse --port 8001

# 运行 REST 服务
docker run -d --name pdf-rest-server \
  -p 8000:8000 \
  -v pdf_data:/app/data \
  smile9493/pdf-mcp:latest \
  pdf-rest --host 0.0.0.0 --port 8000
```

### 本地构建镜像

```bash
# 先编译二进制
cd pdf-module-rs && cargo build --release

# 复制二进制并构建镜像
cp target/release/pdf-rest . && cp target/release/pdf-mcp .
docker build -f Dockerfile.local -t smile9493/pdf-mcp:latest .

# 推送到 Docker Hub
docker push smile9493/pdf-mcp:latest
```

---

## 1Panel 编排

项目提供了 1Panel 专用编排配置，位于 `deploy/1panel/`。

### 部署步骤

1. **上传配置**：将 `deploy/1panel/` 目录上传到服务器

2. **配置环境变量**：
```bash
cd deploy/1panel
cp .env.example .env
# 编辑 .env 修改端口和配置
```

3. **启动服务**：
```bash
docker-compose up -d
```

### 配置项

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `MCP_PORT` | 8001 | MCP SSE 端口 |
| `REST_PORT` | 8000 | REST API 端口 |
| `LOG_LEVEL` | info | 日志级别 |
| `CACHE_ENABLED` | true | 启用缓存 |
| `CACHE_MAX_SIZE` | 1000 | 缓存最大条目 |
| `MAX_FILE_SIZE_MB` | 100 | 最大文件大小 |

### 1Panel 应用商店集成

在 1Panel 中创建自定义应用：

1. 进入 **应用商店** → **已安装** → **创建应用**
2. 选择 **Compose** 模式
3. 填写：
   - 名称：`pdf-mcp-server`
   - Compose 内容：复制 `deploy/1panel/docker-compose.yml`
4. 配置环境变量后启动

---

## Agent 集成

### Cursor

创建 `~/.cursor/mcp.json`（项目级 `.cursor/mcp.json` 也可）：

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "docker",
      "args": ["exec", "-i", "pdf-mcp-server", "pdf-mcp", "serve", "--transport", "stdio"]
    }
  }
}
```

或使用本地二进制：
```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/path/to/pdf-mcp",
      "args": ["serve", "--transport", "stdio"]
    }
  }
}
```

### Claude Desktop

编辑配置文件：
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "docker",
      "args": ["exec", "-i", "pdf-mcp-server", "pdf-mcp", "serve", "--transport", "stdio"]
    }
  }
}
```

配置后重启 IDE，PDF 工具将自动可用。Agent 可直接调用 `extract_text`、`search_keywords` 等工具处理 PDF 文档。

---

## Python SDK

SDK 位于 `pdf-mcp-sdk/`，提供三种客户端。

### 安装

```bash
pip install -e ./pdf-mcp-sdk          # 基础安装
pip install -e "./pdf-mcp-sdk[async]" # 含异步支持
```

### MCP 客户端（推荐）

```python
from pdf_mcp_sdk import PDFMCPClient

with PDFMCPClient("http://localhost:8001") as client:
    # 提取文本
    result = client.extract_text("/path/to/file.pdf", adapter="pdf-extract")
    print(f"提取 {len(result.text)} 字符，共 {result.page_count} 页")

    # 搜索关键词
    search = client.search_keywords("/path/to/file.pdf", ["合同", "协议"])
    print(f"找到 {search.total_matches} 个匹配")

    # 获取页数
    pages = client.get_page_count("/path/to/file.pdf")

    # 列出引擎
    adapters = client.list_adapters()

    # 缓存统计
    stats = client.get_cache_stats()
```

### 异步客户端

```python
import asyncio
from pdf_mcp_sdk import AsyncPDFMCPClient

async def main():
    async with AsyncPDFMCPClient("http://localhost:8001") as client:
        result = await client.extract_text("/path/to/file.pdf")
        print(result.text)

asyncio.run(main())
```

### REST 客户端

```python
from pdf_mcp_sdk import PDFRestClient

with PDFRestClient("http://localhost:8000") as client:
    result = client.extract_text_from_path("/path/to/file.pdf")
    print(result.text)
```

### SDK 提供的工具方法

| 方法 | 客户端 | 说明 |
|------|--------|------|
| `extract_text(path, adapter)` | MCP/REST/Async | 提取文本 |
| `extract_structured(path, adapter)` | MCP/REST | 提取结构化数据 |
| `get_page_count(path)` | MCP/REST/Async | 获取页数 |
| `search_keywords(path, keywords, case_sensitive)` | MCP/Async | 搜索关键词 |
| `extract_keywords(path, top_n)` | MCP | 提取高频关键词 |
| `list_adapters()` | MCP/REST/Async | 列出引擎 |
| `get_cache_stats()` | MCP/REST/Async | 缓存统计 |

---

## MCP 工具参考

### 协议信息

- 协议版本：`2024-11-05`
- 服务名称：`pdf-module-mcp`
- 传输方式：stdio / SSE
- SSE 端点：`/sse` (GET)、`/message` (POST)、`/health` (GET)

### 工具列表

#### `extract_text` — 提取 PDF 文本

```json
{
  "name": "extract_text",
  "arguments": {
    "file_path": "/absolute/path/to/file.pdf",
    "adapter": "pdf-extract"
  }
}
```

**参数**：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file_path` | string | 是 | PDF 文件绝对路径 |
| `adapter` | string | 否 | 引擎：lopdf / pdf-extract / pdfium |

**返回**：纯文本字符串

---

#### `extract_structured` — 提取结构化数据

```json
{
  "name": "extract_structured",
  "arguments": {
    "file_path": "/path/to/file.pdf",
    "adapter": "lopdf",
    "enable_highlight": true
  }
}
```

**返回**：
```json
{
  "extracted_text": "全部文本",
  "page_count": 10,
  "pages": [
    {
      "page_number": 1,
      "text": "页面文本",
      "bbox": [0.0, 0.0, 595.0, 842.0],
      "lines": [{"bbox": [x0,y0,x1,y1], "text": "行文本"}]
    }
  ],
  "file_info": {"file_path": "...", "file_size": 102400, "file_size_mb": 0.1}
}
```

---

#### `get_page_count` — 获取页数

```json
{"name": "get_page_count", "arguments": {"file_path": "/path/to/file.pdf"}}
```

**返回**：整数（如 `17`）

---

#### `search_keywords` — 搜索关键词

```json
{
  "name": "search_keywords",
  "arguments": {
    "file_path": "/path/to/file.pdf",
    "keywords": ["合同", "协议"],
    "case_sensitive": false,
    "context_length": 50
  }
}
```

**参数**：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file_path` | string | 是 | PDF 文件绝对路径 |
| `keywords` | string[] | 是 | 关键词列表 |
| `case_sensitive` | boolean | 否 | 大小写敏感（默认 false） |
| `context_length` | integer | 否 | 匹配上下文字符数（默认 50） |

**返回**：
```json
{
  "keywords": ["合同"],
  "matches": [
    {"keyword": "合同", "page_number": 1, "text": "...上下文...", "bbox": [x0,y0,x1,y1], "confidence": 1.0}
  ],
  "total_matches": 5,
  "pages_with_matches": [1, 3, 5]
}
```

---

#### `extract_keywords` — 提取高频关键词

```json
{"name": "extract_keywords", "arguments": {"file_path": "/path/to/file.pdf", "top_n": 10}}
```

**返回**：`[["关键词1", 15], ["关键词2", 10], ...]`

---

#### `list_adapters` — 列出提取引擎

**返回**：
```json
[
  {"id": "lopdf", "name": "LopdfEngine", "description": "Layout-aware PDF engine based on lopdf"},
  {"id": "pdf-extract", "name": "PdfExtractEngine", "description": "Fast text extraction engine based on pdf-extract"},
  {"id": "pdfium", "name": "PdfiumEngine", "description": "High-compatibility PDF engine based on PDFium"}
]
```

---

#### `cache_stats` — 缓存统计

**返回**：`{"size": 5, "max_size": 1000, "hits": 20, "misses": 5, "hit_rate": 0.8}`

---

## REST API 参考

| 方法 | 端点 | 说明 |
|------|------|------|
| GET | `/api/v1/x2text/health` | 健康检查 |
| POST | `/api/v1/x2text/extract` | 提取文本（返回纯文本） |
| POST | `/api/v1/x2text/extract-json` | 提取结构化数据（返回 JSON） |
| POST | `/api/v1/x2text/info` | 获取 PDF 信息 |
| GET | `/api/v1/x2text/adapters` | 列出引擎 |
| GET | `/api/v1/x2text/cache/stats` | 缓存统计 |

### 请求示例

```bash
# 提取文本
curl -X POST http://localhost:8000/api/v1/x2text/extract \
  -F "file=@document.pdf" -F "adapter=lopdf"

# 提取结构化数据
curl -X POST http://localhost:8000/api/v1/x2text/extract-json \
  -F "file=@document.pdf" -F "adapter=lopdf"

# 获取信息
curl -X POST http://localhost:8000/api/v1/x2text/info \
  -F "file=@document.pdf"
```

### 错误响应

```json
{"error": "Bad Request", "message": "具体错误", "status_code": 400}
```

| 错误类型 | HTTP 状态码 |
|----------|-------------|
| FileNotFound | 404 |
| InvalidFileType / AdapterNotFound | 400 |
| FileTooLarge | 413 |
| CorruptedFile | 422 |
| 其他 | 500 |

---

## 提取引擎

| 引擎 | ID | 特点 | 适用场景 |
|------|-----|------|----------|
| Lopdf | `lopdf` | 布局感知，支持 MediaBox 递归 | 通用、需要布局信息 |
| PDF Extract | `pdf-extract` | 快速文本提取 | 简单文档、批量处理 |
| PDFium | `pdfium` | Chrome PDF 引擎，高兼容 | 特殊编码、复杂 PDF |

### 智能路由

系统自动根据文档特征选择最优引擎：

1. **小文档** (≤5页) + 无复杂布局 → `pdf-extract`
2. **特殊编码** (CIDFont/Type3) → `pdfium`
3. **默认** → `lopdf`

### 熔断器

每个引擎独立熔断，连续 5 次失败后进入 Open 状态（60秒冷却），之后自动 HalfOpen 试探。主引擎失败自动回退到 `pdfium`。

---

## 配置参考

### 环境变量

```bash
# ===== 服务器 =====
SERVER_NAME=pdf-module-mcp        # 服务名称
SERVER_VERSION=0.2.0              # 服务版本
ENVIRONMENT=production            # 环境: development/staging/production

# ===== 存储 =====
STORAGE_TYPE=local                # 存储类型: local/s3/gcs/azure
LOCAL_STORAGE_BASE_DIR=/app/data  # 本地存储目录

# ===== 缓存 =====
CACHE_ENABLED=true                # 启用缓存
CACHE_MAX_SIZE=1000               # 最大条目数
CACHE_TTL_SECONDS=3600            # TTL (秒)

# ===== 审计 =====
AUDIT_ENABLED=true                # 启用审计
AUDIT_BACKEND=file                # 后端: file/database/remote/memory
AUDIT_LOG_DIR=/app/logs/audit     # 日志目录
AUDIT_RETENTION_DAYS=30           # 保留天数

# ===== 日志 =====
LOG_LEVEL=info                    # 级别: trace/debug/info/warn/error
LOG_FORMAT=json                   # 格式: json/text

# ===== 安全 =====
MAX_FILE_SIZE_MB=100              # 最大文件大小 (MB)
PATH_REQUIRE_ABSOLUTE=true        # 要求绝对路径
PATH_ALLOW_TRAVERSAL=false        # 禁止路径遍历
```

---

## 性能优化

| 优化项 | 实现方式 | 提升效果 |
|--------|----------|----------|
| 正则缓存 | `Lazy<Mutex<HashMap>>` 缓存编译后正则 | 10-100x |
| Jieba 缓存 | `Lazy<Jieba>` 全局单例 | 5-10x |
| 缓存键优化 | 小文件用 mtime+size，大文件用首尾1MB部分哈希 | 2-5x |
| 异步 I/O | Tokio 异步运行时 | 高并发 |
| Moka 缓存 | 并发安全、TTL 支持 | 低延迟 |
| 智能路由 | 自动选择最优引擎 | 整体最优 |

---

## 安全机制

| 机制 | 说明 |
|------|------|
| 路径遍历防护 | 禁止 `..`、可选限制基础目录 |
| 文件类型检测 | `infer` crate 嗅探 + `%PDF` 头部回退 |
| 文件大小限制 | 可配置上限 (默认 100MB) |
| 绝对路径要求 | 默认仅允许绝对路径 |
| 审计日志 | JSONL 格式，支持查询和自动清理 |
| 非 root 运行 | 容器内以 pdfuser (uid 1000) 运行 |

---

## 架构设计

```
┌─────────────────────────────────────────────────┐
│                   Client Layer                   │
│  Cursor / Claude Desktop / Python SDK / HTTP    │
└──────────┬──────────────────────┬───────────────┘
           │                      │
     ┌─────▼─────┐          ┌────▼────┐
     │  MCP SSE  │          │ REST API│
     │  :8001    │          │  :8000  │
     └─────┬─────┘          └────┬────┘
           │                      │
     ┌─────▼──────────────────────▼─────┐
     │        PdfExtractorService        │
     │  ┌──────────┐  ┌──────────────┐  │
     │  │  Smart   │  │   Circuit    │  │
     │  │  Router  │  │   Breaker    │  │
     │  └────┬─────┘  └──────────────┘  │
     │       │                          │
     │  ┌────▼──────────────────────┐   │
     │  │    Engine Registry        │   │
     │  │  lopdf | pdf-extract | pdfium│
     │  └──────────────────────────┘   │
     │  ┌──────────┐  ┌──────────────┐ │
     │  │  Cache   │  │   Keyword    │ │
     │  │  (Moka)  │  │  Extractor   │ │
     │  └──────────┘  └──────────────┘ │
     │  ┌──────────┐  ┌──────────────┐ │
     │  │ Validator│  │    Audit     │ │
     │  └──────────┘  └──────────────┘ │
     └──────────────────────────────────┘
```

---

## 开发指南

### 项目结构

```
pdf-module-rs/
├── crates/
│   ├── pdf-core/       # 核心库
│   │   ├── engine/     # 引擎实现 (lopdf, pdf-extract, pdfium)
│   │   ├── cache/      # 缓存系统
│   │   ├── keyword/    # 关键词提取 (jieba + regex)
│   │   ├── validator/  # 文件验证
│   │   ├── audit/      # 审计日志
│   │   ├── storage/    # 存储后端
│   │   ├── metrics/    # Prometheus 指标
│   │   └── plugin/     # 插件系统
│   ├── pdf-mcp/        # MCP Server
│   ├── pdf-rest/       # REST API Server
│   └── pdf-python/     # Python FFI

pdf-mcp-sdk/             # Python SDK
├── pdf_mcp_sdk/
│   ├── client.py        # 同步 MCP 客户端
│   ├── async_client.py  # 异步 MCP 客户端
│   ├── rest_client.py   # REST 客户端
│   ├── models.py        # 数据模型
│   └── exceptions.py    # 异常定义
├── examples/            # 使用示例
└── config/              # Cursor/Claude 配置

deploy/
└── 1panel/              # 1Panel 部署配置
```

### 构建 & 测试

```bash
cargo build --release           # 构建
cargo test                      # 测试
cargo test -p pdf-core -- --nocapture  # 带输出测试
```

### 添加新引擎

1. 在 `crates/pdf-core/src/engine/` 创建新文件
2. 实现 `PdfEngine` trait
3. 在 `PdfExtractorService::new()` 中注册

---

## License

MIT License
