# PDF MCP Module

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/smile9493/rsut_pdf_mcp)](https://github.com/smile9493/rsut_pdf_mcp/releases)

> PDF 提取 MCP 服务端，为 AI Agent 提供 PDF 文档处理能力。采用纯正 Karpathy Wiki 架构，AI Agent 作为知识编译器。

## 特性

- 🔥 **零拷贝加载** - mmap 直接映射，无内存拷贝
- 🧠 **智能质量探测** - 自动识别扫描件，选择最佳提取方式
- 🤖 **Karpathy Wiki** - AI Agent 作为知识编译器，提炼原子化概念
- 🚀 **一键安装** - 预编译二进制，无需构建

## 安装

### 一键安装

```bash
curl -fsSL https://raw.githubusercontent.com/smile9493/rsut_pdf_mcp/main/install.sh | bash
```

安装完成后运行：

```bash
/opt/pdf-module/pdf-mcp-cli
```

### CLI 命令

```bash
pdf-mcp-cli              # 交互式菜单
pdf-mcp-cli config       # 配置 API Key
pdf-mcp-cli status       # 查看状态
pdf-mcp-cli start --web  # 启动服务
pdf-mcp-cli stop         # 停止服务
pdf-mcp-cli restart      # 重启服务
pdf-mcp-cli ps           # 查看进程
pdf-mcp-cli logs -f      # 查看日志
```

### Docker 部署

```yaml
services:
  pdf-mcp:
    image: smile9493/pdf-mcp:latest
    ports:
      - "8000:8000"
    volumes:
      - ./data:/app/data
      - ./wiki:/app/wiki
    environment:
      - PDFIUM_LIB_PATH=/app/lib/libpdfium.so
      - VLM_API_KEY=${VLM_API_KEY}
      - VLM_MODEL=glm-4v-flash
```

```bash
docker-compose up -d
```

### 从源码构建

```bash
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp/pdf-module-rs
cargo build --release --bin pdf-mcp
```

## MCP 工具

| 工具 | 说明 |
|------|------|
| `extract_text` | 提取 PDF 纯文本 |
| `extract_structured` | 提取结构化数据（每页文本 + bbox） |
| `get_page_count` | 获取 PDF 页数 |
| `search_keywords` | 搜索关键词 |
| `extrude_to_server_wiki` | 提取到服务端 Wiki（Karpathy 范式） |
| `extrude_to_agent_payload` | 返回 Markdown payload（供 AI Agent 处理） |

### extrude_to_server_wiki

提取 PDF 并保存到服务端 Wiki（纯正 Karpathy 范式）：

```json
{
  "name": "extrude_to_server_wiki",
  "arguments": {
    "file_path": "/path/to/document.pdf",
    "wiki_base_path": "/path/to/wiki"
  }
}
```

**说明**: Rust 引擎只负责将 PDF 提取到 `raw/` 目录，不预先创建词条。AI Agent 应阅读 `raw/` 内容，提炼核心概念，在 `wiki/` 目录创建原子化词条。

## Wiki 架构（Karpathy 范式）

```
wiki/
├── raw/                 # 输入池（原始提取）
│   └── 文档名.md
├── schema/              # 编译规则层
│   └── CLAUDE.md        # 知识编译器指令集
└── wiki/                # 输出池（AI Agent 编译）
    ├── index.md         # 动态索引
    ├── log.md           # 编译日志
    └── [领域] 概念.md   # 原子化词条
```

### 执行流程

| 阶段 | 执行者 | 操作 |
|------|--------|------|
| Phase 1 | Rust 引擎 | 提取 PDF → 放入 `raw/` |
| Phase 2 | AI Agent | 阅读内容，判断领域 |
| Phase 3 | AI Agent | 提炼 10-15 个核心概念 |
| Phase 4 | AI Agent | 创建词条（如 `[IT] Nginx_事件驱动模型.md`） |
| Phase 5 | AI Agent | 更新 `index.md` 和 `log.md` |

### 词条命名规范

不要按"第1章、第2章"命名，而是提炼原子化概念：
- `[IT] Nginx_多进程通信架构.md`
- `[IT] Nginx_事件驱动模型.md`
- `[IT] Nginx_Upstream负载均衡.md`

## Agent 配置

### Cursor

编辑 `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/opt/pdf-module/pdf-mcp",
      "env": {
        "PDFIUM_LIB_PATH": "/opt/pdf-module/lib/libpdfium.so",
        "VLM_API_KEY": "your-api-key",
        "VLM_MODEL": "glm-4v-flash"
      }
    }
  }
}
```

### Claude Desktop

编辑 `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/opt/pdf-module/pdf-mcp",
      "env": {
        "PDFIUM_LIB_PATH": "/opt/pdf-module/lib/libpdfium.so",
        "VLM_API_KEY": "your-api-key"
      }
    }
  }
}
```

## 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `PDFIUM_LIB_PATH` | PDFium 库路径 | 自动检测 |
| `VLM_API_KEY` | VLM API 密钥 | - |
| `VLM_MODEL` | 模型名称 | `glm-4v-flash` |
| `VLM_ENDPOINT` | API 端点 | `https://open.bigmodel.cn/api/paas/v4/chat/completions` |
| `DASHBOARD_PORT` | Dashboard 端口 | `8000` |

## 下载

GitHub Releases: [latest](https://github.com/smile9493/rsut_pdf_mcp/releases)

| 文件 | 平台 |
|------|------|
| `pdf-mcp-linux-x64.tar.gz` | Linux x86_64 |
| `pdf-mcp-linux-arm64.tar.gz` | Linux ARM64 |
| `pdf-mcp-macos-x64.tar.gz` | macOS Intel |
| `pdf-mcp-macos-arm64.tar.gz` | macOS Apple Silicon |
| `pdf-mcp-windows-x64.zip` | Windows x64 |

## 项目结构

```
pdf-module-rs/
├── crates/
│   ├── pdf-core/            # 核心引擎
│   ├── pdf-mcp/             # MCP 服务端
│   └── vlm-visual-gateway/  # VLM 网关
├── pdf-mcp-installer/       # CLI 工具
└── web/                     # Dashboard 前端
```

## License

[MIT](LICENSE)
