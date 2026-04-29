# PDF MCP Architecture

基于**奥卡姆剃刀**与**截拳道**设计哲学的最小可运行架构。

## 设计原则

1. **最小化**: 只保留核心实体，剔除所有旁路
2. **零状态**: 无缓存、无熔断、无路由状态
3. **FFI 安全**: `catch_unwind` 防波堤隔离 C++ 崩溃
4. **单一引擎**: pdfium 胜任所有场景

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                  AI Agent (Cursor/Claude Desktop)           │
└──────────────────────────┬──────────────────────────────────┘
                           │ JSON-RPC over stdio
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     pdf-mcp (入口)                          │
│  main.rs ──► server.rs ──► handle_request()                │
├─────────────────────────────────────────────────────────────┤
│  MCP Tools (3 个):                                          │
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
│                   │              │                   │
│  catch_unwind {   │              │  检测条件:        │
│    pdfium C++     │              │  • 扫描件         │
│  }                │              │  • 混沌布局       │
│                   │              │                   │
│  FFI 防波堤       │              │  模型:            │
│  C++ 崩溃不传播   │              │  • GPT-4o         │
└───────────────────┘              │  • Claude 3.5     │
                                   └───────────────────┘
```

## Crates

| Crate | 职责 |
|-------|------|
| `pdf-common` | 统一错误、DTO、配置、traits |
| `pdf-macros` | 过程宏 `#[derive(Builder)]` |
| `pdf-core` | PdfiumEngine + FileValidator + VlmPipeline |
| `pdf-mcp` | MCP stdio 入口 (JSON-RPC) |
| `vlm-visual-gateway` | VLM 条件升级网关 |

## FFI 防波堤

```rust
pub fn safe_extract_text(data: &[u8]) -> PdfResult<String> {
    catch_unwind(AssertUnwindSafe(|| {
        // pdfium C++ 调用
    }))
    .map_err(|_| PdfModuleError::Extraction("FFI panic".into()))?
    .map_err(|e| PdfModuleError::Extraction(format!("Pdfium: {}", e)))
}
```

C++ 崩溃无法越界污染 Rust 调用栈。

## VLM 条件升级

```rust
pub struct VlmConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: VlmModel,        // Gpt4o / Claude35Sonnet
    pub timeout: Duration,
    pub max_concurrency: usize,
}
```

环境变量:
- `VLM_API_KEY`: API 密钥
- `VLM_ENDPOINT`: API 端点
- `VLM_MODEL`: `gpt-4o` / `claude-3.5-sonnet`

## 剔除清单

| 剔除项 | 原因 |
|--------|------|
| REST API | MCP stdio 是最终契约 |
| Python SDK | 官方 MCP SDK 足矣 |
| 多引擎 | pdfium 胜任所有 |
| 缓存 | 大模型自带 Prompt Caching |
| 熔断器 | 本地 I/O 无需网络熔断 |
| SSE | stdio 是 MCP 标准 |
| 智能路由 | 无路由 = 无分支惩罚 |
| 插件系统 | 过度工程化 |
| 审计日志 | 简化为日志输出 |
| 存储抽象 | 无状态设计 |
