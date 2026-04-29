# PDF MCP Module

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Vue](https://img.shields.io/badge/Vue-3.5%2B-green.svg)](https://vuejs.org/)

**极简 PDF 提取 MCP 管道** — 单一 pdfium 引擎、纯 stdio 传输、VLM 条件升级、Web 监控面板。

基于**奥卡姆剃刀**与**截拳道**设计哲学，剔除所有非核心实体，收敛至最小可运行架构。

---

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                  AI Agent (Cursor/Claude Desktop)           │
└──────────────────────────┬──────────────────────────────────┘
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
        ┌──────────────────┴──────────────────┐
        ▼                                      ▼
┌───────────────────┐              ┌───────────────────┐
│   PdfiumEngine    │              │   VlmGateway      │
│   (本地提取)       │              │   (条件升级)       │
│   FFI 防波堤       │              │   GPT-4o/Claude   │
└───────────────────┘              └───────────────────┘

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

### 1. 下载二进制

从 [Releases](https://github.com/smile9493/rsut_pdf_mcp/releases) 下载：

| 平台 | 文件 |
|------|------|
| Windows x64 | `pdf-mcp-windows-x64.zip` |
| Linux x64 | `pdf-mcp-linux-x64.tar.gz` |
| macOS x64 | `pdf-mcp-macos-x64.tar.gz` |
| macOS ARM64 | `pdf-mcp-macos-arm64.tar.gz` |

### 2. Agent 集成

**Cursor** (`~/.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/path/to/pdf-mcp"
    }
  }
}
```

**Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/path/to/pdf-mcp"
    }
  }
}
```

### 3. VLM 配置 (可选)

如需 VLM 视觉增强 (扫描件、混沌布局)：

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/path/to/pdf-mcp",
      "env": {
        "VLM_API_KEY": "sk-xxx",
        "VLM_ENDPOINT": "https://api.openai.com/v1/chat/completions",
        "VLM_MODEL": "gpt-4o"
      }
    }
  }
}
```

| 环境变量 | 说明 |
|---------|------|
| `VLM_API_KEY` | OpenAI/Anthropic API Key |
| `VLM_ENDPOINT` | API 端点 |
| `VLM_MODEL` | `gpt-4o` 或 `claude-3.5-sonnet` |

---

## Web Dashboard

### 本地运行

```bash
cd web
npm install
npm run dev
```

访问 http://localhost:5173

### Docker 部署

```bash
docker pull smile9493/pdf-mcp:latest-web
docker run -p 80:80 smile9493/pdf-mcp:latest-web
```

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
│   ├── pdf-common/     # error + dto + config
│   ├── pdf-macros/     # 过程宏
│   ├── pdf-core/       # PdfiumEngine + FileValidator
│   ├── pdf-mcp/        # MCP stdio 入口
│   └── vlm-visual-gateway/  # VLM 条件升级

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
