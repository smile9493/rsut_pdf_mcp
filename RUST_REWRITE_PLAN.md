# pdf-module Rust 重写方案

## 1. 方案概述

将现有 Python (Flask + MCP) PDF 处理系统完整重写为 Rust 原生实现。核心目标：

- **性能提升**：PDF 解析从 Python GIL 限制中解放，实现真正的多线程并行提取，单页解析延迟目标 <1ms
- **内存优化**：零拷贝流式处理大文件，消除 Python 运行时开销，容器镜像从 ~200MB 缩减至 ~30MB
- **协议兼容**：完整保留 MCP (2024-11-05) 协议和 REST API 接口，实现无缝迁移
- **架构延续**：保持现有适配器模式（Strategy Pattern）分层架构，核心逻辑与传输协议解耦

## 2. 现有系统分析

### 2.1 当前架构

```
Client (AI/HTTP)
    ├── MCP Server (stdio JSON-RPC)  ──┐
    └── REST API (Flask + Gunicorn)  ──┤
                                        ▼
                              PDFExtractor (编排器)
                                  ├── X2TextAdapter (Trait)
                                  │   ├── PDFPlumberAdapter
                                  │   └── PyMuPDFAdapter
                                  ├── FileValidator
                                  ├── ExtractionCache (LRU/SHA256/TTL)
                                  └── KeywordExtractor (搜索/词频/TF-IDF)
```

### 2.2 现有模块清单与代码量

| 模块 | 文件 | 核心职责 | 行数 |
|------|------|----------|------|
| pdf_extractor | `pdf_extractor.py` | 适配器调度、统一 API 入口 | 163 |
| adapters/base | `adapters/base.py` | 抽象基类定义 | 68 |
| adapters/pymupdf | `adapters/pymupdf_adapter.py` | PyMuPDF 引擎封装 | 232 |
| adapters/pdfplumber | `adapters/pdfplumber_adapter.py` | pdfplumber 引擎封装 | ~120 |
| keyword_extractor | `keyword_extractor.py` | 关键字搜索/词频/TF-IDF/高亮 | 317 |
| cache | `cache.py` | LRU 缓存 (SHA256 + TTL) | 169 |
| validators | `validators.py` | 文件验证 (路径/扩展名/大小/魔数) | 113 |
| dto | `dto.py` | 数据传输对象 | 118 |
| exceptions | `exceptions.py` | 异常层次结构 | 68 |
| config | `config.py` | 环境变量配置 + 日志 | 70 |
| mcp_server | `mcp_server.py` | MCP 协议服务 (stdio) | 323 |
| controllers | `controllers.py` | REST API 路由 | 223 |

### 2.3 现有 API 接口

**MCP 工具 (7个)**：`extract_text`, `extract_structured`, `get_page_count`, `search_keywords`, `extract_keywords`, `list_adapters`, `cache_stats`

**REST 端点 (6个)**：
- `GET /api/v1/x2text/health`
- `POST /api/v1/x2text/extract` (返回文本文件)
- `POST /api/v1/x2text/extract-json` (返回 JSON)
- `POST /api/v1/x2text/info` (页数信息)
- `GET /api/v1/x2text/adapters`
- `GET /api/v1/x2text/cache/stats`

## 3. 技术栈选型

### 3.1 PDF 引擎（多引擎策略模式）

| 引擎角色 | Rust Crate | 版本 | 选型理由 | 替代 Python |
|----------|-----------|------|----------|-------------|
| **高性能提取引擎** | `pdf-extract` | 0.10+ | 纯 Rust，极致解析速度，适合大规模文本提取 | pdfplumber (纯文本提取) |
| **布局感知引擎** | `lopdf` | 0.40+ | 可解析 PDF Object Tree，获取页面坐标/字体/布局信息，支持结构化提取。**注意：lopdf 是底层库，获取 bbox/LineInfo 需手动解析 Text Matrix (Tm) 操作符，需在 P1 阶段完成 PoC 验证** | PyMuPDF `get_text("dict")` 布局模式 |
| **高兼容性引擎** | `pdfium-render` | 0.8+ | 绑定 PDFium (Chrome 渲染引擎)，工业级稳定性，处理复杂排版/老旧文档 | PyMuPDF 处理非标文件 |
| **底层操作引擎** | `lopdf` (复用) | 0.40+ | 直接操作 PDF Object Tree，文档修补/元数据修改 | PyMuPDF 底层操作 |

**引擎调度策略**：
- 默认使用 `lopdf`（布局感知，功能最全面，对应现有 PyMuPDF 的角色）
- `pdf-extract` 作为快速纯文本提取引擎（对应现有 pdfplumber 的角色）
- `pdfium-render` 作为降级兜底引擎（处理 lopdf 解析失败的复杂文档）
- **智能路由（P4 阶段实现）**：系统具备"嗅探后分发"能力，先快速读取 PDF 元数据或前几页特征，根据文档复杂度自动选择引擎：
  - 纯文本流 / 少量页面 → `pdf-extract`（最快）
  - 标准排版 / 需要布局信息 → `lopdf`（默认）
  - 复杂排版 / 特殊编码 / 非标结构 → `pdfium-render`（最兼容）
- **引擎熔断（P4 阶段实现）**：主引擎解析失败时自动降级至兜底引擎，确保对 AI Agent 层的接口高可用率

### 3.2 Web / 协议层

| 组件 | Rust Crate | 版本 | 用途 | 替代 Python |
|------|-----------|------|------|-------------|
| HTTP 框架 | `axum` | 0.8+ | REST API 服务，基于 hyper/tower，异步高性能 | Flask |
| MCP 协议 | `rmcp` | 0.1+ | Model Context Protocol 的 Rust 实现，支持 stdio/SSE 传输 | 手写 MCP JSON-RPC |
| JSON-RPC | `jsonrpsee` | 0.24+ | 备选：如 rmcp 不满足需求时的 JSON-RPC 框架 | 手写 JSON-RPC |
| 异步运行时 | `tokio` | 1.52+ | 异步 I/O 运行时 | asyncio (隐式) |

### 3.3 基础设施

| 组件 | Rust Crate | 版本 | 用途 | 替代 Python |
|------|-----------|------|------|-------------|
| 序列化 | `serde` + `serde_json` | 1.0+ | JSON 序列化/反序列化 | dataclass + json |
| 缓存 | `moka` | 0.12+ | 并发安全的高性能缓存 (类似 Java Caffeine)，支持 TTL | OrderedDict LRU |
| 哈希 | `sha2` | 0.11+ | SHA256 文件哈希 | hashlib.sha256 |
| 正则 | `regex` | 1.10+ | 关键字搜索与匹配 | re |
| 中文分词 | `jieba-rs` | 0.7+ | 中文分词（替代简单正则分词）。注意：jieba-rs 内部字典加载会占用一定常驻内存（~10MB），如需极致轻量可后期替换为基于 Double-Array Trie 的分词方案 | 正则提取中文字符 |
| 日志 | `tracing` + `tracing-subscriber` | 0.1+ | 结构化日志，支持文件轮转 | logging + RotatingFileHandler |
| 配置 | `dotenvy` + `serde` | 0.15+ | 环境变量加载 | python-dotenv |
| 错误处理 | `thiserror` + `anyhow` | 2.0+ / 1.0+ | 派生 Error trait / 应用层错误 | 自定义异常层次 |
| 文件嗅探 | `infer` | 0.16+ | 基于 Magic Numbers 的深度文件类型探测，阻断伪装上传 | 无（仅 `%PDF` 头检查） |
| 可观测性 | `metrics` + `metrics-exporter-prometheus` | 0.24+ / 0.16+ | Prometheus 指标导出，监控引擎延迟/缓存命中率/路由分布 | 无 |
| 命令行 | `clap` | 4.4+ | CLI 参数解析（选择 MCP/REST 模式） | 环境变量 |
| OpenAPI | `utoipa` | 5.0+ | 自动生成 OpenAPI 文档 | 无 |
| 跨语言绑定 | `pyo3` | 0.22+ | 可选：暴露 Python SDK | - |

## 4. 系统架构设计

### 4.1 整体分层架构

```
┌─────────────────────────────────────────────────────────┐
│                   Interface Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  MCP Server  │  │  REST API    │  │  Python SDK  │  │
│  │  (stdio/SSE) │  │  (axum)      │  │  (PyO3,可选) │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
├─────────┼──────────────────┼──────────────────┼─────────┤
│         │      Service Layer (编排)            │         │
│  ┌──────┴──────────────────┴──────────────────┴───────┐ │
│  │              PdfExtractorService                    │ │
│  │  - extract_text() / extract_structured()           │ │
│  │  - get_page_count() / search_keywords()            │ │
│  │  - extract_keywords() / list_adapters()            │ │
│  └──────────┬────────────────────────┬────────────────┘ │
├─────────────┼────────────────────────┼──────────────────┤
│             │   Core Layer (引擎)    │                  │
│  ┌──────────┴──────┐  ┌─────────────┴──────────────┐  │
│  │  PdfEngine Trait │  │  KeywordExtractor          │  │
│  │  ┌─────────────┐│  │  - search_keywords()       │  │
│  │  │ LopdfEngine ││  │  - extract_by_frequency()  │  │
│  │  │ PdfExtract  ││  │  - extract_tfidf()         │  │
│  │  │ PdfiumEngine││  │  - highlight()             │  │
│  │  └─────────────┘│  └────────────────────────────┘  │
├─────────────────────┼──────────────────────────────────┤
│  Infrastructure      │                                  │
│  ┌──────────┐ ┌─────┴─────┐ ┌──────────┐ ┌──────────┐ │
│  │  Cache   │ │ Validator │ │  Config   │ │  Error   │ │
│  │  (moka)  │ │           │ │          │ │  Types   │ │
│  └──────────┘ └───────────┘ └──────────┘ └──────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Cargo Workspace 结构

```
pdf-module-rs/
├── Cargo.toml                    # Workspace 根配置
├── .env.example
├── Dockerfile
├── docker-compose.yml
├── crates/
│   ├── pdf-core/                 # 核心库：引擎、提取器、缓存、验证
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine/           # PDF 引擎抽象与实现
│   │       │   ├── mod.rs
│   │       │   ├── trait.rs      # PdfEngine trait 定义
│   │       │   ├── lopdf.rs      # lopdf 引擎实现
│   │       │   ├── pdf_extract.rs # pdf-extract 引擎实现
│   │       │   └── pdfium.rs     # pdfium-render 引擎实现
│   │       ├── extractor.rs      # PdfExtractorService 编排器
│   │       ├── keyword.rs        # KeywordExtractor
│   │       ├── cache.rs          # ExtractionCache (moka)
│   │       ├── validator.rs      # FileValidator
│   │       ├── dto.rs            # 数据传输对象 (serde)
│   │       ├── error.rs          # 错误类型 (thiserror)
│   │       ├── config.rs         # ServerConfig
│   │       └── metrics.rs        # Prometheus 指标定义与记录
│   ├── pdf-mcp/                  # MCP 协议服务
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── server.rs         # MCP 工具注册与分发
│   ├── pdf-rest/                 # REST API 服务
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes.rs         # API 路由定义
│   │       └── app.rs            # axum App 构建
│   └── pdf-python/               # 可选：PyO3 Python 绑定
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
└── tests/                        # 集成测试
    └── integration/
```

### 4.3 Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/pdf-core",
    "crates/pdf-mcp",
    "crates/pdf-rest",
    "crates/pdf-python",
]

[workspace.dependencies]
# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 异步
tokio = { version = "1", features = ["full"] }
futures = "0.3"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# 错误
thiserror = "2.0"
anyhow = "1.0"

# PDF 引擎
lopdf = "0.40"
pdf-extract = "0.10"
pdfium-render = { version = "0.8", features = ["sync"] }

# 缓存
moka = { version = "0.12", features = ["sync"] }

# 哈希
sha2 = "0.11"

# 正则 & 分词
regex = "1.10"
jieba-rs = "0.7"

# 配置
dotenvy = "0.15"

# Web (REST)
axum = { version = "0.8", features = ["multipart"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
utoipa = "5.0"
utoipa-swagger-ui = "8.0"

# MCP
rmcp = { version = "0.1", features = ["server"] }

# CLI
clap = { version = "4.4", features = ["derive", "env"] }

# 可观测性
metrics = "0.24"
metrics-exporter-prometheus = "0.16"
```

## 5. 核心模块设计

### 5.1 PdfEngine Trait（对应 `adapters/base.py`）

```rust
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use crate::{dto::*, error::PdfResult};

/// PDF 提取引擎统一接口
/// 对应 Python: X2TextAdapter (adapters/base.py)
#[async_trait]
pub trait PdfEngine: Send + Sync {
    /// 引擎唯一标识
    fn id(&self) -> &str;
    /// 引擎显示名称
    fn name(&self) -> &str;
    /// 引擎描述
    fn description(&self) -> &str;

    /// 提取纯文本
    /// 对应 Python: X2TextAdapter.process()
    async fn extract_text(&self, file_path: &Path) -> PdfResult<TextExtractionResult>;

    /// 提取结构化数据（分页、行、坐标）
    /// 对应 Python: PyMuPDFAdapter.process_structured()
    async fn extract_structured(
        &self,
        file_path: &Path,
        options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult>;

    /// 获取页数
    /// 对应 Python: PyMuPDFAdapter.get_page_count()
    async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32>;

    /// 流式逐页提取（针对超大 PDF 的内存优化）
    /// 返回一个异步迭代器，按页读取、按页吐出结果，
    /// 避免在内存中构建包含整个 PDF 内容的巨大结构体。
    /// 利用 Rust 的 async 特性，将内存峰值压至单页级别。
    async fn extract_page_stream(
        &self,
        file_path: &Path,
        options: &ExtractOptions,
    ) -> PdfResult<Pin<Box<dyn Stream<Item = PdfResult<PageMetadata>> + Send>>>;

    /// 测试引擎可用性
    /// 对应 Python: X2TextAdapter.test_connection()
    fn test_connection(&self) -> bool {
        false
    }
}

/// 提取选项
#[derive(Debug, Default, serde::Deserialize)]
pub struct ExtractOptions {
    pub enable_highlight: bool,
    pub adapter: Option<String>,
}
```

### 5.2 LopdfEngine 实现（对应 `adapters/pymupdf_adapter.py`）

> **PoC 前置要求**：lopdf 是极其底层的库，不像 PyMuPDF 那样提供 `get_text("dict")` 开箱即用的布局信息。要获取带有坐标的结构化数据（bbox、LineInfo），需要手动解析 PDF 页面内容流中的文本矩阵操作符（`Tm`、`Tj`、`TJ`、`Td`、`TD`）并计算字体缩放系数。**在正式开发前，必须先针对这部分核心逻辑完成一个独立的 PoC**，验证：
> 1. 从 lopdf 的 `Stream` 对象中解码页面内容流（需处理 FlateDecode 等过滤器）
> 2. 解析 PDF 操作符序列，提取 `Tm`（设置文本矩阵）、`Tj`/`TJ`（显示文本）操作符
> 3. 根据 Current Text Matrix 和字体字典中的 `/Widths` 数组计算每个字符/文本块的物理坐标 (x, y, width, height)
> 4. 将坐标信息组装为 `LineInfo`（bbox + text），与 PyMuPDF 的输出进行对比验证
>
> 如果 PoC 发现 lopdf 的 Tm 解析工作量过大或精度不足，可考虑：
> - 降级方案：LopdfEngine 的 `extract_structured()` 仅返回页面级 bbox（来自 MediaBox），行级信息留空，由 pdfium-render 引擎补充
> - 替代方案：使用 `pdf-extract` crate（内部已实现部分布局解析）或直接依赖 pdfium-render 作为布局感知引擎

```rust
use lopdf::Document;
use crate::{dto::*, error::*, engine::PdfEngine};

/// 基于 lopdf 的布局感知引擎
/// 对应 Python: PyMuPDFAdapter
pub struct LopdfEngine;

#[async_trait]
impl PdfEngine for LopdfEngine {
    fn id(&self) -> &str { "lopdf" }
    fn name(&self) -> &str { "LopdfEngine" }
    fn description(&self) -> &str { "Layout-aware PDF engine based on lopdf" }

    async fn extract_text(&self, file_path: &Path) -> PdfResult<TextExtractionResult> {
        let doc = Document::load(file_path)
            .map_err(|e| ExtractionError::new(format!("Failed to load PDF: {}", e)))?;

        let mut all_text = String::new();
        let page_count = doc.get_pages().len();

        for (_, page_id) in doc.get_pages() {
            if let Ok(text) = doc.extract_text(&[page_id]) {
                all_text.push_str(&text);
                all_text.push('\n');
            }
        }

        Ok(TextExtractionResult {
            extracted_text: all_text.trim().to_string(),
            extraction_metadata: None,
        })
    }

    async fn extract_structured(
        &self,
        file_path: &Path,
        options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult> {
        let doc = Document::load(file_path)
            .map_err(|e| ExtractionError::new(format!("Failed to load PDF: {}", e)))?;

        let pages_map = doc.get_pages();
        let page_count = pages_map.len() as u32;
        let mut pages = Vec::with_capacity(page_count as usize);
        let mut all_text = String::new();

        for (page_num, page_id) in pages_map.iter() {
            let page_text = doc.extract_text(&[*page_id])
                .unwrap_or_default();

            // lopdf 可获取页面 MediaBox 作为 bbox
            let bbox = doc.get_page_bbox(page_id).ok();

            let page_meta = PageMetadata {
                page_number: *page_num as u32,
                text: page_text.trim().to_string(),
                bbox,
                lines: vec![], // lopdf 行级信息需进一步解析
            };
            all_text.push_str(&page_text);
            all_text.push('\n');
            pages.push(page_meta);
        }

        let file_info = FileInfo::from_path(file_path)?;

        Ok(StructuredExtractionResult {
            extracted_text: all_text.trim().to_string(),
            page_count,
            pages,
            extraction_metadata: None,
            file_info,
        })
    }

    async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        let doc = Document::load(file_path)
            .map_err(|e| ExtractionError::new(format!("Failed to load PDF: {}", e)))?;
        Ok(doc.get_pages().len() as u32)
    }
}
```

### 5.3 PdfExtractorService（对应 `pdf_extractor.py`）

```rust
use std::collections::HashMap;
use std::sync::Arc;
use crate::{engine::*, dto::*, error::*, cache::*, validator::*, keyword::*};

/// PDF 提取服务编排器
/// 对应 Python: PDFExtractor (pdf_extractor.py)
/// 增强特性：智能路由 + 引擎熔断降级
pub struct PdfExtractorService {
    engines: HashMap<String, Arc<dyn PdfEngine>>,
    default_engine: String,
    fallback_engine: String,  // 熔断降级目标引擎
    validator: FileValidator,
    cache: Option<ExtractionCache>,
    keyword_extractor: KeywordExtractor,
    router: SmartRouter,       // 智能路由器
    circuit_breaker: CircuitBreaker,  // 熔断器
}

impl PdfExtractorService {
    pub fn new(config: &ServerConfig) -> PdfResult<Self> {
        let mut engines: HashMap<String, Arc<dyn PdfEngine>> = HashMap::new();

        // 注册引擎（对应 Python AVAILABLE_ADAPTERS）
        let lopdf = Arc::new(LopdfEngine);
        engines.insert("lopdf".into(), lopdf);
        engines.insert("pymupdf".into(), Arc::new(LopdfEngine)); // 兼容别名

        let pdf_extract = Arc::new(PdfExtractEngine);
        engines.insert("pdf-extract".into(), pdf_extract);
        engines.insert("pdfplumber".into(), Arc::new(PdfExtractEngine)); // 兼容别名

        let pdfium = Arc::new(PdfiumEngine::new()?);
        engines.insert("pdfium".into(), pdfium);

        let cache = if config.cache_enabled {
            Some(ExtractionCache::new(config.cache_max_size, config.cache_ttl_seconds))
        } else {
            None
        };

        Ok(Self {
            engines,
            default_engine: config.default_adapter.clone(),
            fallback_engine: "pdfium".to_string(),  // 熔断降级目标
            validator: FileValidator::new(config.max_file_size_mb),
            cache,
            keyword_extractor: KeywordExtractor::new(),
            router: SmartRouter::new(),
            circuit_breaker: CircuitBreaker::new(5, Duration::from_secs(60)),
        })
    }

    pub fn get_engine(&self, name: &str) -> PdfResult<&Arc<dyn PdfEngine>> {
        self.engines.get(name)
            .ok_or_else(|| AdapterNotFoundError::new(
                format!("Unknown engine '{}'. Available: {:?}", name, self.engines.keys())
            ))
    }

    /// 对应 Python: PDFExtractor.extract_text()
    /// 增强特性：智能路由 + 引擎熔断降级
    pub async fn extract_text(
        &self,
        file_path: &Path,
        engine_name: Option<&str>,
    ) -> PdfResult<TextExtractionResult> {
        // 智能路由：如未指定引擎，根据文档特征自动选择
        let engine_name = match engine_name {
            Some(name) => name.to_string(),
            None => self.router.route(file_path).await
                .unwrap_or_else(|| self.default_engine.clone()),
        };
        self.validator.validate(file_path)?;

        // 检查缓存
        if let Some(ref cache) = self.cache {
            if let Some(cached) = cache.get(file_path, &engine_name).await {
                return Ok(cached);
            }
        }

        let engine = self.get_engine(&engine_name)?;

        // 主引擎提取，失败时触发熔断降级
        let result = match engine.extract_text(file_path).await {
            Ok(r) => {
                self.circuit_breaker.record_success(&engine_name);
                r
            }
            Err(e) => {
                self.circuit_breaker.record_failure(&engine_name);
                tracing::warn!(
                    engine = %engine_name,
                    error = %e,
                    "Primary engine failed, attempting fallback to {}",
                    self.fallback_engine
                );
                // 熔断降级：使用兜底引擎重试
                let fallback = self.get_engine(&self.fallback_engine)?;
                fallback.extract_text(file_path).await?
            }
        };

        // 写入缓存
        if let Some(ref cache) = self.cache {
            cache.set(file_path, &engine_name, &result).await;
        }

        Ok(result)
    }

    /// 对应 Python: PDFExtractor.extract_structured()
    /// 增强特性：智能路由 + 引擎熔断降级
    pub async fn extract_structured(
        &self,
        file_path: &Path,
        engine_name: Option<&str>,
        options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult> {
        let engine_name = match engine_name {
            Some(name) => name.to_string(),
            None => self.router.route(file_path).await
                .unwrap_or_else(|| self.default_engine.clone()),
        };
        self.validator.validate(file_path)?;

        let engine = self.get_engine(&engine_name)?;

        // 主引擎提取，失败时触发熔断降级
        match engine.extract_structured(file_path, options).await {
            Ok(r) => {
                self.circuit_breaker.record_success(&engine_name);
                Ok(r)
            }
            Err(e) => {
                self.circuit_breaker.record_failure(&engine_name);
                tracing::warn!(
                    engine = %engine_name,
                    error = %e,
                    "Primary engine failed for structured extraction, fallback to {}",
                    self.fallback_engine
                );
                let fallback = self.get_engine(&self.fallback_engine)?;
                fallback.extract_structured(file_path, options).await
            }
        }
    }

    /// 对应 Python: PDFExtractor.get_page_count()
    pub async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        self.validator.validate(file_path)?;
        let engine = self.get_engine(&self.default_engine)?;
        engine.get_page_count(file_path).await
    }

    /// 对应 Python: PDFExtractor.list_adapters()
    pub fn list_engines(&self) -> Vec<&str> {
        self.engines.keys().map(|s| s.as_str()).collect()
    }
}
```

### 5.3.1 智能路由器 SmartRouter

根据 PDF 文档特征自动选择最合适的引擎，实现"嗅探后分发"：

```rust
/// 智能路由器：根据文档特征自动选择引擎
pub struct SmartRouter;

impl SmartRouter {
    pub fn new() -> Self { Self }

    /// 嗅探文档特征并返回推荐引擎名称
    /// 路由策略：
    /// - 纯文本流 / 少量页面 → pdf-extract（最快）
    /// - 标准排版 / 需要布局信息 → lopdf（默认）
    /// - 复杂排版 / 特殊编码 / 非标结构 → pdfium（最兼容）
    pub async fn route(&self, file_path: &Path) -> Option<String> {
        // 快速读取 PDF 元数据（仅读文件头和 xref，不解析全部内容）
        let doc = lopdf::Document::load(file_path).ok()?;

        // 特征 1：页数 — 少页纯文本优先用 pdf-extract
        let page_count = doc.get_pages().len();
        if page_count <= 5 {
            // 检查前几页是否包含复杂对象（图片、表单等）
            let has_complex_objects = Self::detect_complex_layout(&doc);
            if !has_complex_objects {
                return Some("pdf-extract".to_string());
            }
        }

        // 特征 2：检测非标编码或特殊字体
        if Self::detect_special_encoding(&doc) {
            return Some("pdfium".to_string());
        }

        // 默认：lopdf（布局感知）
        Some("lopdf".to_string())
    }

    fn detect_complex_layout(doc: &lopdf::Document) -> bool {
        // 检测页面是否包含 Image XObject、Form XObject 等复杂对象
        // 简化实现：检查 Resources 字典中是否存在 XObject 子字典
        false // TODO: P4 阶段实现
    }

    fn detect_special_encoding(doc: &lopdf::Document) -> bool {
        // 检测是否使用 CIDFont（中文/日文/韩文）、Type3 字体等
        false // TODO: P4 阶段实现
    }
}
```

### 5.3.2 熔断器 CircuitBreaker

当主引擎连续失败时自动熔断，避免对已故障引擎的无效请求：

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// 引擎熔断器
/// 当某引擎连续失败次数超过阈值时进入 Open 状态，
/// 拒绝请求并直接降级；经过冷却期后进入 Half-Open 状态尝试恢复。
pub struct CircuitBreaker {
    failure_threshold: u32,           // 连续失败阈值（默认 5）
    cooldown: Duration,               // 冷却期（默认 60s）
    states: HashMap<String, EngineState>,
}

struct EngineState {
    consecutive_failures: AtomicU32,
    last_failure_time: Option<Instant>,
    is_open: bool,                    // true = 熔断打开（拒绝请求）
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown: Duration) -> Self {
        Self {
            failure_threshold,
            cooldown,
            states: HashMap::new(),
        }
    }

    pub fn record_success(&self, engine: &str) {
        // 重置连续失败计数
    }

    pub fn record_failure(&self, engine: &str) {
        // 递增失败计数，超过阈值则打开熔断
    }

    pub fn is_available(&self, engine: &str) -> bool {
        // 检查引擎是否可用（Closed 或 Half-Open 状态）
        true
    }
}
```

### 5.4 错误类型（对应 `exceptions.py`）

```rust
use thiserror::Error;

/// PDF 模块基础错误
/// 对应 Python: PdfModuleError
#[derive(Debug, Error)]
pub enum PdfModuleError {
    #[error("File not found: {0}")]
    FileNotFound(#[source] std::io::Error),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("File too large: {0}")]
    FileTooLarge(String),

    #[error("Extraction failed: {0}")]
    Extraction(String),

    #[error("Adapter not found: {0}")]
    AdapterNotFound(String),

    #[error("Corrupted file: {0}")]
    CorruptedFile(String),
}

impl PdfModuleError {
    /// 对应 Python: PdfModuleError.status_code
    pub fn status_code(&self) -> u16 {
        match self {
            Self::FileNotFound(_) => 404,
            Self::InvalidFileType(_) | Self::AdapterNotFound(_) => 400,
            Self::FileTooLarge(_) => 413,
            Self::CorruptedFile(_) => 422,
            Self::Extraction(_) => 500,
        }
    }

    /// 对应 Python: PdfModuleError.to_dict()
    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::json!({
            "error": match self {
                Self::FileNotFound(_) => "FileNotFoundError",
                Self::InvalidFileType(_) => "InvalidFileTypeError",
                Self::FileTooLarge(_) => "FileTooLargeError",
                Self::Extraction(_) => "ExtractionError",
                Self::AdapterNotFound(_) => "AdapterNotFoundError",
                Self::CorruptedFile(_) => "CorruptedFileError",
            },
            "message": self.to_string(),
            "status_code": self.status_code(),
        })
    }
}

pub type PdfResult<T> = Result<T, PdfModuleError>;
```

### 5.5 DTO（对应 `dto.py`）

```rust
use serde::{Serialize, Deserialize};

/// 对应 Python: TextExtractionMetadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtractionMetadata {
    pub whisper_hash: String,
    pub line_metadata: Option<serde_json::Value>,
}

/// 对应 Python: PageMetadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    pub page_number: u32,
    pub text: String,
    pub bbox: Option<(f64, f64, f64, f64)>,
    pub lines: Vec<LineInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineInfo {
    pub bbox: Vec<f64>,
    pub text: String,
}

/// 对应 Python: TextExtractionResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtractionResult {
    pub extracted_text: String,
    pub extraction_metadata: Option<TextExtractionMetadata>,
}

/// 对应 Python: StructuredExtractionResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredExtractionResult {
    pub extracted_text: String,
    pub page_count: u32,
    pub pages: Vec<PageMetadata>,
    pub extraction_metadata: Option<TextExtractionMetadata>,
    pub file_info: FileInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub file_path: String,
    pub file_size: u64,
    pub file_size_mb: f64,
}

/// 对应 Python: KeywordMatch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordMatch {
    pub keyword: String,
    pub page_number: u32,
    pub text: String,
    pub bbox: Option<(f64, f64, f64, f64)>,
    pub start_index: usize,
    pub end_index: usize,
    pub confidence: f64,
}

/// 对应 Python: KeywordSearchResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordSearchResult {
    pub keywords: Vec<String>,
    pub matches: Vec<KeywordMatch>,
    pub total_matches: usize,
    pub pages_with_matches: Vec<u32>,
}
```

### 5.6 缓存（对应 `cache.py`）

```rust
use moka::sync::Cache;
use sha2::{Sha256, Digest};
use std::path::Path;
use std::time::Duration;

/// 提取结果缓存
/// 对应 Python: ExtractionCache (cache.py)
/// 使用 moka 替代 OrderedDict，获得并发安全 + TTL 支持
pub struct ExtractionCache {
    cache: Cache<String, CacheEntry>,
    hits: AtomicU64,
    misses: AtomicU64,
}

#[derive(Clone)]
struct CacheEntry {
    result: String, // serde_json 序列化后的结果
    timestamp: Instant,
}

impl ExtractionCache {
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_size as u64)
            .time_to_idle(Duration::from_secs(ttl_seconds))
            .build();
        Self {
            cache,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// 对应 Python: ExtractionCache.file_hash()
    pub fn file_hash(file_path: &Path) -> Result<String, std::io::Error> {
        let mut hasher = Sha256::new();
        let mut file = std::fs::File::open(file_path)?;
        let mut buf = [0u8; 8192];
        loop {
            let n = std::io::Read::read(&mut file, &mut buf)?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// 对应 Python: ExtractionCache._make_key()
    fn make_key(&self, file_path: &Path, adapter: &str, kwargs: &str) -> String {
        let hash = Self::file_hash(file_path)
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());
        format!("{}|{}|{}", hash, adapter, kwargs)
    }

    /// 对应 Python: ExtractionCache.get()
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self, file_path: &Path, adapter: &str,
    ) -> Option<T> {
        let key = self.make_key(file_path, adapter, "");
        if let Some(entry) = self.cache.get(&key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            serde_json::from_str(&entry.result).ok()
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// 对应 Python: ExtractionCache.set()
    pub async fn set<T: serde::Serialize>(
        &self, file_path: &Path, adapter: &str, result: &T,
    ) {
        let key = self.make_key(file_path, adapter, "");
        let json = serde_json::to_string(result).unwrap_or_default();
        self.cache.insert(key, CacheEntry {
            result: json,
            timestamp: Instant::now(),
        });
    }

    /// 对应 Python: ExtractionCache.stats()
    pub fn stats(&self) -> serde_json::Value {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        serde_json::json!({
            "size": self.cache.entry_count(),
            "max_size": self.cache.max_capacity(),
            "hits": hits,
            "misses": misses,
            "hit_rate": if total > 0 { hits as f64 / total as f64 } else { 0.0 },
        })
    }
}
```

### 5.7 KeywordExtractor（对应 `keyword_extractor.py`）

> **分词引擎选型说明**：
> - **当前选型**：`jieba-rs` — 成熟稳定，API 与 Python jieba 兼容，支持搜索模式/精确模式/HMM 新词发现
> - **内存开销**：jieba-rs 内部字典加载常驻内存约 10MB，对大多数部署场景可接受
> - **极致轻量演进**：如果未来需要部署在极端受限的边缘节点（如嵌入式设备、Serverless 冷启动场景），可替换为基于 **Double-Array Trie**（双数组 Trie 树）的轻量级分词方案。Rust 生态中可选：
>   - `cedar` — Double-Array Trie 实现，内存占用仅为 jieba-rs 的 1/5~1/3，查找复杂度 O(1)
>   - `daachorse` — 基于 Double-Array Aho-Corasick 的多模式匹配，适合关键字高亮场景
>   - 自定义方案：用 `cedar` 构建精简词典（仅保留高频词），配合正则回退处理未登录词
> - **替换策略**：定义 `Tokenizer` trait，jieba-rs 和 cedar 分别实现，通过配置切换

```rust
use regex::Regex;
use jieba_rs::Jieba;
use std::collections::HashMap;

/// 关键字提取器
/// 对应 Python: KeywordExtractor (keyword_extractor.py)
pub struct KeywordExtractor {
    case_sensitive: bool,
    jieba: Jieba,
}

impl KeywordExtractor {
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
            jieba: Jieba::new(),
        }
    }

    /// 对应 Python: KeywordExtractor.search_keywords()
    pub fn search_keywords(
        &self,
        pages: &[PageMetadata],
        keywords: &[String],
        context_length: usize,
    ) -> KeywordSearchResult {
        let mut matches = Vec::new();
        let mut pages_with_matches = std::collections::HashSet::new();

        for page in pages {
            for keyword in keywords {
                let pattern = regex::escape(keyword);
                let re = if self.case_sensitive {
                    Regex::new(&pattern).unwrap()
                } else {
                    Regex::new(&format!("(?i){}", pattern)).unwrap()
                };

                for mat in re.find_iter(&page.text) {
                    let start = mat.start();
                    let end = mat.end();
                    let ctx_start = start.saturating_sub(context_length);
                    let ctx_end = (end + context_length).min(page.text.len());
                    let context = &page.text[ctx_start..ctx_end];

                    let bbox = Self::find_bbox_for_position(&page.lines, keyword, start);

                    matches.push(KeywordMatch {
                        keyword: keyword.clone(),
                        page_number: page.page_number,
                        text: context.to_string(),
                        bbox,
                        start_index: start,
                        end_index: end,
                        confidence: 1.0,
                    });
                    pages_with_matches.insert(page.page_number);
                }
            }
        }

        KeywordSearchResult {
            keywords: keywords.to_vec(),
            total_matches: matches.len(),
            pages_with_matches: pages_with_matches.into_iter().collect(),
            matches,
        }
    }

    /// 对应 Python: KeywordExtractor.extract_keywords_by_frequency()
    /// 使用 jieba-rs 进行中文分词，替代 Python 的简单正则分词
    pub fn extract_by_frequency(
        &self,
        text: &str,
        min_length: usize,
        max_length: usize,
        top_n: usize,
    ) -> Vec<(String, usize)> {
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        // 中文分词
        let words = self.jieba.cut(text, true);
        for word in words {
            let len = word.chars().count();
            if len >= min_length && len <= max_length && !word.trim().is_empty() {
                *word_counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }

        // 英文词提取
        let re = Regex::new(r"[a-zA-Z]{2,}").unwrap();
        for mat in re.find_iter(text) {
            let word = mat.as_str();
            let len = word.chars().count();
            if len >= min_length && len <= max_length {
                *word_counts.entry(word.to_string().to_lowercase()).or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<_> = word_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(top_n).collect()
    }

    /// 对应 Python: KeywordExtractor.highlight_keywords_in_text()
    pub fn highlight(&self, text: &str, keywords: &[String], prefix: &str, suffix: &str) -> String {
        let mut result = text.to_string();
        for keyword in keywords {
            let pattern = regex::escape(keyword);
            let re = if self.case_sensitive {
                Regex::new(&pattern).unwrap()
            } else {
                Regex::new(&format!("(?i){}", pattern)).unwrap()
            };
            result = re.replace_all(&result, &format!("{}{}{}", prefix, keyword, suffix)).to_string();
        }
        result
    }

    fn find_bbox_for_position(
        lines: &[LineInfo],
        text: &str,
        _position: usize,
    ) -> Option<(f64, f64, f64, f64)> {
        for line in lines {
            if line.text.contains(text) {
                if line.bbox.len() == 4 {
                    return Some((line.bbox[0], line.bbox[1], line.bbox[2], line.bbox[3]));
                }
            }
        }
        None
    }
}
```

### 5.8 FileValidator（对应 `validators.py`）

引入 `infer` crate 进行深度文件嗅探，通过底层 Magic Numbers 探测真实文件类型，彻底阻断将恶意可执行文件（如 ELF、PE、shell 脚本）伪装成 PDF 上传的安全风险。这是对外暴露 API 和 MCP Server 的必要安全加固。

```rust
use std::path::Path;
use infer;
use crate::error::*;

const ALLOWED_EXTENSIONS: &[&str] = &[".pdf"];

/// 文件验证器
/// 对应 Python: FileValidator (validators.py)
/// 安全增强：使用 infer crate 进行 Magic Number 深度嗅探
pub struct FileValidator {
    max_size_bytes: u64,
}

impl FileValidator {
    pub fn new(max_size_mb: u32) -> Self {
        Self {
            max_size_bytes: max_size_mb as u64 * 1024 * 1024,
        }
    }

    /// 对应 Python: FileValidator.validate()
    pub fn validate(&self, file_path: &Path) -> PdfResult<FileInfo> {
        // 1. 检查文件存在
        if !file_path.exists() {
            return Err(PdfModuleError::FileNotFound(
                std::io::Error::new(std::io::ErrorKind::NotFound, format!("File not found: {}", file_path.display()))
            ));
        }

        // 2. 检查扩展名
        let ext = file_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !ALLOWED_EXTENSIONS.contains(&format!(".{}", ext).as_str()) {
            return Err(PdfModuleError::InvalidFileType(
                format!("Invalid extension '.{}', allowed: {:?}", ext, ALLOWED_EXTENSIONS)
            ));
        }

        // 3. 检查文件大小
        let file_size = std::fs::metadata(file_path)
            .map_err(|e| PdfModuleError::CorruptedFile(e.to_string()))?
            .len();
        if file_size > self.max_size_bytes {
            return Err(PdfModuleError::FileTooLarge(
                format!("File size {:.1}MB exceeds limit", file_size as f64 / 1024.0 / 1024.0)
            ));
        }
        if file_size == 0 {
            return Err(PdfModuleError::CorruptedFile("File is empty".into()));
        }

        // 4. 深度文件嗅探（安全加固核心）
        //    使用 infer crate 读取文件头部 Magic Numbers，
        //    验证真实文件类型为 application/pdf。
        //    这比仅检查 %PDF 头更安全：infer 能识别 ELF、PE、Mach-O、
        //    ZIP(伪装)、脚本等恶意文件的真实类型，彻底阻断伪装上传。
        let inferred_type = infer::get_from_path(file_path)
            .map_err(|e| PdfModuleError::CorruptedFile(format!("Cannot read file for sniffing: {}", e)))?;

        match inferred_type {
            Some(t) if t.mime_type() == "application/pdf" => { /* 通过 */ }
            Some(t) => {
                // 文件真实类型不是 PDF，可能是伪装的恶意文件
                return Err(PdfModuleError::InvalidFileType(
                    format!(
                        "File content type mismatch: extension is .pdf but actual type is {} ({}). \
                         Possible malicious file upload attempt.",
                        t.mime_type(), t.extension()
                    )
                ));
            }
            None => {
                // infer 无法识别类型，回退到 %PDF 头检查
                let mut file = std::fs::File::open(file_path)
                    .map_err(|e| PdfModuleError::CorruptedFile(e.to_string()))?;
                let mut header = [0u8; 4];
                std::io::Read::read_exact(&mut file, &mut header)
                    .map_err(|e| PdfModuleError::CorruptedFile(format!("Cannot read header: {}", e)))?;
                if &header != b"%PDF" {
                    return Err(PdfModuleError::CorruptedFile(
                        format!("Not a valid PDF, header: {:?}", header)
                    ));
                }
            }
        }

        Ok(FileInfo {
            file_path: file_path.to_string_lossy().to_string(),
            file_size,
            file_size_mb: (file_size as f64 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
        })
    }

    /// 对应 Python: FileValidator.validate_upload()
    pub fn validate_upload(&self, filename: &str, content_length: Option<u64>) -> PdfResult<()> {
        if filename.is_empty() {
            return Err(PdfModuleError::InvalidFileType("No filename provided".into()));
        }
        let ext = std::path::Path::new(filename).extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !ALLOWED_EXTENSIONS.contains(&format!(".{}", ext).as_str()) {
            return Err(PdfModuleError::InvalidFileType(
                format!("Invalid extension '.{}'", ext)
            ));
        }
        if let Some(len) = content_length {
            if len > self.max_size_bytes {
                return Err(PdfModuleError::FileTooLarge(
                    format!("Upload size {:.1}MB exceeds limit", len as f64 / 1024.0 / 1024.0)
                ));
            }
        }
        Ok(())
    }
}
```

## 6. 接口层设计

### 6.1 MCP Server（对应 `mcp_server.py`）

使用 `rmcp` crate 实现 MCP 协议，支持 stdio 和 SSE 两种传输模式：

```rust
use rmcp::{Server, Tool, ToolCall};
use crate::core::PdfExtractorService;

/// MCP 工具注册（对应 Python: PdfMcpServer.tools）
pub fn register_tools(service: Arc<PdfExtractorService>) -> Server {
    Server::new("pdf-module-mcp", "1.0.0")
        .tool(Tool::new("extract_text")
            .description("Extract text content from a PDF file")
            .param("file_path", "string", "Absolute path to the PDF file", true)
            .param("adapter", "string", "Extraction engine: lopdf, pdf-extract, pdfium", false)
            .handler(|args, svc| {
                let path = PathBuf::from(args["file_path"].as_str());
                let adapter = args.get("adapter").map(|v| v.as_str());
                // tokio::spawn + svc.extract_text
            })
        )
        .tool(Tool::new("extract_structured")
            .description("Extract structured data with page info and positions")
            .param("file_path", "string", "Absolute path to the PDF file", true)
            .param("adapter", "string", "Extraction engine", false)
            .param("enable_highlight", "boolean", "Include highlight metadata", false)
            .handler(/* ... */)
        )
        .tool(Tool::new("get_page_count")
            .description("Get the number of pages in a PDF file")
            .param("file_path", "string", "Absolute path to the PDF file", true)
            .handler(/* ... */)
        )
        .tool(Tool::new("search_keywords")
            .description("Search for keywords in a PDF file")
            .param("file_path", "string", "Absolute path to the PDF file", true)
            .param("keywords", "array", "List of keywords to search", true)
            .param("case_sensitive", "boolean", "Case sensitive search", false)
            .handler(/* ... */)
        )
        .tool(Tool::new("extract_keywords")
            .description("Auto-extract top keywords by frequency")
            .param("file_path", "string", "Absolute path to the PDF file", true)
            .param("top_n", "integer", "Number of top keywords", false)
            .handler(/* ... */)
        )
        .tool(Tool::new("list_adapters")
            .description("List available PDF extraction engines")
            .handler(|_, svc| serde_json::json!({"adapters": svc.list_engines()}))
        )
        .tool(Tool::new("cache_stats")
            .description("Get cache statistics")
            .handler(|_, svc| svc.cache_stats())
        )
}
```

**MCP 传输模式**：
- **stdio**（默认）：对应现有 Python 实现，通过 stdin/stdout 传输 JSON-RPC
- **SSE**（新增）：`POST /sse` 端点，适合容器化部署和远程 AI Agent 调用

### 6.2 REST API（对应 `controllers.py`）

```rust
use axum::{
    routing::{get, post},
    Router, Json, extract::Multipart,
};
use crate::core::PdfExtractorService;

/// 构建 REST API 路由
/// 对应 Python: controllers.py 中的 Blueprint 路由
pub fn build_router(service: Arc<PdfExtractorService>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/extract", post(extract_text_file))
        .route("/extract-json", post(extract_structured_json))
        .route("/info", post(get_pdf_info))
        .route("/adapters", get(list_adapters))
        .route("/cache/stats", get(cache_stats))
        .route("/metrics", get(serve_metrics))  // Prometheus 指标端点
        .with_state(service)
}

/// GET /health — 对应 Python: health()
async fn health() -> &'static str {
    "OK"
}

/// POST /extract — 对应 Python: extract()
async fn extract_text_file(
    State(svc): State<Arc<PdfExtractorService>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, PdfModuleError> {
    // 解析 multipart 上传，保存临时文件，调用 svc.extract_text()
    // 返回 text/plain 文件下载
}

/// POST /extract-json — 对应 Python: extract_json()
async fn extract_structured_json(
    State(svc): State<Arc<PdfExtractorService>>,
    mut multipart: Multipart,
) -> Result<Json<StructuredExtractionResult>, PdfModuleError> {
    // 解析 multipart 上传，保存临时文件，调用 svc.extract_structured()
    // 返回 application/json
}

/// POST /info — 对应 Python: info()
async fn get_pdf_info(
    State(svc): State<Arc<PdfExtractorService>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, PdfModuleError> {
    // 返回 { filename, page_count, mime_type }
}

/// GET /adapters — 对应 Python: list_adapters()
async fn list_adapters(
    State(svc): State<Arc<PdfExtractorService>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"adapters": svc.list_engines()}))
}
```

### 6.3 CLI 入口（统一二进制，替代两个 Python 入口）

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pdf-module", version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动 MCP 服务器 (stdio 传输)
    Mcp {
        /// 传输模式: stdio 或 sse
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// SSE 监听端口 (仅 sse 模式)
        #[arg(long, default_value_t = 8000)]
        port: u16,
    },
    /// 启动 REST API 服务器
    Rest {
        /// 监听地址
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// 监听端口
        #[arg(long, default_value_t = 8000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let config = ServerConfig::from_env();

    match cli.command {
        Commands::Mcp { transport, port } => {
            // 启动 MCP 服务器
        }
        Commands::Rest { host, port } => {
            // 启动 REST API 服务器
        }
    }
    Ok(())
}
```

## 7. 配置管理（对应 `config.py`）

```rust
use serde::Deserialize;

/// 对应 Python: ServerConfig (config.py)
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub debug: bool,
    pub api_url_prefix: String,
    pub log_level: String,
    pub log_file: Option<String>,
    pub max_file_size_mb: u32,
    pub default_adapter: String,
    pub cache_enabled: bool,
    pub cache_max_size: usize,
    pub cache_ttl_seconds: u64,
}

impl ServerConfig {
    /// 对应 Python: ServerConfig.from_env()
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("PDF_SERVER_HOST").unwrap_or("0.0.0.0".into()),
            port: std::env::var("PDF_SERVER_PORT").unwrap_or("8000".into()).parse().unwrap(),
            debug: std::env::var("PDF_SERVER_DEBUG").unwrap_or("false".into()) == "true",
            api_url_prefix: std::env::var("API_URL_PREFIX").unwrap_or("/api/v1".into()),
            log_level: std::env::var("PDF_LOG_LEVEL").unwrap_or("INFO".into()).to_uppercase(),
            log_file: std::env::var("PDF_LOG_FILE").ok(),
            max_file_size_mb: std::env::var("PDF_MAX_FILE_SIZE_MB").unwrap_or("200".into()).parse().unwrap(),
            default_adapter: std::env::var("PDF_DEFAULT_ADAPTER").unwrap_or("lopdf".into()),
            cache_enabled: std::env::var("PDF_CACHE_ENABLED").unwrap_or("true".into()) == "true",
            cache_max_size: std::env::var("PDF_CACHE_MAX_SIZE").unwrap_or("100".into()).parse().unwrap(),
            cache_ttl_seconds: std::env::var("PDF_CACHE_TTL_SECONDS").unwrap_or("3600".into()).parse().unwrap(),
        }
    }
}
```

## 7.5 可观测性设计（Prometheus 指标导出）

系统脱离 Python 运行时后具备极高的并发上限，必须引入指标监控以支撑生产环境性能调优。使用 `metrics` + `metrics-exporter-prometheus` crate 实现零成本抽象的指标埋点。

### 7.5.1 核心指标定义

```rust
use metrics::{counter, gauge, histogram};

/// 指标命名规范：pdf_module_<子系统>_<指标名>
pub mod metrics_def {
    /// 引擎单页提取延迟（直方图，单位：ms）
    /// 标签：engine = lopdf | pdf-extract | pdfium
    /// 用途：识别慢引擎，对比不同引擎的性能差异
    pub fn extraction_duration_ms(engine: &str, duration_ms: f64) {
        histogram!("pdf_module_extraction_duration_ms", "engine" => engine).record(duration_ms);
    }

    /// 引擎提取总次数（计数器）
    /// 标签：engine, result = success | failure
    pub fn extraction_total(engine: &str, result: &str) {
        counter!("pdf_module_extraction_total", "engine" => engine, "result" => result).increment(1);
    }

    /// 缓存命中率（仪表盘）
    /// 用途：监控 moka 缓存效果，辅助调整 max_size 和 TTL
    pub fn cache_hit_rate(hit_rate: f64) {
        gauge!("pdf_module_cache_hit_rate").set(hit_rate);
    }

    /// 缓存操作次数（计数器）
    /// 标签：operation = hit | miss | eviction
    pub fn cache_operations(operation: &str) {
        counter!("pdf_module_cache_operations", "operation" => operation).increment(1);
    }

    /// 智能路由分布（计数器）
    /// 标签：engine = lopdf | pdf-extract | pdfium
    /// 用途：了解文档特征分布，验证路由策略是否合理
    pub fn route_distribution(engine: &str) {
        counter!("pdf_module_route_distribution", "engine" => engine).increment(1);
    }

    /// 熔断器状态（仪表盘）
    /// 标签：engine, state = closed | open | half_open
    pub fn circuit_breaker_state(engine: &str, state: &str) {
        gauge!("pdf_module_circuit_breaker_state", "engine" => engine, "state" => state).set(1.0);
    }

    /// 当前在处理的 PDF 文件大小（直方图，单位：MB）
    pub fn file_size_mb(size_mb: f64) {
        histogram!("pdf_module_file_size_mb").record(size_mb);
    }

    /// 关键字搜索延迟（直方图，单位：ms）
    pub fn keyword_search_duration_ms(duration_ms: f64) {
        histogram!("pdf_module_keyword_search_duration_ms").record(duration_ms);
    }
}
```

### 7.5.2 Prometheus 端点与采集

```rust
use metrics_exporter_prometheus::PrometheusBuilder;

/// 初始化 Prometheus 指标导出器
/// 在 REST API 模式下，通过 /metrics 端点暴露指标
pub fn init_metrics() {
    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("Failed to install Prometheus exporter");
}

/// GET /metrics — 供 Prometheus 采集
async fn serve_metrics() -> impl IntoResponse {
    use metrics_exporter_prometheus::exporter;
    let body = exporter().render();
    axum::response::Html(body)
}
```

### 7.5.3 指标埋点示例（在 PdfExtractorService 中）

```rust
// 在 extract_text 方法中埋点
let start = Instant::now();
let result = engine.extract_text(file_path).await?;
let elapsed = start.elapsed().as_secs_f64() * 1000.0;

metrics_def::extraction_duration_ms(&engine_name, elapsed);
metrics_def::extraction_total(&engine_name, "success");
metrics_def::file_size_mb(file_size as f64 / 1024.0 / 1024.0);
```

### 7.5.4 推荐 Grafana 监控面板

| 面板 | 指标 | 用途 |
|------|------|------|
| 引擎延迟 P50/P95/P99 | `pdf_module_extraction_duration_ms` | 识别慢引擎和长尾延迟 |
| 引擎成功率 | `pdf_module_extraction_total{result="success"} / pdf_module_extraction_total` | 监控引擎稳定性 |
| 缓存命中率趋势 | `pdf_module_cache_hit_rate` | 调整缓存参数 |
| 路由分布饼图 | `pdf_module_route_distribution` | 验证智能路由策略 |
| 熔断器状态 | `pdf_module_circuit_breaker_state` | 告警引擎故障 |
| 文件大小分布 | `pdf_module_file_size_mb` | 容量规划 |

## 8. 部署方案

### 8.1 Dockerfile（多阶段构建）

```dockerfile
# Stage 1: Build
FROM rust:1.82-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p pdf-module

# Stage 2: Runtime (MCP 模式)
FROM debian:bookworm-slim AS mcp
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pdf-module /usr/local/bin/
ENV PDF_DEFAULT_ADAPTER=lopdf
ENV PDF_CACHE_ENABLED=true
ENTRYPOINT ["pdf-module", "mcp"]

# Stage 3: Runtime (REST 模式)
FROM debian:bookworm-slim AS rest
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pdf-module /usr/local/bin/
EXPOSE 8000
ENV PDF_DEFAULT_ADAPTER=lopdf
ENV PDF_CACHE_ENABLED=true
ENTRYPOINT ["pdf-module", "rest"]
```

### 8.2 docker-compose.yml

```yaml
services:
  pdf-mcp:
    build:
      context: .
      target: mcp
    stdin_open: true
    environment:
      - PDF_DEFAULT_ADAPTER=lopdf
      - PDF_CACHE_ENABLED=true
    volumes:
      - ./data:/data:ro

  pdf-rest:
    build:
      context: .
      target: rest
    ports:
      - "8000:8000"
    environment:
      - PDF_SERVER_HOST=0.0.0.0
      - PDF_SERVER_PORT=8000
      - PDF_DEFAULT_ADAPTER=lopdf
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/api/v1/x2text/health"]
      interval: 30s
      timeout: 5s
      retries: 3
```

### 8.3 镜像体积对比

| 指标 | Python 版本 | Rust 版本 |
|------|------------|-----------|
| 基础镜像 | python:3.12-slim (~130MB) | debian:bookworm-slim (~80MB) |
| 运行时依赖 | Python + pdfplumber + PyMuPDF + Flask | 静态链接，仅 libc + libssl |
| 最终镜像 | ~200MB | ~30-40MB |
| 启动时间 | ~2-3s | ~50ms |
| 内存占用 | ~80-150MB (idle) | ~5-15MB (idle) |

## 9. 迁移策略与兼容性

### 9.1 引擎名称兼容映射

为保持与现有 MCP 客户端和 REST API 调用者的兼容性，保留原有引擎名称作为别名：

| Python 名称 | Rust 实际引擎 | 说明 |
|-------------|--------------|------|
| `pymupdf` | `LopdfEngine` | 默认引擎，功能最全 |
| `fitz` | `LopdfEngine` | PyMuPDF 别名，保持兼容 |
| `pdfplumber` | `PdfExtractEngine` | 快速纯文本提取 |
| `lopdf` | `LopdfEngine` | Rust 原生名称 |
| `pdf-extract` | `PdfExtractEngine` | Rust 原生名称 |
| `pdfium` | `PdfiumEngine` | 新增：高兼容性引擎 |

### 9.2 API 接口完全兼容

- REST API 路径、请求格式、响应格式保持不变
- MCP 工具名称、参数 schema、响应格式保持不变
- 环境变量名称和默认值保持不变（仅 `PDF_DEFAULT_ADAPTER` 默认值从 `pymupdf` 改为 `lopdf`，但 `pymupdf` 仍作为别名可用）

### 9.3 分阶段迁移

| 阶段 | 内容 | 风险 |
|------|------|------|
| **P1** | `pdf-core` 核心库：engine trait + lopdf + pdf-extract + dto + error + config + validator (含 infer 深度嗅探) + cache + metrics 指标定义 | 低 |
| **P1-PoC** | lopdf Text Matrix (Tm) 解析 PoC：验证从内容流提取 bbox/LineInfo 的可行性和精度 | 中 |
| **P2** | `pdf-mcp`：MCP stdio 服务，7 个工具完整实现 | 中（rmcp crate 成熟度） |
| **P3** | `pdf-rest`：REST API 服务，6 个端点 + `/metrics` Prometheus 端点完整实现 | 低 |
| **P4** | `pdfium` 引擎集成 + 智能路由 (SmartRouter) + 引擎熔断降级 (CircuitBreaker) | 中（pdfium C FFI） |
| **P5** | SSE 传输模式 + PyO3 Python 绑定（可选）+ Double-Array Trie 分词替换评估 | 低 |

## 10. 后续演进方向

1. **引擎熔断**（P4 阶段）：主引擎解析失败时自动降级至 pdfium，`CircuitBreaker` 已设计，待 P4 阶段实现完整的三态转换（Closed → Open → Half-Open）
2. **流式提取**（P2/P3 阶段）：`extract_page_stream()` 接口已定义在 PdfEngine trait 中，针对超长 PDF 实现按页读取、按页吐出 JSON，将内存峰值压至单页级别
3. **智能路由**（P4 阶段）：`SmartRouter` 已设计，待实现 `detect_complex_layout()` 和 `detect_special_encoding()` 的完整特征检测逻辑
4. **TF-IDF 原生实现**：用 Rust 实现轻量 TF-IDF（替代 sklearn 依赖），或集成 `tantivy` 搜索引擎
5. **并行提取**：利用 `tokio` 任务池，多页并行提取，充分利用多核 CPU
6. **可观测性**（P1/P3 阶段）：`metrics` + `metrics-exporter-prometheus` 已选型，核心指标已定义，P3 阶段暴露 `/metrics` 端点
7. **Double-Array Trie 分词**（P5 评估）：如需极致轻量部署，将 jieba-rs 替换为 `cedar` + `daachorse` 方案，内存降至 1/5~1/3
8. **OpenTelemetry 集成**：如需分布式追踪（跨 AI Agent 调用链），可从 `metrics` 迁移至 `opentelemetry` 统一可观测性框架

## 11. Python → Rust 模块映射总表

| Python 模块 | Rust 模块 | Crate 依赖 | 备注 |
|-------------|-----------|-----------|------|
| `pdf_extractor.py` | `pdf-core::extractor` | 内部 | 编排器，逻辑 1:1 对应 |
| `adapters/base.py` | `pdf-core::engine::trait` | `async-trait` | Trait 替代 ABC |
| `adapters/pymupdf_adapter.py` | `pdf-core::engine::lopdf` | `lopdf` | lopdf 替代 PyMuPDF |
| `adapters/pdfplumber_adapter.py` | `pdf-core::engine::pdf_extract` | `pdf-extract` | pdf-extract 替代 pdfplumber |
| (新增) | `pdf-core::engine::pdfium` | `pdfium-render` | 高兼容性引擎 |
| `keyword_extractor.py` | `pdf-core::keyword` | `regex`, `jieba-rs` | jieba-rs 替代简单正则分词 |
| `cache.py` | `pdf-core::cache` | `moka`, `sha2` | moka 替代 OrderedDict LRU |
| `validators.py` | `pdf-core::validator` | `infer` | infer 深度嗅探替代简单 %PDF 头检查 |
| `dto.py` | `pdf-core::dto` | `serde` | serde Serialize/Deserialize 替代 dataclass |
| `exceptions.py` | `pdf-core::error` | `thiserror` | thiserror 枚举替代异常层次 |
| `config.py` | `pdf-core::config` | `dotenvy` | 逻辑 1:1 对应 |
| (新增) | `pdf-core::metrics` | `metrics`, `metrics-exporter-prometheus` | Prometheus 指标导出 |
| (新增) | `pdf-core::extractor` (SmartRouter) | `lopdf` | 智能路由：嗅探后分发 |
| (新增) | `pdf-core::extractor` (CircuitBreaker) | 内部 | 引擎熔断降级 |
| `mcp_server.py` | `pdf-mcp::server` | `rmcp` | rmcp 替代手写 JSON-RPC |
| `controllers.py` | `pdf-rest::routes` | `axum` | axum 替代 Flask |
| `run.py` | `pdf-mcp::main` / `pdf-rest::main` | `clap`, `tokio` | 统一二进制 + 子命令 |
