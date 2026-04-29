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
│  /mcp-config   │ MCP 配置 (服务器 + VLM API Key)            │
│  /mcp-monitor  │ 实时监控 (连接、工具调用、日志)             │
│  /mcp-tools    │ 工具测试                                    │
│  /extract      │ 文本提取                                    │
│  /search       │ 关键词搜索                                  │
│  /batch        │ 批量处理                                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 快速开始

### 方式一：下载二进制文件

从 [Releases](https://github.com/smile9493/rsut_pdf_mcp/releases) 下载对应平台的二进制文件：

| 平台 | 文件 | 架构 |
|------|------|------|
| Windows x64 | `pdf-mcp-windows-x64.zip` | x86_64 |
| Linux x64 | `pdf-mcp-linux-x64.tar.gz` | x86_64 |
| Linux ARM64 | `pdf-mcp-linux-arm64.tar.gz` | aarch64 |
| macOS x64 | `pdf-mcp-macos-x64.tar.gz` | x86_64 (Intel) |
| macOS ARM64 | `pdf-mcp-macos-arm64.tar.gz` | aarch64 (Apple Silicon) |

**安装步骤：**

```bash
# Linux/macOS
tar -xzf pdf-mcp-linux-x64.tar.gz
chmod +x pdf-mcp
sudo mv pdf-mcp /usr/local/bin/

# Windows
# 解压 zip 文件，将 pdf-mcp.exe 添加到 PATH
```

### 方式二：Docker 部署

```bash
# 拉取最新 MCP 服务器镜像
docker pull smile9493/pdf-mcp:latest-mcp

# 拉取最新 Web 前端镜像
docker pull smile9493/pdf-mcp:latest-web

# 运行 MCP 服务器 (stdio 模式)
docker run --rm -i smile9493/pdf-mcp:latest-mcp

# 运行 Web 前端
docker run -d -p 80:80 smile9493/pdf-mcp:latest-web
```

### 方式三：从源码构建

```bash
# 克隆仓库
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp

# 构建 MCP 服务器
cd pdf-module-rs
cargo build --release --bin pdf-mcp
# 二进制文件位于: target/release/pdf-mcp

# 构建 Web 前端
cd ../web
npm install
npm run build
# 静态文件位于: dist/
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

## Web Dashboard

### 本地开发

```bash
cd web
npm install
npm run dev
```

访问 http://localhost:5173

### Docker 部署

```bash
# 拉取并运行
docker pull smile9493/pdf-mcp:latest-web
docker run -d -p 80:80 smile9493/pdf-mcp:latest-web

# 自定义端口
docker run -d -p 3000:80 smile9493/pdf-mcp:latest-web
```

### 生产环境部署 (Nginx)

```bash
# 构建静态文件
cd web && npm run build

# 使用 Nginx 部署
docker run -d -p 80:80 \
  -v $(pwd)/dist:/usr/share/nginx/html \
  -v ./nginx.conf:/etc/nginx/nginx.conf \
  nginx:alpine
```

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
# 1. 确保代码已提交
git add . && git commit -m "准备发布 v1.0.0"

# 2. 创建版本标签
git tag v1.0.0

# 3. 推送代码和标签
git push origin main
git push origin v1.0.0
```

推送 `v*` 格式的标签后，CI 会自动：
1. 构建所有平台的二进制文件
2. 构建 Web 前端静态文件
3. 构建并推送 Docker 镜像
4. 创建 GitHub Release 并上传所有产物

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

### 示例调用

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "extract_text",
    "arguments": { "file_path": "/path/to/document.pdf" }
  }
}
```

---

## CI/CD 工作流

本项目使用 GitHub Actions 进行持续集成和发布。

### 触发条件

| 事件 | 触发任务 |
|------|---------|
| `push` 到 `main` | 测试、构建、Docker 推送 (需配置) |
| `push` 标签 `v*` | 测试、构建、创建 Release |
| `pull_request` | 测试、构建验证 |
| 手动触发 | 全部任务 |

### 工作流任务

```
test-rust → build-mcp (5平台) → docker-mcp → release
                ↓
            web-build → docker-web
```

### 构建产物

| 产物 | 说明 |
|------|------|
| `pdf-mcp-linux-x64.tar.gz` | Linux x64 二进制 |
| `pdf-mcp-linux-arm64.tar.gz` | Linux ARM64 二进制 |
| `pdf-mcp-macos-x64.tar.gz` | macOS Intel 二进制 |
| `pdf-mcp-macos-arm64.tar.gz` | macOS Apple Silicon 二进制 |
| `pdf-mcp-windows-x64.zip` | Windows x64 二进制 |
| `web-dist.tar.gz` | Web 前端静态文件 |
| `smile9493/pdf-mcp:*-mcp` | MCP 服务器 Docker 镜像 |
| `smile9493/pdf-mcp:*-web` | Web 前端 Docker 镜像 |

---

## 剔除清单

基于奥卡姆剃刀原则，以下冗余已被剔除：

| 剔除项 | 原因 |
|--------|------|
| REST API | MCP stdio 是最终契约 |
| Python SDK | 官方 MCP SDK 足矣 |
| 多引擎抽象 | pdfium 胜任所有场景 |
| 缓存模块 | 大模型自带 Prompt Caching |
| 熔断器 | 本地 I/O 无需网络熔断 |
| SSE 传输 | stdio 是 MCP 标准 |
| 智能路由 | 无路由 = 无分支预测惩罚 |

---

## 项目结构

```
pdf-module-rs/
├── crates/
│   ├── pdf-common/         # error + dto + config
│   ├── pdf-macros/         # 过程宏
│   ├── pdf-core/           # PdfiumEngine + FileValidator
│   ├── pdf-mcp/            # MCP stdio 入口
│   └── vlm-visual-gateway/ # VLM 条件升级

web/
├── src/
│   ├── views/
│   │   ├── McpConfigView.vue    # MCP 配置
│   │   ├── McpMonitorView.vue   # MCP 监控
│   │   └── McpToolsView.vue     # 工具测试
│   ├── stores/
│   │   └── mcpStore.ts          # MCP 状态管理
│   └── composables/
│       └── useAsyncAction.ts    # 统一异步处理

docker/
├── Dockerfile.mcp               # MCP 服务器镜像
└── Dockerfile.ci                # Web 前端镜像
```

---

## FFI 防波堤

所有 pdfium C++ 调用被 `catch_unwind` 包裹：

```rust
pub fn safe_extract_text(data: &[u8]) -> PdfResult<String> {
    catch_unwind(|| {
        // pdfium C++ 调用
    })
    .map_err(|_| PdfModuleError::Extraction("FFI panic".into()))?
}
```

C++ 崩溃无法越界污染 Rust 调用栈。

---

## License

[MIT](LICENSE)
