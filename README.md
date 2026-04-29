# PDF MCP Module

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Vue](https://img.shields.io/badge/Vue-3.5%2B-green.svg)](https://vuejs.org/)
[![CI](https://github.com/smile9493/rsut_pdf_mcp/actions/workflows/build.yml/badge.svg)](https://github.com/smile9493/rsut_pdf_mcp/actions)
[![Release](https://img.shields.io/github/v/release/smile9493/rsut_pdf_mcp)](https://github.com/smile9493/rsut_pdf_mcp/releases)

**极简 PDF 提取 MCP 管道** — 单一 pdfium 引擎、纯 stdio 传输、VLM 条件升级、Web 监控面板。

基于**奥卡姆剃刀**与**截拳道**设计哲学，剔除所有非核心实体，收敛至最小可运行架构。

---

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                  AI Agent (Cursor/Claude Desktop)           │
──────────────────────────┬──────────────────────────────────┘
                           │ JSON-RPC over stdio
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     pdf-mcp (二进制客户端)                   │
│  Windows: pdf-mcp.exe  │  Linux/macOS: pdf-mcp             │
├─────────────────────────────────────────────────────────────┤
│  MCP Tools:                                                 │
│  • extract_text        - 提取纯文本                         │
│  • extract_structured  - 提取结构化数据 (per-page + bbox)   │
│  • get_page_count      - 获取页数                           │
└──────────────────────────┬──────────────────────────────────┘
                           │
        ──────────────────┴──────────────────┐
        ▼                                      ▼
┌───────────────────              ┌───────────────────┐
│   PdfiumEngine    │              │   VlmGateway      │
│   (本地提取)       │              │   (条件升级)       │
│   FFI 防波堤       │              │   GPT-4o/Claude   │
───────────────────┘              └───────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   Web Dashboard (Vue 3)                     │
├─────────────────────────────────────────────────────────────┤
│  /              │ 首页 (状态、工具概览、快速操作)             │
│  /extract       │ 文本提取                                    │
│  /search        │ 关键词搜索                                  │
│  /batch         │ 批量处理                                    │
│  /mcp-tools     │ 工具测试                                    │
│  /settings      │ MCP 配置 (服务器 + VLM API Key)            │
└─────────────────────────────────────────────────────────────┘
```

---

## Web 界面预览

### 首页 - 状态总览

```
┌─────────────────────────────────────────────────────────────┐
│  📊 PDF Module Dashboard                                      │
│  极简 PDF 提取 MCP 管道 - 单一引擎、纯 stdio、条件升级         │
─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────  ┌──────────┐    │
│  │ MCP Tools│  │ Engine   │  │ Protocol │  │ VLM      │    │
│  │    4     │  │ PDFium   │  │  stdio   │  │  OFF     │    │
│  │ extract  │  │ 本地引擎  │  │JSON-RPC  │  │ 未配置   │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
├─────────────────────────────────────────────────────────────┤
│  Available Tools                                             │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │ 📄 extract_text  │  │ 🧊 extract_struct│                 │
│  │ 提取纯文本        │  │ 提取结构化数据    │                 │
│  │ [file_path]      │  │ [file_path]      │                 │
│  └──────────────────┘  ──────────────────┘                 │
│  ──────────────────┐  ┌──────────────────┐                 │
│  │ 📑 get_page_count│  │ 🔍 search_keywords│                 │
│  │ 获取页数          │  │ 关键词搜索        │                 │
│  │ [file_path]      │  │ [path][keys][cs] │                 │
│  └──────────────────┘  └──────────────────┘                 │
├─────────────────────────────────────────────────────────────┤
│  Quick Actions                                               │
│  [📄 文本提取]  [🔍 关键词搜索]  [ 批量处理]  [ 工具测试]  │
├─────────────────────────────────────────────────────────────┤
│  Architecture                                                │
│  AI Agent → pdf-mcp (stdio JSON-RPC) → PdfiumEngine → PDF   │
─────────────────────────────────────────────────────────────┘
```

### 文本提取页

```
─────────────────────────────────────────────────────────────┐
│  📄 Text Extraction                                           │
├──────────────────────┬──────────────────────────────────────┤
│  PDF File            │  📊 Stats                            │
│  ┌──────────────────┐ │  Pages: 42  Chars: 15,234  1.2s    │
│  │  📁 Drop PDF or  │ │  Engine: auto                        │
│  │     Click        │ │                                      │
│  └──────────────────┘ │  ┌──────────────────────────────┐  │
│                       │  │ # Document Title              │  │
│  Engine: [Auto ▾]     │  │ Extracted text content...     │  │
│  Mode: ○ Text         │  │  - Paragraph 1                │  │
│        ○ Structured   │  │  - Paragraph 2                │  │
│                       │  │  [Table data...]              │  │
│  [ Extract Button ]   │  ──────────────────────────────┘  │
│                       │  [📋 Copy]                          │
└──────────────────────┴──────────────────────────────────────┘
```

### 工具测试页

```
┌─────────────────────────────────────────────────────────────┐
│   MCP Tools                                                 │
├─────────────────────────────────────────────────────────────┤
│  Status: ● Connected  │  Tools: 4  │  Calls: 12  │  100%   │
─────────────────────────────────────────────────────────────┤
│  Select Tool              │  Execution Log                  │
│  ┌──────────────────┐    │  ┌───────────────────────────┐  │
│  │  extract_text  │    │  │ extract_text       ✓ 230ms│  │
│  │ 提取纯文本        │    │  │ get_page_count     ✓ 45ms │  │
│  └──────────────────┘    │  │ search_keywords    ✓ 890ms│  │
│  ┌──────────────────┐    │  │ extract_structured ✓ 1.2s │  │
│  │  extract_struct│    │  └───────────────────────────┘  │
│  │ 提取结构化数据    │    │                                 │
│  └──────────────────┘    │  Available Tools                 │
│  ┌──────────────────┐    │  ● extract_text                  │
│  │ 📑 get_page_count│    │  ● extract_structured            │
│  │ 获取页数          │    │  ● get_page_count                │
│  └──────────────────┘    │  ● search_keywords               │
│  ┌──────────────────┐    │                                 │
│  │ 🔍 search_keywords│   │                                 │
│  │ 关键词搜索        │    │                                 │
│  ──────────────────┘    │                                 │
│                           │                                 │
│  File Path: [_____________] [Browse]                        │
│  Keywords: [_______]  ○ Case Sensitive                     │
│  [ Execute ]                                                   │
└─────────────────────────────────────────────────────────────┘
```

---

## 部署指南

### 方式一：Docker Compose（推荐）

适合完整部署 MCP + Web 面板。

#### 1. 创建 `docker-compose.yml`

```yaml
version: "3.8"

services:
  # MCP 服务器 - stdio 协议
  pdf-mcp:
    image: smile9493/pdf-mcp:latest-mcp
    container_name: pdf-mcp
    restart: unless-stopped
    volumes:
      - ./data:/app/data
      - /path/to/pdfs:/pdfs:ro
    environment:
      - VLM_API_KEY=${VLM_API_KEY:-}
      - VLM_ENDPOINT=${VLM_ENDPOINT:-https://api.openai.com/v1/chat/completions}
      - VLM_MODEL=${VLM_MODEL:-gpt-4o}
    stdin_open: true
    tty: true

  # Web 前端面板
  pdf-web:
    image: smile9493/pdf-mcp:latest-web
    container_name: pdf-web
    restart: unless-stopped
    ports:
      - "80:80"
    depends_on:
      - pdf-mcp
    environment:
      - BACKEND_URL=http://pdf-mcp:8080
```

#### 2. 创建 `.env` 文件（可选）

```bash
# VLM 配置（可选，用于扫描件/混沌布局增强）
VLM_API_KEY=sk-your-api-key
VLM_ENDPOINT=https://api.openai.com/v1/chat/completions
VLM_MODEL=gpt-4o
```

#### 3. 启动服务

```bash
# 启动所有服务
docker compose up -d

# 查看日志
docker compose logs -f

# 停止服务
docker compose down
```

#### 4. 访问 Web 面板

打开浏览器访问: **http://localhost**

---

### 方式二：Docker 独立运行

#### 仅运行 MCP 服务器

```bash
# 拉取镜像
docker pull smile9493/pdf-mcp:latest-mcp

# 运行（stdin/stdout 模式，供 Agent 连接）
docker run --rm -i \
  -v /path/to/pdfs:/pdfs:ro \
  -v $(pwd)/data:/app/data \
  smile9493/pdf-mcp:latest-mcp

# 带 VLM 配置
docker run --rm -i \
  -e VLM_API_KEY=sk-xxx \
  -e VLM_MODEL=gpt-4o \
  smile9493/pdf-mcp:latest-mcp
```

#### 仅运行 Web 前端

```bash
# 拉取镜像
docker pull smile9493/pdf-mcp:latest-web

# 默认端口 80
docker run -d -p 80:80 smile9493/pdf-mcp:latest-web

# 自定义端口
docker run -d -p 3000:80 smile9493/pdf-mcp:latest-web
```

---

### 方式三：下载二进制文件

从 [Releases](https://github.com/smile9493/rsut_pdf_mcp/releases) 下载：

| 平台 | 文件 | 架构 |
|------|------|------|
| Linux x64 | `pdf-mcp-linux-x64.tar.gz` | x86_64 |
| Linux ARM64 | `pdf-mcp-linux-arm64.tar.gz` | aarch64 |
| macOS Intel | `pdf-mcp-macos-x64.tar.gz` | x86_64 |
| macOS Apple Silicon | `pdf-mcp-macos-arm64.tar.gz` | aarch64 |
| Windows x64 | `pdf-mcp-windows-x64.zip` | x86_64 |

```bash
# Linux/macOS
tar -xzf pdf-mcp-linux-x64.tar.gz
chmod +x pdf-mcp
sudo mv pdf-mcp /usr/local/bin/

# Windows - 解压 zip，将 pdf-mcp.exe 加入 PATH
```

---

### 方式四：从源码构建

```bash
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp

# 构建 MCP 服务器
cd pdf-module-rs
cargo build --release --bin pdf-mcp
# 二进制文件: target/release/pdf-mcp

# 构建 Web 前端
cd ../web
npm install
npm run build
# 静态文件: dist/
```

---

## Agent 集成

### Cursor 配置

编辑 `~/.cursor/mcp.json`：

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/usr/local/bin/pdf-mcp"
    }
  }
}
```

### Claude Desktop 配置

编辑 `~/Library/Application Support/Claude/claude_desktop_config.json`：

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/usr/local/bin/pdf-mcp"
    }
  }
}
```

### Docker 集成 (Cursor/Claude)

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "smile9493/pdf-mcp:latest-mcp"]
    }
  }
}
```

---

## VLM 视觉增强配置 (可选)

如需 VLM 视觉增强 (扫描件、混沌布局)：

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/usr/local/bin/pdf-mcp",
      "env": {
        "VLM_API_KEY": "sk-xxx",
        "VLM_ENDPOINT": "https://api.openai.com/v1/chat/completions",
        "VLM_MODEL": "gpt-4o"
      }
    }
  }
}
```

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `VLM_API_KEY` | OpenAI/Anthropic API Key | - |
| `VLM_ENDPOINT` | API 端点 | `https://api.openai.com/v1/chat/completions` |
| `VLM_MODEL` | 模型名称 | `gpt-4o` |

---

## 版本说明

### 版本标签规则

| 标签格式 | 说明 | Docker 镜像标签 |
|---------|------|----------------|
| `v1.0.0` | 正式版本 | `smile9493/pdf-mcp:1.0.0-mcp`, `smile9493/pdf-mcp:1.0.0-web` |
| `main` 分支 | 开发版本 | `smile9493/pdf-mcp:main-mcp`, `smile9493/pdf-mcp:main-web` |
| `latest` | 最新稳定版 | `smile9493/pdf-mcp:latest-mcp`, `smile9493/pdf-mcp:latest-web` |

### 发布流程

```bash
git tag v1.0.0
git push origin main
git push origin v1.0.0
```

推送 `v*` 标签后，CI 会自动构建所有平台二进制并创建 Release。

### 当前版本

- **最新版本**: [查看 Releases](https://github.com/smile9493/rsut_pdf_mcp/releases)
- **Docker 镜像**: [Docker Hub](https://hub.docker.com/r/smile9493/pdf-mcp)

---

## MCP 工具

| 工具 | 说明 | 参数 |
|------|------|------|
| `extract_text` | 提取纯文本 | `file_path` |
| `extract_structured` | 提取结构化数据 | `file_path` |
| `get_page_count` | 获取页数 | `file_path` |
| `search_keywords` | 关键词搜索 | `file_path`, `keywords`, `case_sensitive?` |

---

## 项目结构

```
pdf-module-rs/
├── crates/
│   ├── pdf-common/         # error + dto + config
│   ├── pdf-core/           # PdfiumEngine + FileValidator
│   ├── pdf-mcp/            # MCP stdio 入口
│   └── vlm-visual-gateway/ # VLM 条件升级

web/
├── src/
│   ├── views/
│   │   ├── HomeView.vue         # 首页
│   │   ├── ExtractView.vue      # 文本提取
│   │   ├── SearchView.vue       # 关键词搜索
│   │   ├── BatchProcessView.vue # 批量处理
│   │   ├── McpToolsView.vue     # 工具测试
│   │   └── SettingsView.vue     # 配置
│   └── locales/
│       ├── en.js                # 英文
│       └── zh.js                # 中文

docker/
├── Dockerfile.mcp               # MCP 服务器镜像
── Dockerfile.ci                # Web 前端镜像
```

---

## License

[MIT](LICENSE)
