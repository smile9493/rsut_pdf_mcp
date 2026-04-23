# 依赖说明

本文档详细说明 PDF Module MCP Server 的所有依赖项及其用途。

---

## 核心依赖

### 序列化

| 依赖 | 版本 | 用途 |
|------|------|------|
| `serde` | 1.0 | 序列化框架，支持 JSON/TOML/YAML |
| `serde_json` | 1.0 | JSON 序列化/反序列化 |

### 异步运行时

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tokio` | 1.x | 异步运行时，支持全特性 |
| `futures` | 0.3 | Future trait 和工具 |
| `async-trait` | 0.1 | 异步 trait 支持 |

### 日志

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tracing` | 0.1 | 结构化日志和追踪 |
| `tracing-subscriber` | 0.3 | 日志订阅者，支持 JSON 格式 |
| `tracing-config` | 0.2 | 日志配置 |

### 错误处理

| 依赖 | 版本 | 用途 |
|------|------|------|
| `thiserror` | 2.0 | 派生错误类型 |
| `anyhow` | 1.0 | 通用错误处理 |

---

## PDF 引擎

| 依赖 | 版本 | 用途 |
|------|------|------|
| `lopdf` | 0.34 | PDF 文档解析，布局感知 |
| `pdf-extract` | 0.7 | 快速文本提取 |
| `pdfium-render` | 0.8 | Chrome PDFium 引擎绑定，高兼容性 |

---

## 缓存与性能

| 依赖 | 版本 | 用途 |
|------|------|------|
| `moka` | 0.12 | 高性能并发缓存，支持 TTL |
| `sha2` | 0.10 | SHA-256 哈希计算 |
| `regex` | 1.10 | 正则表达式匹配 |
| `jieba-rs` | 0.7 | 中文分词 |
| `once_cell` | 1.19 | 延迟初始化，全局单例 |

---

## 配置管理

| 依赖 | 版本 | 用途 |
|------|------|------|
| `dotenvy` | 0.15 | .env 文件加载 |
| `config` | 0.15 | 多格式配置加载 (TOML/JSON/YAML) |
| `toml` | 0.8 | TOML 解析 |
| `serde_yaml` | 0.9 | YAML 解析 |

---

## Web 框架

| 依赖 | 版本 | 用途 |
|------|------|------|
| `axum` | 0.8 | 异步 Web 框架，支持 multipart |
| `tower-http` | 0.6 | HTTP 中间件 (CORS, Trace) |
| `utoipa` | 5.0 | OpenAPI 文档生成 |
| `utoipa-swagger-ui` | 8.0 | Swagger UI 界面 |

---

## MCP 协议

| 依赖 | 版本 | 用途 |
|------|------|------|
| `rust-mcp-sdk` | 0.9 | MCP 协议 SDK，支持 stdio/SSE |
| `json-rpc-rs` | 0.3 | JSON-RPC 2.0 实现 |
| `sse` | 0.2 | Server-Sent Events |

---

## CLI

| 依赖 | 版本 | 用途 |
|------|------|------|
| `clap` | 4.4 | 命令行参数解析，支持 derive |

---

## 可观测性

| 依赖 | 版本 | 用途 |
|------|------|------|
| `metrics` | 0.24 | 指标收集接口 |
| `metrics-exporter-prometheus` | 0.16 | Prometheus 指标导出 |
| `opentelemetry` | 0.31 | OpenTelemetry 追踪和指标 |

---

## 文件处理

| 依赖 | 版本 | 用途 |
|------|------|------|
| `infer` | 0.16 | 文件类型嗅探 (MIME 检测) |
| `walkdir` | 2.4 | 递归目录遍历 |

---

## 时间与标识

| 依赖 | 版本 | 用途 |
|------|------|------|
| `chrono` | 0.4 | 日期时间处理 |
| `uuid` | 1.6 | UUID 生成 |

---

## 云存储 (可选)

| 依赖 | 版本 | 用途 |
|------|------|------|
| `aws-sdk-s3` | 1.0 | AWS S3 存储 |
| `google-cloud-storage` | 1.11 | Google Cloud Storage |
| `azure_storage_blobs` | 0.21 | Azure Blob Storage |
| `remotefs-aws-s3` | 0.4 | 远程文件系统抽象 |

---

## 插件系统 (可选)

| 依赖 | 版本 | 用途 |
|------|------|------|
| `mcp-kit` | 0.4 | MCP 工具包，支持热重载 |
| `libloading` | 0.9 | 动态库加载 |
| `wasmtime` | 44 | WebAssembly 运行时 |

---

## Python 绑定

| 依赖 | 版本 | 用途 |
|------|------|------|
| `pyo3` | 0.22 | Python FFI 绑定 |
| `pyo3-async-runtimes` | 0.22 | PyO3 异步支持 |

---

## 依赖关系图

```
pdf-module-rs
├── pdf-core (核心库)
│   ├── lopdf, pdf-extract, pdfium-render (PDF引擎)
│   ├── moka (缓存)
│   ├── jieba-rs, regex (文本处理)
│   ├── serde, serde_json (序列化)
│   ├── tokio (异步)
│   └── tracing (日志)
│
├── pdf-mcp (MCP Server)
│   ├── pdf-core
│   ├── rust-mcp-sdk (MCP协议)
│   ├── axum (HTTP/SSE)
│   └── clap (CLI)
│
├── pdf-rest (REST API)
│   ├── pdf-core
│   ├── axum, tower-http (Web)
│   ├── utoipa (OpenAPI)
│   └── clap (CLI)
│
└── pdf-python (Python绑定)
    ├── pdf-core
    └── pyo3 (FFI)
```

---

## 版本要求

| 组件 | 最低版本 |
|------|----------|
| Rust | 1.83 |
| Cargo | 1.83 |
| Python | 3.8+ (SDK) |
| Docker | 20.10+ |

---

## 安全审计

所有依赖均通过 `cargo audit` 检查，无已知安全漏洞。

运行安全审计：
```bash
cargo install cargo-audit
cargo audit
```

---

## 更新依赖

检查可更新依赖：
```bash
cargo outdated
```

更新依赖：
```bash
cargo update
```

更新特定依赖：
```bash
cargo update -p serde
```
