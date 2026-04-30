# PDF MCP Module

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/smile9493/rsut_pdf_mcp)](https://github.com/smile9493/rsut_pdf_mcp/releases)

PDF 提取 MCP 服务端，为 AI Agent 提供 PDF 文档处理能力。

## 特性

- **零拷贝加载**: mmap 直接映射，无内存拷贝
- **智能质量探测**: 自动识别扫描件，选择最佳提取方式
- **VLM 增强**: 扫描件自动调用视觉语言模型
- **Wiki 自动化**: 三级存储结构，自动索引生成
- **双模态工具**: 服务端构建 + 本地投影

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

## 架构

```
┌─────────────────────────────────────────┐
│         AI Agent (Cursor/Claude)         │
└──────────────────┬──────────────────────┘
                   │ JSON-RPC over stdio
                   ▼
┌─────────────────────────────────────────┐
│              pdf-mcp                     │
├─────────────────────────────────────────┤
│  MCP Tools:                              │
│  • extract_text           提取纯文本      │
│  • extract_structured     提取结构化数据  │
│  • get_page_count         获取页数        │
│  • search_keywords        关键词搜索      │
│  • extrude_to_server_wiki 服务端存储      │
│  • extrude_to_agent_payload 本地投影      │
└─────────────────────────────────────────┘
                   │
       ┌───────────┴───────────┐
       ▼                       ▼
┌─────────────┐         ┌─────────────┐
│ PdfiumEngine│         │ VlmGateway  │
│ 本地提取     │         │ 扫描件增强   │
└─────────────┘         └─────────────┘
```

## 质量探测

系统自动分析 PDF 并选择最佳提取方式：

| 类型 | 条件 | 方式 |
|------|------|------|
| Digital | 文本密度 > 0.3 | Pdfium 本地提取 |
| Scanned | 无字体，有图像 | VLM 多模态增强 |
| LowQuality | 文本密度 < 0.1 | 混合模式 |

## Wiki 存储

```
wiki/
├── raw/           # 原始提取 (带 YAML 元数据)
├── wiki/          # 精炼页面
├── scheme/        # 类型约束
└── MAP.md         # 自动索引
```

## Agent 配置

### Cursor

编辑 `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/opt/pdf-module/pdf-mcp",
      "env": {
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
        "VLM_API_KEY": "your-api-key"
      }
    }
  }
}
```

## 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `VLM_API_KEY` | VLM API 密钥 | - |
| `VLM_MODEL` | 模型名称 | `glm-4v-flash` |
| `VLM_ENDPOINT` | API 端点 | `https://open.bigmodel.cn/api/paas/v4/chat/completions` |
| `DASHBOARD_PORT` | Dashboard 端口 | `8000` |
| `DASHBOARD_WEB_DIR` | 前端目录 | `./web/dist` |

## 下载

GitHub Releases: [latest](https://github.com/smile9493/rsut_pdf_mcp/releases)

| 文件 | 平台 |
|------|------|
| `pdf-mcp-linux-x64.tar.gz` | Linux x86_64 |
| `pdf-mcp-linux-arm64.tar.gz` | Linux ARM64 |
| `pdf-mcp-macos-x64.tar.gz` | macOS Intel |
| `pdf-mcp-macos-arm64.tar.gz` | macOS Apple Silicon |
| `pdf-mcp-windows-x64.zip` | Windows x64 |

每个包包含 `pdf-mcp` 和 `pdf-mcp-cli`。

## Docker

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
      - VLM_API_KEY=${VLM_API_KEY}
      - VLM_MODEL=glm-4v-flash
```

## 从源码构建

```bash
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp/pdf-module-rs
cargo build --release --bin pdf-mcp
```

## 项目结构

```
pdf-module-rs/
├── crates/
│   ├── pdf-core/         # 核心引擎
│   ├── pdf-mcp/          # MCP 服务端
│   └── vlm-visual-gateway/  # VLM 网关
├── pdf-mcp-installer/    # CLI 工具
└── web/                  # Dashboard 前端
```

## License

[MIT](LICENSE)
