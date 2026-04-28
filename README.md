# PDF MCP Pipe

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)

**极简 PDF 提取 MCP 管道** — 单一 pdfium 引擎、纯 stdio 传输、零状态、零缓存。

基于**奥卡姆剃刀**与**截拳道**设计哲学，剔除所有非核心实体，收敛至最小可运行架构。

---

## 架构

```
大模型 (Cursor/Claude)
        │
        ▼ JSON-RPC over stdio
┌───────────────────────┐
│   pdf-mcp (入口)       │
│   ├─ server.rs        │  ← 仅 3 个 Tool
│   └─ main.rs          │  ← 无 CLI 参数，强制 stdio
└───────────────────────┘
        │
        ▼
┌───────────────────────┐
│   McpPdfPipeline      │
│   ├─ PdfiumEngine     │  ← FFI 防波堤 (catch_unwind)
│   └─ FileValidator    │  ← 路径安全 + 文件嗅探
└───────────────────────┘
        │
        ▼ std::fs::read (零 mmap、零缓存)
     PDF 文件
```

---

## 剔除清单

| 剔除项 | 原因 |
|--------|------|
| REST API | MCP 协议已是最终契约，旁路冗余 |
| Python SDK | 官方 `@modelcontextprotocol/sdk` 足矣 |
| lopdf / pdf-extract | 单一 pdfium 胜任所有场景 |
| Moka 缓存 | 大模型客户端自带 Prompt Caching |
| 熔断器 | 本地 I/O 无需网络级熔断 |
| Jieba 分词 | 分词是大模型的强项 |
| SSE 传输 | stdio 是 MCP 的标准方式 |
| 智能路由 | 无路由 = 无分支预测惩罚 |

---

## MCP 工具

仅 3 个核心工具：

| 工具 | 说明 |
|------|------|
| `extract_text` | 提取纯文本 |
| `extract_structured` | 提取结构化数据 (per-page + bbox) |
| `get_page_count` | 获取页数 |

---

## 快速开始

### 编译

```bash
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp/pdf-module-rs
cargo build --release
```

### 运行

```bash
./target/release/pdf-mcp
```

无命令行参数，无配置文件，直接启动 stdio 监听。

### Agent 集成

**Cursor** (`~/.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "pdf": {
      "command": "/path/to/pdf-mcp"
    }
  }
}
```

**Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "pdf": {
      "command": "/path/to/pdf-mcp"
    }
  }
}
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
    .map_err(|e| PdfModuleError::Extraction(format!("Pdfium: {}", e)))
}
```

C++ 崩溃无法越界污染 Rust 调用栈。

---

## 项目结构

```
pdf-module-rs/
├── Cargo.toml              # workspace: 3 个 crate
├── crates/
│   ├── pdf-common/         # error + dto + config + traits
│   ├── pdf-macros/         # 过程宏
│   ├── pdf-core/           # McpPdfPipeline → PdfiumEngine
│   └── pdf-mcp/            # 入口 (stdio JSON-RPC)
```

---

## License

[MIT](LICENSE)
