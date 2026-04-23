# PDF Module - MCP Server

一个高性能的 PDF 文本提取 MCP (Model Context Protocol) 服务器，支持多种 PDF 提取引擎和灵活的配置选项。

## 🌟 特性

- **多引擎支持**: 支持 lopdf、pdf-extract、pdfium 等多种 PDF 提取引擎
- **MCP 协议**: 完整的 MCP (Model Context Protocol) 协议支持，可直接集成到 Cursor、Claude Desktop
- **REST API**: 提供 REST API 接口
- **高性能**: 基于异步 I/O、智能缓存、正则表达式缓存、Jieba实例缓存
- **Python SDK**: 提供完整的 Python SDK，支持同步/异步调用
- **多存储后端**: 支持本地、S3、GCS、Azure Blob 等多种存储后端
- **审计日志**: 完整的操作审计和日志记录
- **安全验证**: 文件路径安全验证和文件类型检查

## 📦 项目结构

```
pdf-module/
├── pdf-module-rs/          # Rust 实现
│   ├── crates/
│   │   ├── pdf-core/       # 核心库
│   │   ├── pdf-mcp/        # MCP 服务器
│   │   ├── pdf-rest/       # REST API 服务器
│   │   └── pdf-python/     # Python 绑定
│   ├── Cargo.toml
│   └── .env.example
├── pdf-mcp-sdk/            # Python SDK
│   ├── pdf_mcp_sdk/
│   │   ├── client.py       # 同步 MCP 客户端
│   │   ├── async_client.py # 异步 MCP 客户端
│   │   └── rest_client.py  # REST API 客户端
│   ├── examples/           # 使用示例
│   └── config/             # Cursor/Claude 配置
├── docs/                   # 文档
├── Dockerfile
├── docker-compose.yml
└── README.md
```

## 🚀 快速开始

### 环境要求

- Rust 1.83 或更高版本
- Cargo 包管理器
- Docker (可选)

### 本地开发

1. 克隆项目:
```bash
git clone <repository-url>
cd pdf-module
```

2. 构建项目:
```bash
cd pdf-module-rs
cargo build --release
```

3. 运行测试:
```bash
cargo test
```

### Docker 部署

```bash
# 构建镜像
docker build -t pdf-module:latest .

# 运行 MCP 服务
docker run -d --name pdf-mcp-server -p 8001:8001 pdf-module:latest \
  pdf-mcp serve --transport sse --port 8001

# 或使用 docker-compose
docker-compose up -d
```

## 🤖 Agent 集成

### 方式一：Cursor/Claude Desktop（推荐）

**Cursor 配置** - 创建 `~/.cursor/mcp.json`:
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

**Claude Desktop 配置** - 编辑 `~/Library/Application Support/Claude/claude_desktop_config.json`:
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

配置后重启 Cursor/Claude，PDF 工具将自动可用。

### 方式二：Python SDK

```python
from pdf_mcp_sdk import PDFMCPClient

# 连接 MCP 服务
client = PDFMCPClient("http://localhost:8001")

# 提取文本
result = client.extract_text("/path/to/file.pdf")
print(f"提取了 {len(result.text)} 字符，共 {result.page_count} 页")

# 搜索关键词
search = client.search_keywords("/path/to/file.pdf", ["合同", "协议"])
print(f"找到 {search.total_matches} 个匹配")

# 列出可用引擎
adapters = client.list_adapters()
for adapter in adapters:
    print(f"- {adapter.id}: {adapter.description}")

client.close()
```

更多示例见 [pdf-mcp-sdk/examples/](pdf-mcp-sdk/examples/)

## 📖 MCP 工具列表

| 工具 | 描述 |
|------|------|
| `extract_text` | 提取 PDF 文本内容 |
| `extract_structured` | 提取结构化数据（含页码、位置） |
| `get_page_count` | 获取 PDF 页数 |
| `search_keywords` | 搜索关键词 |
| `extract_keywords` | 自动提取高频关键词 |
| `list_adapters` | 列出可用的提取引擎 |
| `cache_stats` | 获取缓存统计 |

## 🌐 REST API

### 端点

- `GET /api/v1/x2text/health` - 健康检查
- `POST /api/v1/x2text/extract` - 提取文本
- `POST /api/v1/x2text/extract-json` - 提取结构化数据
- `POST /api/v1/x2text/info` - 获取 PDF 信息
- `GET /api/v1/x2text/adapters` - 列出提取引擎
- `GET /api/v1/x2text/cache/stats` - 获取缓存统计

### 示例

```bash
# 提取文本
curl -X POST http://localhost:8000/api/v1/x2text/extract \
  -F "file=@document.pdf" \
  -F "adapter=lopdf"

# 获取 PDF 信息
curl -X POST http://localhost:8000/api/v1/x2text/info \
  -F "file=@document.pdf"
```

## 🔧 提取引擎

| 引擎 | ID | 特点 |
|------|-----|------|
| Lopdf | `lopdf` | 布局感知，通用性强 |
| PDF Extract | `pdf-extract` | 快速文本提取 |
| PDFium | `pdfium` | 高兼容性，Chrome PDF 引擎 |

## ⚙️ 配置

### 环境变量

```bash
# 服务器
HOST=0.0.0.0
PORT=8000

# 存储
STORAGE_TYPE=local
STORAGE_LOCAL_DIR=/app/data

# 缓存
CACHE_ENABLED=true
CACHE_MAX_SIZE_MB=100
CACHE_TTL_SECONDS=3600

# 审计
AUDIT_ENABLED=true
AUDIT_LOG_DIR=/app/logs/audit
AUDIT_RETENTION_DAYS=30

# 日志
LOG_LEVEL=info
LOG_FORMAT=text

# 安全
ENABLE_CORS=true
ALLOWED_ORIGINS=*
MAX_FILE_SIZE_MB=100
ALLOWED_EXTENSIONS=pdf
```

## 📊 性能优化

- **异步 I/O**: 基于 Tokio 异步运行时
- **智能缓存**: Moka 高性能缓存，支持 TTL
- **正则缓存**: 编译后正则表达式缓存，提升 10-100 倍
- **Jieba 缓存**: 中文分词实例缓存，提升 5-10 倍
- **缓存键优化**: 小文件用元数据，大文件用部分哈希，提升 2-5 倍

## 🔒 安全

- **路径验证**: 防止目录遍历攻击
- **文件类型检查**: 基于文件头的深度检测
- **大小限制**: 可配置的文件大小限制
- **CORS 支持**: 可配置的跨域资源共享
- **审计日志**: 完整的操作审计追踪

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定模块
cargo test -p pdf-core

# 运行带输出
cargo test -- --nocapture
```

## 📝 许可证

MIT License

## 🙏 致谢

- [lopdf](https://github.com/J-F-Liu/lopdf) - PDF 处理库
- [pdf-extract](https://github.com/j_frolo/pdf-extract) - PDF 文本提取
- [pdfium-render](https://github.com/ajrcarey/pdfium-render) - PDFium 绑定
- [axum](https://github.com/tokio-rs/axum) - Web 框架
- [moka](https://github.com/moka-rs/moka) - 高性能缓存
