# Rust PDF 模块架构审计与重构计划

> **审计日期**: 2026-04-25  
> **项目**: pdf-module-rs  
> **架构师**: Rust 软件架构审计报告

---

## 📋 目录

- [一、项目现状分析](#一项目现状分析)
- [二、核心问题识别](#二核心问题识别)
- [三、分阶段重构计划](#三分阶段重构计划)
- [四、重构风险评估](#四重构风险评估)
- [五、重构收益评估](#五重构收益评估)
- [六、实施建议](#六实施建议)
- [七、总结](#七总结)

---

## 一、项目现状分析

### 1.1 整体架构评价

#### ✅ 优点

| 优点 | 说明 | 位置 |
|------|------|------|
| **清晰的层次架构** | 采用 Workspace 模式，分为核心层、中间层、应用层 | `Cargo.toml` |
| **无循环依赖** | 依赖关系为单向层次结构 | 所有 crate |
| **丰富的 Trait 抽象** | `PdfEngine`、`FileStorage`、`ToolHandler` 等核心接口设计良好 | `pdf-core/src/engine/trait.rs` |
| **完善的错误处理** | 使用 `thiserror` 提供结构化错误类型 | `pdf-core/src/error.rs` |
| **智能路由系统** | `SmartRouter` 根据文档特征自动选择最优引擎 | `pdf-core/src/extractor.rs:23-162` |
| **熔断器模式** | 提供故障容错能力 | `pdf-core/src/control/circuit_breaker.rs` |
| **插件化架构** | 支持编译时和运行时插件发现 | `pdf-core/src/plugin/` |
| **多传输支持** | MCP 服务器支持 stdio 和 SSE 传输 | `pdf-mcp/src/` |
| **可观测性** | 集成 Prometheus 指标和审计日志 | `pdf-core/src/metrics.rs` |

#### ⚠️ 待改进

| 问题 | 严重程度 | 影响范围 |
|------|----------|----------|
| **类型重复定义** | 🔴 高 | 4 处重复 |
| **错误处理分散** | 🟡 中 | 2 个 crate |
| **Boilerplate 代码** | 🟡 中 | 多处重复模式 |
| **配置管理分散** | 🟢 低 | 多个模块 |

---

### 1.2 项目结构概览

```
pdf-module-rs/
├── Cargo.toml                 # Workspace 配置
├── crates/
│   ├── pdf-core/             # 核心库 (底层)
│   │   ├── engine/           # PDF 引擎实现
│   │   │   ├── trait.rs      # PdfEngine trait
│   │   │   ├── lopdf.rs      # LopdfEngine
│   │   │   ├── pdf_extract.rs # PdfExtractEngine
│   │   │   └── pdfium.rs     # PdfiumEngine
│   │   ├── cache.rs          # 缓存系统
│   │   ├── keyword.rs        # 关键词提取
│   │   ├── validator.rs      # 文件验证
│   │   ├── audit/            # 审计日志
│   │   ├── storage/          # 存储后端
│   │   ├── control/          # 控制平面
│   │   │   └── circuit_breaker.rs
│   │   ├── plugin/           # 插件架构
│   │   │   ├── tool_handler.rs
│   │   │   ├── registry.rs
│   │   │   └── discovery.rs
│   │   ├── protocol/         # 协议定义
│   │   ├── streamer/         # 消息流
│   │   ├── error.rs          # 错误类型
│   │   ├── dto.rs            # 数据传输对象
│   │   └── config.rs         # 配置管理
│   │
│   ├── pdf-etl/              # ETL 流水线 (中间层)
│   │   ├── database/         # 数据库适配器
│   │   ├── llm/              # LLM 适配器
│   │   ├── schema/           # Schema 定义
│   │   ├── pipeline/         # 流水线实现
│   │   ├── tools/            # MCP 工具
│   │   ├── error.rs          # 错误类型
│   │   └── dto.rs            # DTO
│   │
│   ├── pdf-mcp/              # MCP 服务器 (应用层)
│   │   ├── server.rs         # stdio 传输
│   │   ├── sse.rs            # SSE 传输
│   │   └── mcp_server.rs     # MCP 协议
│   │
│   ├── pdf-rest/             # REST API 服务器 (应用层)
│   │   ├── routes.rs         # 路由定义
│   │   └── api/              # API 处理器
│   │
│   └── pdf-python/           # Python 绑定 (可选)
│
└── tests/                    # 测试文件
    ├── e2e_test.rs
    ├── integration_test.rs
    ├── perf_test.rs
    └── stress_test.rs
```

**代码统计**:
- 总文件数: 103 个 Rust 源文件
- 总代码行数: 21,976 行
- 测试文件: 4 个

---

### 1.3 依赖关系图

```
┌─────────────┐
│  pdf-mcp    │─────┐
└─────────────┘     │
                    ├──> ┌───────────┐
┌─────────────┐     │    │ pdf-core  │
│  pdf-rest   │─────┘    └───────────┘
└─────────────┘              ▲
                             │
┌─────────────┐              │
│  pdf-etl    │──────────────┘
└─────────────┘

┌─────────────┐
│ pdf-python  │──────────────> pdf-core
└─────────────┘
```

**结论**: ✅ 无循环依赖，依赖关系清晰

---

## 二、核心问题识别

### 2.1 重复类型定义

#### 问题 1: CircuitBreaker 重复

| 位置 | 文件路径 | 行号 | 说明 |
|------|----------|------|------|
| 定义 1 | `pdf-core/src/extractor.rs` | 191-276 | 提取器专用，使用 Mutex |
| 定义 2 | `pdf-core/src/control/circuit_breaker.rs` | 52-189 | 控制平面通用，无锁实现 |

**问题分析**:
- 两处实现不同，功能重复
- `extractor.rs` 中的版本使用 `Mutex<HashMap>`，性能较差
- `control/circuit_breaker.rs` 中的版本使用原子操作，性能更好

**建议**: 统一使用 `control/circuit_breaker.rs` 中的无锁实现

---

#### 问题 2: ToolContext 重复

| 位置 | 文件路径 | 行号 | 字段 |
|------|----------|------|------|
| 定义 1 | `pdf-core/src/dto.rs` | 433-445 | execution_id, org_id, workflow_id, user_id, request_id, metadata |
| 定义 2 | `pdf-core/src/plugin/tool_handler.rs` | 87-100 | 完全相同的字段 |

**问题分析**:
- 字段完全相同，应该统一
- `dto.rs` 版本支持序列化（`Serialize + Deserialize`）
- `tool_handler.rs` 版本提供了 Builder 模式方法

**建议**: 统一到 `dto.rs`，并添加 Builder 模式方法

---

#### 问题 3: ToolExecutionOptions 重复

| 位置 | 文件路径 | 行号 |
|------|----------|------|
| 定义 1 | `pdf-core/src/dto.rs` | 448-460 |
| 定义 2 | `pdf-core/src/plugin/tool_handler.rs` | (未明确行号) |

**建议**: 统一到 `dto.rs`

---

#### 问题 4: PathValidationConfig 重复

| 位置 | 文件路径 | 行号 |
|------|----------|------|
| 定义 1 | `pdf-core/src/validator.rs` | 12 |
| 定义 2 | `pdf-core/src/config.rs` | 111 |

**建议**: 统一到 `config.rs`

---

### 2.2 错误处理机制分散

#### 现状对比

| Crate | 错误类型 | 变体数量 | 文件路径 |
|-------|----------|----------|----------|
| pdf-core | `PdfModuleError` | 22 个 | `pdf-core/src/error.rs` |
| pdf-etl | `EtlError` | 13 个 | `pdf-etl/src/error.rs` |

#### 问题分析

1. **错误类型不统一**
   - 两个 crate 使用不同的错误类型
   - 无法直接转换

2. **错误转换缺失**
   - `pdf-etl` 无法直接返回 `pdf-core` 的错误
   - 需要手动转换

3. **状态码映射重复**
   ```rust
   // pdf-core/src/error.rs:94-122
   pub fn status_code(&self) -> u16 {
       match self {
           Self::FileNotFound(_) => 404,
           Self::InvalidFileType(_) => 400,
           // ... 22 个变体
       }
   }
   
   // pdf-etl/src/error.rs (类似实现)
   ```

4. **JSON 序列化重复**
   - `PdfModuleError::to_dict()`
   - `EtlError::to_json()`
   - 实现逻辑相似

#### 错误类型对比表

| 错误类型 | pdf-core | pdf-etl | 说明 |
|----------|----------|---------|------|
| 文件未找到 | `FileNotFound` | `FileNotFoundError` | 重复 |
| 文件损坏 | `CorruptedFile` | `CorruptedPdfError` | 重复 |
| 提取失败 | `Extraction` | `ExtractionError` | 重复 |
| 配置错误 | `ConfigError` | `ConfigError` | 重复 |
| IO 错误 | `IoError` | `IoError` | 重复 |
| JSON 错误 | `JsonError` | `JsonError` | 重复 |
| 工具注册错误 | `ToolRegistrationError` | - | pdf-core 特有 |
| 熔断器错误 | `CircuitBreakerOpen` | - | pdf-core 特有 |
| LLM 错误 | - | `LLMError` | pdf-etl 特有 |
| 数据库错误 | - | `DatabaseError` | pdf-etl 特有 |

---

### 2.3 Boilerplate 代码模式

#### 模式 1: 引擎注册

**位置**: `pdf-core/src/extractor.rs:299-312`

```rust
// 当前实现 - 重复的注册模式
let lopdf = Arc::new(LopdfEngine::new());
engines.insert("lopdf".into(), lopdf.clone());
engines.insert("pymupdf".into(), lopdf.clone());
engines.insert("fitz".into(), lopdf);

let pdf_extract = Arc::new(PdfExtractEngine::new());
engines.insert("pdf-extract".into(), pdf_extract.clone());
engines.insert("pdfplumber".into(), pdf_extract);

let pdfium = Arc::new(PdfiumEngine::new());
engines.insert("pdfium".into(), pdfium);
```

**问题**: 每个引擎都需要 3-4 行重复代码

**建议**: 使用宏简化
```rust
register_engines!(engines,
    LopdfEngine => ["lopdf", "pymupdf", "fitz"],
    PdfExtractEngine => ["pdf-extract", "pdfplumber"],
    PdfiumEngine => ["pdfium"],
);
```

---

#### 模式 2: 错误状态码映射

**位置**: `pdf-core/src/error.rs:94-122`

```rust
pub fn status_code(&self) -> u16 {
    match self {
        Self::FileNotFound(_) => 404,
        Self::InvalidFileType(_) => 400,
        Self::FileTooLarge(_) => 413,
        Self::CorruptedFile(_) => 422,
        Self::Extraction(_) => 500,
        Self::IoError(_) => 500,
        Self::ToolRegistrationError(_) => 500,
        Self::ToolExecutionError(_) => 500,
        Self::ValidationFailed(_) => 400,
        Self::StorageError(_) => 500,
        Self::AuditError(_) => 500,
        Self::ConfigError(_) => 500,
        Self::MessageSendError(_) => 500,
        Self::ToolNotFound(_) => 404,
        Self::InvalidToolDefinition(_) => 400,
        Self::PluginLoadError(_) => 500,
        Self::JsonError(_) => 500,
        Self::ToolAlreadyRegistered(_) => 409,
        Self::RateLimitExceeded(_) => 429,
        Self::CircuitBreakerOpen(_) => 503,
        Self::SchemaValidationError(_) => 400,
        Self::ExecutionTimeout(_) => 408,
        Self::ToolUnavailable(_, _) => 503,
        Self::DiscoveryError(_) => 500,
        Self::ControlPlaneError(_) => 500,
    }
}
```

**问题**: 每个错误变体都需要手动映射，容易遗漏

**建议**: 使用宏或属性自动生成

---

#### 模式 3: 错误类型名称

**位置**: `pdf-core/src/error.rs:125-154`

```rust
pub fn error_type(&self) -> &'static str {
    match self {
        Self::FileNotFound(_) => "FileNotFoundError",
        Self::InvalidFileType(_) => "InvalidFileTypeError",
        Self::FileTooLarge(_) => "FileTooLargeError",
        Self::Extraction(_) => "ExtractionError",
        // ... 22 个变体
    }
}
```

**问题**: 与错误变体名称高度相关，可以自动生成

---

#### 模式 4: Default 实现

**位置**: 多个配置结构

```rust
// pdf-core/src/config.rs:42-50
impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 1000,
            ttl_seconds: 3600,
            cache_dir: None,
        }
    }
}

// pdf-core/src/config.rs:79-xxx
impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: AuditBackendConfig::Memory,
            retention_days: 30,
        }
    }
}

// ... 更多类似的 Default 实现
```

**问题**: 几乎所有配置结构都有类似的 Default 实现

**建议**: 使用 `#[derive(Default)]` 或配置宏

---

### 2.4 模块耦合分析

#### 当前依赖关系

```
pdf-mcp ────┐
            ├──> pdf-core
pdf-rest ───┘

pdf-etl ────> pdf-core

pdf-python ─> pdf-core
```

#### 问题分析

1. **pdf-etl 与 pdf-core 错误类型不兼容**
   - `pdf-etl` 的函数返回 `Result<T, EtlError>`
   - `pdf-core` 的函数返回 `PdfResult<T>`
   - 需要手动转换

2. **配置类型分散**
   - `pdf-core/src/config.rs` 定义了 `BaseConfig`、`CacheConfig`、`AuditConfig` 等
   - `pdf-etl/src/config.rs` 定义了自己的配置
   - 部分配置可能重复

3. **DTO 类型重复**
   - `pdf-core/src/dto.rs` 定义了 `ToolContext`、`ToolExecutionOptions`
   - `pdf-etl/src/dto.rs` 定义了自己的 DTO
   - 需要检查是否有重复

---

## 三、分阶段重构计划

### 阶段一：提取公共层

**目标**: 构建统一的 `pdf-common` crate，消除重复定义

**优先级**: 🔴 高  
**预估工作量**: 5-7 天  
**风险**: 低

---

#### 任务 1.1: 创建 pdf-common crate

**目录结构**:
```
crates/pdf-common/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs          # 统一错误类型
    ├── dto.rs            # 公共 DTO
    ├── config.rs         # 公共配置
    ├── traits.rs         # 公共 Trait
    └── utils/
        ├── mod.rs
        ├── circuit_breaker.rs  # 统一熔断器
        └── cache.rs           # 缓存工具
```

**Cargo.toml**:
```toml
[package]
name = "pdf-common"
version = "0.1.0"
edition = "2021"

[dependencies]
# 错误处理
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

# 异步
async-trait = { workspace = true }

# 时间
chrono = { workspace = true }

# 原子操作
tokio = { workspace = true }
```

**lib.rs**:
```rust
//! PDF 模块公共库
//! 
//! 提供统一的错误处理、DTO、配置和工具函数

pub mod error;
pub mod dto;
pub mod config;
pub mod traits;
pub mod utils;

// 重导出常用类型
pub use error::{PdfError, Result};
pub use dto::{ToolContext, ToolExecutionOptions};
pub use config::AppConfig;
```

---

#### 任务 1.2: 统一错误类型

**设计原则**:
1. 使用错误分类（Category）组织错误
2. 提供统一的 HTTP 状态码映射
3. 支持错误链（Error Chain）
4. 提供结构化 JSON 输出

**实现代码** (`pdf-common/src/error.rs`):

```rust
//! 统一错误类型
//! 
//! 整合 pdf-core 和 pdf-etl 的错误类型

use thiserror::Error;
use serde::{Serialize, Serializer};

/// 错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// 文件系统错误 (4xx)
    FileSystem,
    /// 提取错误 (5xx)
    Extraction,
    /// 插件错误 (5xx)
    Plugin,
    /// 配置错误 (5xx)
    Config,
    /// 验证错误 (4xx)
    Validation,
    /// 网络错误 (5xx)
    Network,
    /// 数据库错误 (5xx)
    Database,
    /// LLM 错误 (5xx)
    LLM,
}

/// 统一错误类型
#[derive(Debug, Error)]
pub enum PdfError {
    // === 文件系统错误 ===
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("File too large: {0}")]
    FileTooLarge(String),

    #[error("Corrupted file: {0}")]
    CorruptedFile(String),

    // === 提取错误 ===
    #[error("Extraction failed: {0}")]
    Extraction(String),

    #[error("Engine not found: {0}")]
    EngineNotFound(String),

    // === 插件错误 ===
    #[error("Tool registration failed: {0}")]
    ToolRegistration(String),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool already registered: {0}")]
    ToolAlreadyRegistered(String),

    // === 控制平面错误 ===
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    // === 验证错误 ===
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    // === 配置错误 ===
    #[error("Configuration error: {0}")]
    Config(String),

    // === 网络错误 ===
    #[error("HTTP error: {0}")]
    Http(String),

    // === 数据库错误 ===
    #[error("Database error: {0}")]
    Database(String),

    // === LLM 错误 ===
    #[error("LLM error: {0}")]
    LLM(String),

    // === 外部错误转换 ===
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl PdfError {
    /// 获取 HTTP 状态码
    pub fn status_code(&self) -> u16 {
        match self {
            // 4xx Client Errors
            Self::FileNotFound(_) => 404,
            Self::InvalidFileType(_) => 400,
            Self::FileTooLarge(_) => 413,
            Self::CorruptedFile(_) => 422,
            Self::ToolNotFound(_) => 404,
            Self::ToolAlreadyRegistered(_) => 409,
            Self::RateLimitExceeded(_) => 429,
            Self::Validation(_) => 400,
            Self::SchemaValidation(_) => 400,
            
            // 5xx Server Errors
            Self::Extraction(_) => 500,
            Self::EngineNotFound(_) => 500,
            Self::ToolRegistration(_) => 500,
            Self::ToolExecution(_) => 500,
            Self::CircuitBreakerOpen(_) => 503,
            Self::Timeout(_) => 408,
            Self::Config(_) => 500,
            Self::Http(_) => 500,
            Self::Database(_) => 500,
            Self::LLM(_) => 500,
            Self::Io(_) => 500,
            Self::Json(_) => 500,
        }
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::FileNotFound(_) | 
            Self::InvalidFileType(_) | 
            Self::FileTooLarge(_) | 
            Self::CorruptedFile(_) |
            Self::Io(_) => ErrorCategory::FileSystem,
            
            Self::Extraction(_) | 
            Self::EngineNotFound(_) => ErrorCategory::Extraction,
            
            Self::ToolRegistration(_) | 
            Self::ToolExecution(_) | 
            Self::ToolNotFound(_) |
            Self::ToolAlreadyRegistered(_) => ErrorCategory::Plugin,
            
            Self::RateLimitExceeded(_) | 
            Self::CircuitBreakerOpen(_) | 
            Self::Timeout(_) |
            Self::Validation(_) |
            Self::SchemaValidation(_) => ErrorCategory::Validation,
            
            Self::Config(_) => ErrorCategory::Config,
            Self::Http(_) => ErrorCategory::Network,
            Self::Database(_) => ErrorCategory::Database,
            Self::LLM(_) => ErrorCategory::LLM,
            Self::Json(_) => ErrorCategory::Config,
        }
    }

    /// 获取错误类型名称
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::FileNotFound(_) => "FileNotFoundError",
            Self::InvalidFileType(_) => "InvalidFileTypeError",
            Self::FileTooLarge(_) => "FileTooLargeError",
            Self::CorruptedFile(_) => "CorruptedFileError",
            Self::Extraction(_) => "ExtractionError",
            Self::EngineNotFound(_) => "EngineNotFoundError",
            Self::ToolRegistration(_) => "ToolRegistrationError",
            Self::ToolExecution(_) => "ToolExecutionError",
            Self::ToolNotFound(_) => "ToolNotFoundError",
            Self::ToolAlreadyRegistered(_) => "ToolAlreadyRegisteredError",
            Self::RateLimitExceeded(_) => "RateLimitExceededError",
            Self::CircuitBreakerOpen(_) => "CircuitBreakerOpenError",
            Self::Timeout(_) => "TimeoutError",
            Self::Validation(_) => "ValidationError",
            Self::SchemaValidation(_) => "SchemaValidationError",
            Self::Config(_) => "ConfigError",
            Self::Http(_) => "HttpError",
            Self::Database(_) => "DatabaseError",
            Self::LLM(_) => "LLMError",
            Self::Io(_) => "IoError",
            Self::Json(_) => "JsonError",
        }
    }

    /// 转换为 JSON 格式
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": self.error_type(),
            "message": self.to_string(),
            "category": self.category(),
            "status_code": self.status_code(),
        })
    }
}

impl Serialize for PdfError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_json().serialize(serializer)
    }
}

/// 统一 Result 类型
pub type Result<T> = std::result::Result<T, PdfError>;

// === 向后兼容转换 ===

impl From<crate::error::PdfModuleError> for PdfError {
    fn from(err: crate::error::PdfModuleError) -> Self {
        match err {
            crate::error::PdfModuleError::FileNotFound(s) => Self::FileNotFound(s),
            crate::error::PdfModuleError::InvalidFileType(s) => Self::InvalidFileType(s),
            crate::error::PdfModuleError::FileTooLarge(s) => Self::FileTooLarge(s),
            crate::error::PdfModuleError::CorruptedFile(s) => Self::CorruptedFile(s),
            crate::error::PdfModuleError::Extraction(s) => Self::Extraction(s),
            // ... 其他转换
            _ => Self::Extraction(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_code() {
        assert_eq!(PdfError::FileNotFound("test".into()).status_code(), 404);
        assert_eq!(PdfError::InvalidFileType("test".into()).status_code(), 400);
        assert_eq!(PdfError::FileTooLarge("test".into()).status_code(), 413);
        assert_eq!(PdfError::Extraction("test".into()).status_code(), 500);
    }

    #[test]
    fn test_error_category() {
        assert_eq!(PdfError::FileNotFound("test".into()).category(), ErrorCategory::FileSystem);
        assert_eq!(PdfError::Extraction("test".into()).category(), ErrorCategory::Extraction);
        assert_eq!(PdfError::ToolExecution("test".into()).category(), ErrorCategory::Plugin);
    }

    #[test]
    fn test_error_to_json() {
        let err = PdfError::FileNotFound("/path/to/file.pdf".into());
        let json = err.to_json();
        assert_eq!(json["error"], "FileNotFoundError");
        assert_eq!(json["status_code"], 404);
        assert_eq!(json["category"], "file_system");
    }
}
```

---

#### 任务 1.3: 统一 DTO 类型

**实现代码** (`pdf-common/src/dto.rs`):

```rust
//! 公共数据传输对象
//! 
//! 整合 pdf-core 和 pdf-etl 的 DTO

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工具执行上下文（统一版本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    /// 执行 ID
    pub execution_id: String,
    
    /// 组织 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    
    /// 工作流 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    
    /// 用户 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    
    /// 请求 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    
    /// 额外元数据
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ToolContext {
    /// 创建新的工具上下文
    pub fn new(execution_id: impl Into<String>) -> Self {
        Self {
            execution_id: execution_id.into(),
            org_id: None,
            workflow_id: None,
            user_id: None,
            request_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Builder: 设置组织 ID
    pub fn with_org_id(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    /// Builder: 设置工作流 ID
    pub fn with_workflow_id(mut self, workflow_id: impl Into<String>) -> Self {
        self.workflow_id = Some(workflow_id.into());
        self
    }

    /// Builder: 设置用户 ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Builder: 设置请求 ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Builder: 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// 工具执行选项（统一版本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionOptions {
    /// 是否启用流式输出
    #[serde(default)]
    pub enable_streaming: bool,
    
    /// 超时时间（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    
    /// 是否启用缓存
    #[serde(default = "default_true")]
    pub enable_cache: bool,
    
    /// 是否启用指标
    #[serde(default = "default_true")]
    pub enable_metrics: bool,
    
    /// 额外选项
    #[serde(default)]
    pub additional: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool { true }

impl Default for ToolExecutionOptions {
    fn default() -> Self {
        Self {
            enable_streaming: false,
            timeout: None,
            enable_cache: true,
            enable_metrics: true,
            additional: HashMap::new(),
        }
    }
}

impl ToolExecutionOptions {
    /// 创建新的执行选项
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: 启用流式输出
    pub fn with_streaming(mut self) -> Self {
        self.enable_streaming = true;
        self
    }

    /// Builder: 设置超时
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }

    /// Builder: 禁用缓存
    pub fn without_cache(mut self) -> Self {
        self.enable_cache = false;
        self
    }
}

/// 文本提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtractionResult {
    /// 提取的文本
    pub extracted_text: String,
    
    /// 提取元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_metadata: Option<TextExtractionMetadata>,
}

/// 文本提取元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtractionMetadata {
    /// Whisper 哈希
    pub whisper_hash: String,
    
    /// 行级元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_metadata: Option<serde_json::Value>,
}

/// 页面元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    /// 页码
    pub page_number: u32,
    
    /// 文本内容
    pub text: String,
    
    /// 边界框
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<(f64, f64, f64, f64)>,
    
    /// 行信息
    pub lines: Vec<LineInfo>,
}

/// 行信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineInfo {
    /// 边界框
    pub bbox: Vec<f64>,
    
    /// 文本内容
    pub text: String,
}

/// 结构化提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredExtractionResult {
    /// 提取的文本
    pub extracted_text: String,
    
    /// 页数
    pub page_count: u32,
    
    /// 页面列表
    pub pages: Vec<PageMetadata>,
    
    /// 提取元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_metadata: Option<TextExtractionMetadata>,
    
    /// 文件信息
    pub file_info: FileInfo,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 文件路径
    pub file_path: String,
    
    /// 文件大小（字节）
    pub file_size: u64,
    
    /// 文件大小（MB）
    pub file_size_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_context_builder() {
        let ctx = ToolContext::new("exec-123")
            .with_org_id("org-456")
            .with_user_id("user-789")
            .with_metadata("key", "value");
        
        assert_eq!(ctx.execution_id, "exec-123");
        assert_eq!(ctx.org_id, Some("org-456".to_string()));
        assert_eq!(ctx.user_id, Some("user-789".to_string()));
        assert_eq!(ctx.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_tool_execution_options_default() {
        let opts = ToolExecutionOptions::default();
        assert!(!opts.enable_streaming);
        assert!(opts.enable_cache);
        assert!(opts.enable_metrics);
    }
}
```

---

#### 任务 1.4: 统一熔断器

**实现代码** (`pdf-common/src/utils/circuit_breaker.rs`):

```rust
//! 熔断器实现
//! 
//! 提供无锁的熔断器实现，用于故障容错

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// 正常状态 - 允许请求
    Closed = 0,
    /// 熔断状态 - 拒绝请求
    Open = 1,
    /// 半开状态 - 尝试恢复
    HalfOpen = 2,
}

impl CircuitState {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 失败阈值（达到此数值后熔断）
    pub failure_threshold: u64,
    
    /// 成功阈值（半开状态下达到此数值后恢复）
    pub success_threshold: u64,
    
    /// 超时时间（熔断后等待多久尝试恢复）
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
        }
    }
}

/// 熔断器（无锁实现）
pub struct CircuitBreaker {
    /// 当前状态
    state: AtomicU8,
    
    /// 连续失败次数
    failure_count: AtomicU64,
    
    /// 连续成功次数（半开状态）
    success_count: AtomicU64,
    
    /// 最后失败时间（毫秒）
    last_failure_time: AtomicU64,
    
    /// 配置
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: AtomicU8::new(CircuitState::Closed as u8),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: AtomicU64::new(0),
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// 检查是否允许调用
    pub fn allow_call(&self) -> bool {
        let state = CircuitState::from_u8(self.state.load(Ordering::Relaxed));

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // 检查是否超过超时时间
                let elapsed = self.elapsed_since_last_failure();
                if elapsed >= self.config.timeout.as_millis() as u64 {
                    // 转换到半开状态
                    self.state.store(CircuitState::HalfOpen as u8, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// 记录成功
    pub fn record_success(&self) {
        let state = CircuitState::from_u8(self.state.load(Ordering::Relaxed));

        match state {
            CircuitState::Closed => {
                // 重置失败计数
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                // 增加成功计数
                let success = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success >= self.config.success_threshold {
                    // 转换到关闭状态
                    self.state.store(CircuitState::Closed as u8, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {}
        }
    }

    /// 记录失败
    pub fn record_failure(&self) {
        let state = CircuitState::from_u8(self.state.load(Ordering::Relaxed));

        match state {
            CircuitState::Closed => {
                // 增加失败计数
                let failure = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failure >= self.config.failure_threshold {
                    // 转换到打开状态
                    self.state.store(CircuitState::Open as u8, Ordering::Relaxed);
                    self.last_failure_time.store(current_time_ms(), Ordering::Relaxed);
                }
            }
            CircuitState::HalfOpen => {
                // 半开状态下任何失败都回到打开状态
                self.state.store(CircuitState::Open as u8, Ordering::Relaxed);
                self.last_failure_time.store(current_time_ms(), Ordering::Relaxed);
            }
            CircuitState::Open => {}
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> CircuitState {
        CircuitState::from_u8(self.state.load(Ordering::Relaxed))
    }

    /// 获取失败计数
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// 获取成功计数
    pub fn success_count(&self) -> u64 {
        self.success_count.load(Ordering::Relaxed)
    }

    /// 重置熔断器
    pub fn reset(&self) {
        self.state.store(CircuitState::Closed as u8, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
    }

    /// 计算距离上次失败的时间
    fn elapsed_since_last_failure(&self) -> u64 {
        let last = self.last_failure_time.load(Ordering::Relaxed);
        let now = current_time_ms();
        now.saturating_sub(last)
    }
}

/// 获取当前时间（毫秒）
fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::with_defaults();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_call());
    }

    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        });

        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_call());
    }

    #[test]
    fn test_circuit_breaker_half_open_after_timeout() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout: Duration::from_secs(0), // 立即转换
        });

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // 超时后应该转换到半开状态
        assert!(cb.allow_call());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_closes_on_success() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_secs(0),
        });

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // 转换到半开状态
        assert!(cb.allow_call());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // 半开状态下的成功
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
```

---

### 阶段二：解耦逻辑

**目标**: 基于 Trait 的依赖注入，降低模块耦合

**优先级**: 🟡 中  
**预估工作量**: 5-6 天  
**风险**: 中

---

#### 任务 2.1: 定义核心 Trait

**实现代码** (`pdf-common/src/traits.rs`):

```rust
//! 核心 Trait 定义
//! 
//! 提供统一的接口抽象

use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use crate::{PdfError, Result};
use crate::dto::{
    TextExtractionResult, StructuredExtractionResult, 
    ToolContext, ToolExecutionOptions
};

/// PDF 引擎 Trait
#[async_trait]
pub trait PdfEngine: Send + Sync {
    /// 引擎标识
    fn id(&self) -> &str;
    
    /// 引擎名称
    fn name(&self) -> &str;
    
    /// 引擎描述
    fn description(&self) -> &str;
    
    /// 提取文本
    async fn extract_text(&self, file_path: &Path) -> Result<TextExtractionResult>;
    
    /// 提取结构化数据
    async fn extract_structured(
        &self, 
        file_path: &Path, 
        options: &ExtractOptions
    ) -> Result<StructuredExtractionResult>;
    
    /// 获取页数
    async fn get_page_count(&self, file_path: &Path) -> Result<u32>;
}

/// 提取选项
#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    pub start_page: Option<u32>,
    pub end_page: Option<u32>,
    pub include_images: bool,
    pub include_metadata: bool,
}

/// 文件存储 Trait
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// 读取文件
    async fn read(&self, path: &str) -> Result<Vec<u8>>;
    
    /// 写入文件
    async fn write(&self, path: &str, data: &[u8]) -> Result<()>;
    
    /// 检查文件是否存在
    async fn exists(&self, path: &str) -> Result<bool>;
    
    /// 获取文件元数据
    async fn metadata(&self, path: &str) -> Result<FileMetadata>;
}

/// 文件元数据
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: std::time::SystemTime,
    pub is_file: bool,
}

/// 工具处理器 Trait
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// 执行工具
    async fn execute(
        &self, 
        context: ToolContext, 
        params: serde_json::Value,
        options: ToolExecutionOptions
    ) -> Result<serde_json::Value>;
    
    /// 获取工具元数据
    fn metadata(&self) -> ToolDefinition;
    
    /// 健康检查
    fn health_check(&self) -> bool {
        true
    }
}

/// 工具定义
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// 插件注册表 Trait
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// 注册工具
    async fn register(&mut self, handler: Arc<dyn ToolHandler>) -> Result<()>;
    
    /// 注销工具
    async fn unregister(&mut self, tool_name: &str) -> Result<()>;
    
    /// 获取工具
    fn get(&self, tool_name: &str) -> Option<Arc<dyn ToolHandler>>;
    
    /// 列出所有工具
    fn list(&self) -> Vec<ToolDefinition>;
}

/// 消息流 Trait
#[async_trait]
pub trait MessageStreamer: Send + Sync {
    /// 发送消息
    async fn stream(&self, message: ToolMessage) -> Result<()>;
}

/// 工具消息
#[derive(Debug, Clone)]
pub struct ToolMessage {
    pub tool_name: String,
    pub execution_id: String,
    pub content: serde_json::Value,
}
```

---

#### 任务 2.2: 依赖注入容器

**实现代码** (`pdf-core/src/container.rs`):

```rust
//! 服务容器
//! 
//! 提供依赖注入功能

use std::sync::Arc;
use std::collections::HashMap;
use pdf_common::traits::{PdfEngine, FileStorage, ToolHandler, PluginRegistry};
use pdf_common::{PdfError, Result};

/// 服务容器
pub struct ServiceContainer {
    /// PDF 引擎注册表
    engines: HashMap<String, Arc<dyn PdfEngine>>,
    
    /// 默认引擎
    default_engine: String,
    
    /// 回退引擎
    fallback_engine: String,
    
    /// 文件存储后端
    storage: Arc<dyn FileStorage>,
    
    /// 工具处理器注册表
    tool_handlers: HashMap<String, Arc<dyn ToolHandler>>,
    
    /// 插件注册表
    plugin_registry: Option<Arc<dyn PluginRegistry>>,
}

impl ServiceContainer {
    /// 创建新的服务容器
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
            default_engine: "lopdf".to_string(),
            fallback_engine: "pdfium".to_string(),
            storage: Arc::new(crate::storage::LocalFileStorage::default()),
            tool_handlers: HashMap::new(),
            plugin_registry: None,
        }
    }

    /// 注册 PDF 引擎
    pub fn register_engine(&mut self, engine: Arc<dyn PdfEngine>) {
        let id = engine.id().to_string();
        self.engines.insert(id, engine);
    }

    /// 注册引擎（带别名）
    pub fn register_engine_with_aliases(
        &mut self, 
        engine: Arc<dyn PdfEngine>, 
        aliases: &[&str]
    ) {
        for alias in aliases {
            self.engines.insert(alias.to_string(), engine.clone());
        }
    }

    /// 获取引擎
    pub fn get_engine(&self, id: &str) -> Option<Arc<dyn PdfEngine>> {
        self.engines.get(id).cloned()
    }

    /// 获取默认引擎
    pub fn get_default_engine(&self) -> Option<Arc<dyn PdfEngine>> {
        self.get_engine(&self.default_engine)
    }

    /// 获取回退引擎
    pub fn get_fallback_engine(&self) -> Option<Arc<dyn PdfEngine>> {
        self.get_engine(&self.fallback_engine)
    }

    /// 设置默认引擎
    pub fn set_default_engine(&mut self, engine_id: impl Into<String>) {
        self.default_engine = engine_id.into();
    }

    /// 设置回退引擎
    pub fn set_fallback_engine(&mut self, engine_id: impl Into<String>) {
        self.fallback_engine = engine_id.into();
    }

    /// 设置存储后端
    pub fn set_storage(&mut self, storage: Arc<dyn FileStorage>) {
        self.storage = storage;
    }

    /// 获取存储后端
    pub fn get_storage(&self) -> Arc<dyn FileStorage> {
        self.storage.clone()
    }

    /// 注册工具处理器
    pub fn register_tool(&mut self, handler: Arc<dyn ToolHandler>) {
        let name = handler.metadata().name;
        self.tool_handlers.insert(name, handler);
    }

    /// 获取工具处理器
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.tool_handlers.get(name).cloned()
    }

    /// 列出所有引擎
    pub fn list_engines(&self) -> Vec<String> {
        self.engines.keys().cloned().collect()
    }

    /// 列出所有工具
    pub fn list_tools(&self) -> Vec<String> {
        self.tool_handlers.keys().cloned().collect()
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}
```

---

#### 任务 2.3: 重构 PdfExtractorService

**重构前** (`pdf-core/src/extractor.rs`):

```rust
pub struct PdfExtractorService {
    engines: HashMap<String, Arc<dyn PdfEngine>>,
    default_engine: String,
    fallback_engine: String,
    validator: FileValidator,
    cache: Option<ExtractionCache>,
    keyword_extractor: KeywordExtractor,
    router: SmartRouter,
    circuit_breaker: CircuitBreaker,
}
```

**重构后**:

```rust
use pdf_common::{PdfError, Result, traits::PdfEngine};
use crate::container::ServiceContainer;

pub struct PdfExtractorService {
    /// 服务容器
    container: Arc<ServiceContainer>,
    
    /// 智能路由器
    router: SmartRouter,
    
    /// 文件验证器
    validator: FileValidator,
    
    /// 缓存
    cache: Option<ExtractionCache>,
    
    /// 关键词提取器
    keyword_extractor: KeywordExtractor,
}

impl PdfExtractorService {
    /// 创建新的提取服务
    pub fn new(container: Arc<ServiceContainer>) -> Self {
        Self {
            container,
            router: SmartRouter::new(),
            validator: FileValidator::default(),
            cache: None,
            keyword_extractor: KeywordExtractor::new(),
        }
    }

    /// 启用缓存
    pub fn with_cache(mut self, cache: ExtractionCache) -> Self {
        self.cache = Some(cache);
        self
    }

    /// 提取文本
    pub async fn extract_text(&self, file_path: &Path) -> Result<TextExtractionResult> {
        // 1. 验证文件
        self.validator.validate(file_path)?;
        
        // 2. 检查缓存
        if let Some(ref cache) = self.cache {
            if let Some(result) = cache.get(file_path).await? {
                return Ok(result);
            }
        }
        
        // 3. 选择引擎
        let engine_id = self.router.select(file_path).await?;
        
        // 4. 获取引擎
        let engine = self.container.get_engine(&engine_id)
            .ok_or_else(|| PdfError::EngineNotFound(engine_id.clone()))?;
        
        // 5. 执行提取
        let result = engine.extract_text(file_path).await?;
        
        // 6. 更新缓存
        if let Some(ref cache) = self.cache {
            cache.set(file_path, &result).await?;
        }
        
        Ok(result)
    }

    /// 提取结构化数据
    pub async fn extract_structured(
        &self, 
        file_path: &Path,
        options: &ExtractOptions
    ) -> Result<StructuredExtractionResult> {
        // 类似实现
        todo!()
    }
}
```

---

### 阶段三：减少 Boilerplate

**目标**: 使用宏和泛型精简重复代码

**优先级**: 🟢 低  
**预估工作量**: 5 天  
**风险**: 中

---

#### 任务 3.1: 创建过程宏 crate

**目录结构**:
```
crates/pdf-macros/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs        # 错误类型宏
    ├── builder.rs      # Builder 模式宏
    └── engine.rs       # 引擎注册宏
```

**Cargo.toml**:
```toml
[package]
name = "pdf-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"
```

---

#### 任务 3.2: 引擎注册宏

**实现代码** (`pdf-macros/src/engine.rs`):

```rust
//! 引擎注册宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// 引擎注册宏
/// 
/// # 示例
/// 
/// ```rust
/// register_engines!(container,
///     LopdfEngine => ["lopdf", "pymupdf", "fitz"],
///     PdfExtractEngine => ["pdf-extract", "pdfplumber"],
///     PdfiumEngine => ["pdfium"],
/// );
/// ```
#[proc_macro]
pub fn register_engines(input: TokenStream) -> TokenStream {
    // 解析输入
    // ...
    
    let expanded = quote! {
        // 生成注册代码
    };
    
    TokenStream::from(expanded)
}
```

---

#### 任务 3.3: Builder 模式宏

**实现代码** (`pdf-macros/src/builder.rs`):

```rust
//! Builder 模式宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// 自动生成 Builder 模式方法
/// 
/// # 示例
/// 
/// ```rust
/// #[derive(Builder)]
/// pub struct ToolContext {
///     pub execution_id: String,
///     pub org_id: Option<String>,
///     pub user_id: Option<String>,
/// }
/// ```
#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // 生成 Builder 方法
    let expanded = quote! {
        // ...
    };
    
    TokenStream::from(expanded)
}
```

---

### 阶段四：优化配置管理

**目标**: 统一配置结构，支持多环境

**优先级**: 🟡 中  
**预估工作量**: 3 天  
**风险**: 低

---

#### 任务 4.1: 创建统一配置

**实现代码** (`pdf-common/src/config.rs`):

```rust
//! 统一配置管理

use serde::{Deserialize, Serialize};
use std::path::Path;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 基础配置
    pub base: BaseConfig,
    
    /// 缓存配置
    pub cache: CacheConfig,
    
    /// 审计配置
    pub audit: AuditConfig,
    
    /// 存储配置
    pub storage: StorageConfig,
    
    /// 安全配置
    pub security: SecurityConfig,
    
    /// 日志配置
    pub logging: LoggingConfig,
}

impl AppConfig {
    /// 从环境变量加载
    pub fn from_env() -> crate::Result<Self> {
        Ok(Self {
            base: BaseConfig::from_env(),
            cache: CacheConfig::from_env(),
            audit: AuditConfig::from_env(),
            storage: StorageConfig::from_env(),
            security: SecurityConfig::from_env(),
            logging: LoggingConfig::from_env(),
        })
    }
    
    /// 从文件加载
    pub fn from_file(path: &Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// 验证配置
    pub fn validate(&self) -> crate::Result<()> {
        self.cache.validate()?;
        self.audit.validate()?;
        self.storage.validate()?;
        Ok(())
    }
}

/// 基础配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseConfig {
    pub server_name: String,
    pub server_version: String,
    pub environment: Environment,
}

/// 环境类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_size: usize,
    pub ttl_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<String>,
}

impl CacheConfig {
    pub fn validate(&self) -> crate::Result<()> {
        if self.max_size == 0 {
            return Err(crate::PdfError::Config("cache.max_size must be > 0".into()));
        }
        if self.ttl_seconds == 0 {
            return Err(crate::PdfError::Config("cache.ttl_seconds must be > 0".into()));
        }
        Ok(())
    }
}

// ... 其他配置结构
```

---

## 四、重构风险评估

### 4.1 高风险项

| 风险项 | 风险等级 | 影响范围 | 缓解措施 |
|--------|----------|----------|----------|
| **统一错误类型** | 🔴 高 | 所有使用错误的地方 | 使用 `From` trait 实现向后兼容 |
| **依赖注入容器** | 🔴 高 | 所有服务创建 | 提供迁移指南和示例 |

### 4.2 中风险项

| 风险项 | 风险等级 | 影响范围 | 缓解措施 |
|--------|----------|----------|----------|
| **过程宏** | 🟡 中 | 编译时间 | 提供详细的编译错误信息 |
| **配置管理** | 🟡 中 | 所有配置加载 | 提供配置迁移脚本 |

### 4.3 低风险项

| 风险项 | 风险等级 | 影响范围 | 缓解措施 |
|--------|----------|----------|----------|
| **统一 DTO** | 🟢 低 | 数据传输 | 只是移动代码位置 |
| **统一熔断器** | 🟢 低 | 故障容错 | 功能已验证 |

---

## 五、重构收益评估

### 5.1 代码质量提升

| 指标 | 当前 | 重构后 | 提升幅度 |
|------|------|--------|----------|
| 重复代码行数 | ~500 行 | ~100 行 | **-80%** |
| 错误类型数量 | 2 个 | 1 个 | **-50%** |
| 配置结构数量 | 8 个 | 5 个 | **-37%** |
| Boilerplate 代码 | ~300 行 | ~50 行 | **-83%** |

### 5.2 可维护性提升

| 原则 | 当前状态 | 重构后 |
|------|----------|--------|
| **单一职责** | ⚠️ 部分违反 | ✅ 每个 crate 职责清晰 |
| **开闭原则** | ⚠️ 部分违反 | ✅ 通过 Trait 扩展 |
| **依赖倒置** | ⚠️ 部分违反 | ✅ 依赖抽象而非具体 |
| **接口隔离** | ✅ 良好 | ✅ Trait 设计精简 |

### 5.3 性能影响

| 指标 | 影响 | 说明 |
|------|------|------|
| **Arc 克隆开销** | ⚠️ +10ns/次 | 每次服务调用增加 |
| **编译时间** | ⚠️ +5% | 过程宏增加 |
| **运行时性能** | ✅ 无影响 | 无显著变化 |

---

## 六、实施建议

### 6.1 重构策略

#### 策略 1: 渐进式重构

```
阶段 1: 创建 pdf-common
    ↓ (不修改现有代码)
阶段 2: 逐步迁移类型定义
    ↓ (标记旧 API 为 deprecated)
阶段 3: 更新依赖关系
    ↓ (删除重复代码)
阶段 4: 引入过程宏
    ↓ (优化 Boilerplate)
完成
```

#### 策略 2: 向后兼容

```rust
// 保留旧 API，标记为 deprecated
#[deprecated(since = "0.3.0", note = "Use pdf_common::PdfError instead")]
pub type PdfModuleError = pdf_common::PdfError;

#[deprecated(since = "0.3.0", note = "Use pdf_common::Result instead")]
pub type PdfResult<T> = pdf_common::Result<T>;
```

#### 策略 3: 测试保障

- ✅ 为每个重构步骤编写单元测试
- ✅ 使用集成测试验证兼容性
- ✅ 性能基准测试对比

---

### 6.2 代码审查要点

#### 要点 1: 错误处理

- [ ] 确保所有错误都能正确转换
- [ ] 验证错误链完整性
- [ ] 检查错误日志格式

#### 要点 2: Trait 设计

- [ ] 确保 Trait 方法签名合理
- [ ] 检查 `Send + Sync` 约束
- [ ] 验证异步方法正确性

#### 要点 3: 宏使用

- [ ] 确保宏展开正确
- [ ] 检查宏卫生性
- [ ] 验证编译错误信息

---

### 6.3 迁移检查清单

#### 阶段一检查清单

- [ ] 创建 `pdf-common` crate
- [ ] 实现 `PdfError` 统一错误类型
- [ ] 实现 `ToolContext` 统一 DTO
- [ ] 实现 `CircuitBreaker` 统一熔断器
- [ ] 编写单元测试
- [ ] 更新文档

#### 阶段二检查清单

- [ ] 定义核心 Trait
- [ ] 实现 `ServiceContainer`
- [ ] 重构 `PdfExtractorService`
- [ ] 更新所有服务创建代码
- [ ] 编写集成测试

#### 阶段三检查清单

- [ ] 创建 `pdf-macros` crate
- [ ] 实现引擎注册宏
- [ ] 实现 Builder 宏
- [ ] 更新所有使用 Boilerplate 的代码
- [ ] 验证宏展开正确

#### 阶段四检查清单

- [ ] 统一配置结构
- [ ] 实现配置验证
- [ ] 提供配置迁移脚本
- [ ] 更新所有配置加载代码

---

## 七、总结

### 7.1 核心发现

| 发现 | 严重程度 | 说明 |
|------|----------|------|
| **架构良好** | ✅ | 项目整体架构清晰，无循环依赖 |
| **存在重复** | ⚠️ | 4 处类型重复定义，需要统一 |
| **Boilerplate 较多** | ⚠️ | 引擎注册、错误处理等存在重复模式 |
| **耦合可控** | ✅ | 模块间耦合度适中，可通过 Trait 进一步解耦 |

### 7.2 重构价值

| 时间维度 | 价值 |
|----------|------|
| **短期** | 消除重复代码，提高代码质量 |
| **中期** | 降低维护成本，提高开发效率 |
| **长期** | 增强可扩展性，支持更多功能 |

### 7.3 下一步行动

| 优先级 | 行动 | 时间 |
|--------|------|------|
| 🔴 **立即执行** | 创建 `pdf-common` crate | 第 1 周 |
| 🔴 **立即执行** | 统一错误类型和 DTO | 第 1-2 周 |
| 🟡 **短期规划** | 实现依赖注入容器 | 第 3 周 |
| 🟡 **短期规划** | 重构核心服务 | 第 3-4 周 |
| 🟢 **中期规划** | 引入过程宏 | 第 5 周 |
| 🟢 **中期规划** | 优化配置管理 | 第 6 周 |

---

## 附录

### A. 参考资源

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)

### B. 相关文档

- [Cargo Workspace Guide](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [thiserror Documentation](https://docs.rs/thiserror)
- [async-trait Documentation](https://docs.rs/async-trait)

---

**文档版本**: 1.0  
**最后更新**: 2026-04-25  
**维护者**: Rust 架构团队
