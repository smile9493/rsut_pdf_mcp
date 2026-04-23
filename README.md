# PDF Module MCP Server

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/Docker-ready-blue.svg)](https://www.docker.com/)

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
- [开发指南](#开发指南)

---

## 特性

| 类别 | 特性 |
|------|------|
| **协议支持** | MCP stdio/SSE 双传输模式、REST API |
| **提取引擎** | lopdf (布局感知)、pdf-extract (快速)、pdfium (高兼容) |
| **智能路由** | 根据文档特征自动选择最优引擎 |
| **容错机制** | 熔断器 (Circuit Breaker)、自动回退 |
| **缓存优化** | Moka 并发缓存、正则表达式缓存、Jieba 实例缓存 |
| **安全防护** | 路径遍历防护、文件类型深度检测、大小限制 |
| **可观测性** | Prometheus 指标、JSON 审计日志、结构化日志 |
| **客户端 SDK** | Python SDK (同步/异步/REST) |
| **部署方式** | Docker、1Panel、docker-compose |

---

## 快速开始

### 环境要求

| 依赖 | 版本 | 用途 |
|------|------|------|
| Rust | 1.83+ | 源码编译 |
| Docker | 20.10+ | 容器部署 |

### 方式一：Docker（推荐）

```bash
# 拉取镜像
docker pull smile9493/pdf-mcp:latest

# 启动 MCP 服务
docker run -d --name pdf-mcp-server \
  -p 8001:8001 \
  -v pdf_data:/app/data \
  smile9493/pdf-mcp:latest \
  pdf-mcp serve --transport sse --port 8001

# 启动 REST 服务
docker run -d --name pdf-rest-server \
  -p 8000:8000 \
  -v pdf_data:/app/data \
  smile9493/pdf-mcp:latest \
  pdf-rest --host 0.0.0.0 --port 8000
```

### 方式二：源码编译

```bash
# 克隆仓库
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp/pdf-module-rs

# 编译
cargo build --release

# 运行 MCP 服务 (stdio 模式 - 供 Cursor/Claude 使用)
./target/release/pdf-mcp serve --transport stdio

# 运行 MCP 服务 (SSE 模式 - 供 HTTP 客户端使用)
./target/release/pdf-mcp serve --transport sse --port 8001

# 运行 REST 服务
./target/release/pdf-rest --host 0.0.0.0 --port 8000
```

### 验证安装

```bash
# MCP 健康检查
curl http://localhost:8001/health
# 输出: OK

# REST 健康检查
curl http://localhost:8000/api/v1/x2text/health
# 输出: OK

# 列出可用引擎
curl http://localhost:8000/api/v1/x2text/adapters
```

---

## Docker 部署

### 镜像信息

| 镜像 | 大小 | 说明 |
|------|------|------|
| `smile9493/pdf-mcp:latest` | ~180MB | 基于 debian:bookworm-slim |

### 构建镜像

```bash
cd pdf-module-rs

# 编译二进制
cargo build --release

# 复制二进制到构建上下文
cp target/release/pdf-rest .
cp target/release/pdf-mcp .

# 构建镜像
docker build -f Dockerfile.local -t smile9493/pdf-mcp:latest .

# 推送到 Docker Hub
docker push smile9493/pdf-mcp:latest
```

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `RUST_LOG` | `info` | 日志级别 |
| `STORAGE_TYPE` | `local` | 存储类型 |
| `STORAGE_LOCAL_DIR` | `/app/data` | 本地存储目录 |
| `CACHE_ENABLED` | `true` | 启用缓存 |
| `CACHE_MAX_SIZE` | `1000` | 缓存最大条目数 |
| `MAX_FILE_SIZE_MB` | `100` | 最大文件大小 |

---

## 1Panel 编排

项目提供 1Panel 专用编排配置，位于 `deploy/1panel/` 目录。

### 部署步骤

**1. 上传配置文件**

将 `deploy/1panel/` 目录上传到服务器。

**2. 配置环境变量**

```bash
cd deploy/1panel
cp .env.example .env

# 编辑 .env 文件
vim .env
```

**3. 启动服务**

```bash
docker-compose up -d
```

**4. 验证服务**

```bash
# 检查容器状态
docker-compose ps

# 查看日志
docker-compose logs -f
```

### 1Panel 应用商店集成

在 1Panel 中创建自定义应用：

1. 进入 **应用商店** → **已安装** → **创建应用**
2. 选择 **Compose** 模式
3. 填写应用信息：
   - 名称：`pdf-mcp-server`
   - Compose 内容：粘贴 `deploy/1panel/docker-compose.yml` 内容
4. 配置环境变量后启动

### 配置项说明

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `MCP_PORT` | 8001 | MCP SSE 服务端口 |
| `REST_PORT` | 8000 | REST API 服务端口 |
| `LOG_LEVEL` | info | 日志级别 (trace/debug/info/warn/error) |
| `LOG_FORMAT` | json | 日志格式 (json/text) |
| `CACHE_ENABLED` | true | 启用缓存 |
| `CACHE_MAX_SIZE` | 1000 | 缓存最大条目数 |
| `CACHE_TTL` | 3600 | 缓存过期时间(秒) |
| `AUDIT_ENABLED` | true | 启用审计日志 |
| `AUDIT_RETENTION_DAYS` | 30 | 审计日志保留天数 |
| `MAX_FILE_SIZE_MB` | 100 | 最大文件大小(MB) |

---

## Agent 集成

### Cursor 配置

创建配置文件 `~/.cursor/mcp.json`（或项目级 `.cursor/mcp.json`）：

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

### Claude Desktop 配置

编辑配置文件：
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

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

### 使用说明

配置完成后重启 IDE，PDF 工具将自动可用。Agent 可直接调用以下工具：

- `extract_text` - 提取 PDF 文本
- `extract_structured` - 提取结构化数据
- `get_page_count` - 获取页数
- `search_keywords` - 搜索关键词
- `extract_keywords` - 提取高频关键词
- `list_adapters` - 列出提取引擎
- `cache_stats` - 缓存统计

---

## Python SDK

SDK 位于 `pdf-mcp-sdk/` 目录，提供三种客户端实现。

### 安装

```bash
# 基础安装
pip install -e ./pdf-mcp-sdk

# 含异步支持
pip install -e "./pdf-mcp-sdk[async]"
```

### MCP 客户端（推荐）

```python
from pdf_mcp_sdk import PDFMCPClient

# 创建客户端
client = PDFMCPClient("http://localhost:8001")

# 提取文本
result = client.extract_text("/path/to/file.pdf", adapter="pdf-extract")
print(f"提取 {len(result.text)} 字符，共 {result.page_count} 页")

# 搜索关键词
search = client.search_keywords(
    "/path/to/file.pdf",
    keywords=["合同", "协议"],
    case_sensitive=False
)
print(f"找到 {search.total_matches} 个匹配")

# 获取页数
pages = client.get_page_count("/path/to/file.pdf")

# 列出引擎
adapters = client.list_adapters()
for a in adapters:
    print(f"{a.id}: {a.description}")

# 缓存统计
stats = client.get_cache_stats()
print(f"缓存命中率: {stats.hit_rate:.2%}")

# 关闭客户端
client.close()
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
    # 从文件路径提取
    result = client.extract_text_from_path("/path/to/file.pdf")
    print(result.text)
    
    # 从文件对象提取
    with open("document.pdf", "rb") as f:
        result = client.extract_text(f, adapter="lopdf")
        print(result.text)
```

### SDK API 参考

| 方法 | 客户端 | 说明 |
|------|--------|------|
| `extract_text(path, adapter)` | 全部 | 提取文本 |
| `extract_structured(path, adapter)` | MCP/REST | 提取结构化数据 |
| `get_page_count(path)` | 全部 | 获取页数 |
| `search_keywords(path, keywords, case_sensitive)` | MCP/Async | 搜索关键词 |
| `extract_keywords(path, top_n)` | MCP | 提取高频关键词 |
| `list_adapters()` | 全部 | 列出引擎 |
| `get_cache_stats()` | 全部 | 缓存统计 |

---

## MCP 工具参考

### 协议信息

| 项目 | 值 |
|------|-----|
| 协议版本 | `2024-11-05` |
| 服务名称 | `pdf-module-mcp` |
| 服务版本 | `0.1.0` |
| 传输方式 | stdio / SSE |

### SSE 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/sse` | GET | SSE 流式连接 |
| `/message` | POST | JSON-RPC 请求 |
| `/health` | GET | 健康检查 |

---

### 工具：`extract_text`

提取 PDF 文本内容。

**请求示例**：
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

### 工具：`extract_structured`

提取结构化数据，包含页码、位置信息。

**请求示例**：
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

**返回格式**：
```json
{
  "extracted_text": "全部文本",
  "page_count": 10,
  "pages": [
    {
      "page_number": 1,
      "text": "页面文本",
      "bbox": [0.0, 0.0, 595.0, 842.0],
      "lines": [
        {"bbox": [x0, y0, x1, y1], "text": "行文本"}
      ]
    }
  ],
  "file_info": {
    "file_path": "/path/to/file.pdf",
    "file_size": 102400,
    "file_size_mb": 0.1
  }
}
```

---

### 工具：`get_page_count`

获取 PDF 页数。

**请求示例**：
```json
{
  "name": "get_page_count",
  "arguments": {
    "file_path": "/path/to/file.pdf"
  }
}
```

**返回**：整数（如 `17`）

---

### 工具：`search_keywords`

在 PDF 中搜索关键词。

**请求示例**：
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

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `file_path` | string | 是 | - | PDF 文件绝对路径 |
| `keywords` | string[] | 是 | - | 关键词列表 |
| `case_sensitive` | boolean | 否 | false | 大小写敏感 |
| `context_length` | integer | 否 | 50 | 匹配上下文字符数 |

**返回格式**：
```json
{
  "keywords": ["合同"],
  "matches": [
    {
      "keyword": "合同",
      "page_number": 1,
      "text": "...上下文文本...",
      "bbox": [x0, y0, x1, y1],
      "start_index": 100,
      "end_index": 102,
      "confidence": 1.0
    }
  ],
  "total_matches": 5,
  "pages_with_matches": [1, 3, 5]
}
```

---

### 工具：`extract_keywords`

自动提取高频关键词。

**请求示例**：
```json
{
  "name": "extract_keywords",
  "arguments": {
    "file_path": "/path/to/file.pdf",
    "top_n": 10
  }
}
```

**返回**：`[["关键词1", 15], ["关键词2", 10], ...]`

---

### 工具：`list_adapters`

列出可用的 PDF 提取引擎。

**返回**：
```json
[
  {
    "id": "lopdf",
    "name": "LopdfEngine",
    "description": "Layout-aware PDF engine based on lopdf"
  },
  {
    "id": "pdf-extract",
    "name": "PdfExtractEngine",
    "description": "Fast text extraction engine based on pdf-extract"
  },
  {
    "id": "pdfium",
    "name": "PdfiumEngine",
    "description": "High-compatibility PDF engine based on PDFium"
  }
]
```

---

### 工具：`cache_stats`

获取缓存统计信息。

**返回**：
```json
{
  "size": 5,
  "max_size": 1000,
  "hits": 20,
  "misses": 5,
  "hit_rate": 0.8
}
```

---

## REST API 参考

### 端点列表

| 方法 | 端点 | 说明 |
|------|------|------|
| GET | `/api/v1/x2text/health` | 健康检查 |
| POST | `/api/v1/x2text/extract` | 提取文本（返回纯文本） |
| POST | `/api/v1/x2text/extract-json` | 提取结构化数据（返回 JSON） |
| POST | `/api/v1/x2text/info` | 获取 PDF 信息 |
| GET | `/api/v1/x2text/adapters` | 列出引擎 |
| GET | `/api/v1/x2text/cache/stats` | 缓存统计 |

### 请求示例

**提取文本**：
```bash
curl -X POST http://localhost:8000/api/v1/x2text/extract \
  -F "file=@document.pdf" \
  -F "adapter=lopdf"
```

**提取结构化数据**：
```bash
curl -X POST http://localhost:8000/api/v1/x2text/extract-json \
  -F "file=@document.pdf" \
  -F "adapter=lopdf"
```

**获取 PDF 信息**：
```bash
curl -X POST http://localhost:8000/api/v1/x2text/info \
  -F "file=@document.pdf"
```

**列出引擎**：
```bash
curl http://localhost:8000/api/v1/x2text/adapters
```

### 错误响应

```json
{
  "error": "Bad Request",
  "message": "具体错误信息",
  "status_code": 400
}
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

### 引擎对比

| 引擎 | ID | 特点 | 适用场景 |
|------|-----|------|----------|
| Lopdf | `lopdf` | 布局感知，支持 MediaBox 递归解析 | 通用场景、需要布局信息 |
| PDF Extract | `pdf-extract` | 快速文本提取 | 简单文档、批量处理 |
| PDFium | `pdfium` | Chrome PDF 引擎，高兼容性 | 特殊编码、复杂 PDF |

### 智能路由

系统根据文档特征自动选择最优引擎：

```
┌─────────────────────────────────────┐
│         文档特征检测                │
└──────────────┬──────────────────────┘
               │
    ┌──────────┴──────────┐
    │  页数 ≤ 5 且无复杂布局？  │
    └──────────┬──────────┘
          是 │     │ 否
             ▼     │
      ┌──────────┐ │
      │ pdf-extract│ │
      └──────────┘ │
                   │
    ┌──────────────┴──────────┐
    │  检测到特殊编码(CIDFont)？  │
    └──────────┬──────────────┘
          是 │     │ 否
             ▼     │
      ┌──────────┐ │
      │  pdfium  │ │
      └──────────┘ │
                   ▼
            ┌──────────┐
            │  lopdf   │
            └──────────┘
```

### 熔断器

每个引擎独立熔断：

- **失败阈值**：5 次连续失败
- **冷却时间**：60 秒
- **状态转换**：Closed → Open → HalfOpen

主引擎失败时自动回退到 `pdfium`。

---

## 配置参考

### 环境变量完整列表

```bash
# ===== 服务器配置 =====
SERVER_NAME=pdf-module-mcp        # 服务名称
SERVER_VERSION=0.2.0              # 服务版本
ENVIRONMENT=production            # 环境: development/staging/production

# ===== 存储配置 =====
STORAGE_TYPE=local                # 存储类型: local/s3/gcs/azure
LOCAL_STORAGE_BASE_DIR=/app/data  # 本地存储目录

# S3 配置 (STORAGE_TYPE=s3 时需要)
S3_BUCKET=                        # S3 桶名
S3_REGION=us-east-1               # S3 区域
S3_ACCESS_KEY=                    # S3 访问密钥
S3_SECRET_KEY=                    # S3 秘密密钥
S3_ENDPOINT=                      # S3 自定义端点

# ===== 缓存配置 =====
CACHE_ENABLED=true                # 启用缓存
CACHE_MAX_SIZE=1000               # 最大条目数
CACHE_TTL_SECONDS=3600            # TTL (秒)

# ===== 审计配置 =====
AUDIT_ENABLED=true                # 启用审计
AUDIT_BACKEND=file                # 后端: file/database/remote/memory
AUDIT_LOG_DIR=/app/logs/audit     # 日志目录
AUDIT_RETENTION_DAYS=30           # 保留天数

# ===== 日志配置 =====
LOG_LEVEL=info                    # 级别: trace/debug/info/warn/error
LOG_FORMAT=json                   # 格式: json/text

# ===== 安全配置 =====
MAX_FILE_SIZE_MB=100              # 最大文件大小 (MB)
PATH_REQUIRE_ABSOLUTE=true        # 要求绝对路径
PATH_ALLOW_TRAVERSAL=false        # 禁止路径遍历
```

---

## 性能优化

### 优化措施

| 优化项 | 实现方式 | 提升效果 |
|--------|----------|----------|
| 正则表达式缓存 | `Lazy<Mutex<HashMap>>` 缓存编译后正则 | 10-100x |
| Jieba 实例缓存 | `Lazy<Jieba>` 全局单例 | 5-10x |
| 缓存键优化 | 小文件用 mtime+size，大文件用首尾1MB部分哈希 | 2-5x |
| 异步 I/O | Tokio 异步运行时 | 高并发支持 |
| Moka 缓存 | 并发安全、TTL 支持 | 低延迟 |
| 智能路由 | 自动选择最优引擎 | 整体性能最优 |

### 缓存策略

```
小文件 (≤10MB):
  缓存键 = path + mtime + size + adapter
  优势: 无需计算哈希，极快

大文件 (>10MB):
  缓存键 = partial_hash + adapter + size
  partial_hash = SHA256(首1MB + 末1MB + 文件大小)
  优势: 避免全文件哈希计算
```

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

## 开发指南

### 项目结构

```
rsut_pdf_mcp/
├── pdf-module-rs/              # Rust 实现
│   ├── crates/
│   │   ├── pdf-core/           # 核心库
│   │   │   ├── src/
│   │   │   │   ├── engine/     # 引擎实现
│   │   │   │   ├── cache.rs    # 缓存系统
│   │   │   │   ├── keyword.rs  # 关键词提取
│   │   │   │   ├── validator.rs# 文件验证
│   │   │   │   ├── audit/      # 审计日志
│   │   │   │   ├── storage/    # 存储后端
│   │   │   │   └── metrics.rs  # Prometheus 指标
│   │   ├── pdf-mcp/            # MCP Server
│   │   ├── pdf-rest/           # REST API Server
│   │   └── pdf-python/         # Python FFI
│   ├── Cargo.toml
│   └── Dockerfile.local
│
├── pdf-mcp-sdk/                # Python SDK
│   ├── pdf_mcp_sdk/
│   │   ├── client.py           # 同步 MCP 客户端
│   │   ├── async_client.py     # 异步 MCP 客户端
│   │   ├── rest_client.py      # REST 客户端
│   │   ├── models.py           # 数据模型
│   │   └── exceptions.py       # 异常定义
│   ├── examples/               # 使用示例
│   └── config/                 # Cursor/Claude 配置
│
├── deploy/
│   └── 1panel/                 # 1Panel 部署配置
│
├── docs/                       # 文档
├── README.md
└── LICENSE
```

### 构建 & 测试

```bash
# 开发构建
cargo build

# 生产构建
cargo build --release

# 运行测试
cargo test

# 运行特定模块测试
cargo test -p pdf-core
cargo test -p pdf-core keyword
cargo test -p pdf-core cache

# 带输出测试
cargo test -- --nocapture
```

### 添加新引擎

1. 在 `crates/pdf-core/src/engine/` 创建新文件
2. 实现 `PdfEngine` trait
3. 在 `PdfExtractorService::new()` 中注册

---

## License

[MIT License](LICENSE)

---

## 相关链接

- [MCP 协议规范](https://modelcontextprotocol.io/)
- [Cursor 文档](https://cursor.sh/docs)
- [Claude Desktop](https://claude.ai/)
- [lopdf](https://github.com/J-F-Liu/lopdf)
- [pdfium-render](https://github.com/ajrcarey/pdfium-render)
