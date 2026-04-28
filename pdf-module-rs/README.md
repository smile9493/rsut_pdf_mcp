# PDF Module Rust

高性能 PDF 文本提取服务，支持 REST API 和 MCP (Model Context Protocol) 双接口，基于插件化架构实现动态工具加载与编排。

## 特性

- **多引擎支持**: lopdf、pdf-extract、pdfium 三种提取引擎
- **智能路由**: 根据文档特征自动选择最优引擎
- **熔断降级**: 自动故障转移，保证服务可用性
- **双接口**: REST API + MCP (stdio/SSE)
- **插件化架构**: 动态工具注册/发现/编排，支持编译期和运行期插件加载
- **控制平面**: 审计日志、限流、熔断、Schema 管理、Prometheus 指标
- **SurrealDB 嵌入式存储**: Schema-less JSON 存储，适配任意 PDF 模板输出
- **Web 管理层**: REST API 管理工具/审计/健康检查/Prometheus 指标
- **高性能**: Rust 实现，内存占用低，速度快

## 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp

# 构建
cargo build --release
```

### REST API 服务

```bash
# 启动服务
./target/release/pdf-rest --host 0.0.0.0 --port 8000

# 健康检查
curl http://localhost:8000/api/v1/x2text/health

# 提取文本
curl -X POST http://localhost:8000/api/v1/x2text/extract \
  -F "file=@document.pdf"

# 指定引擎
curl -X POST http://localhost:8000/api/v1/x2text/extract \
  -F "file=@document.pdf" \
  -F "adapter=pdfium"

# 获取结构化数据
curl -X POST http://localhost:8000/api/v1/x2text/extract-json \
  -F "file=@document.pdf"

# 列出可用引擎
curl http://localhost:8000/api/v1/x2text/adapters
```

### MCP Server

```bash
# stdio 模式 (用于 Claude Desktop 等)
./target/release/pdf-mcp serve --transport stdio

# SSE 模式 (HTTP)
./target/release/pdf-mcp serve --transport sse --port 8001
```

**Claude Desktop 配置** (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "pdf": {
      "command": "/path/to/pdf-mcp",
      "args": ["serve", "--transport", "stdio"]
    }
  }
}
```

## API 端点

### REST API (PDF 提取)

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/v1/x2text/health` | GET | 健康检查 |
| `/api/v1/x2text/extract` | POST | 提取纯文本 |
| `/api/v1/x2text/extract-json` | POST | 提取结构化 JSON |
| `/api/v1/x2text/info` | POST | 获取 PDF 信息 |
| `/api/v1/x2text/adapters` | GET | 列出可用引擎 |
| `/api/v1/x2text/cache/stats` | GET | 缓存统计 |

### REST API (插件管理)

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/v1/tools` | GET | 列出所有注册工具 |
| `/api/v1/tools/{name}` | GET | 获取工具详情 |
| `/api/v1/tools/{name}/execute` | POST | 执行工具 |
| `/api/v1/audit-logs` | GET | 查询审计日志 |
| `/api/v1/audit-logs/stats` | GET | 审计统计 |
| `/api/v1/audit-logs/export` | GET | 导出审计日志 (CSV/JSON) |
| `/api/v1/schemas` | GET/POST | Schema 管理 |
| `/api/v1/health` | GET | 健康检查 |

### 健康检查端点

| 端点 | 说明 |
|------|------|
| `/health` | 完整健康检查 |
| `/health/live` | 存活探针 (Kubernetes liveness) |
| `/health/ready` | 就绪探针 (Kubernetes readiness) |
| `/metrics` | Prometheus 指标 |

### MCP Tools

| 工具 | 说明 |
|------|------|
| `extract_text` | 提取纯文本 |
| `extract_structured` | 提取结构化数据 |
| `get_page_count` | 获取页数 |
| `search_keywords` | 关键词搜索 |
| `extract_keywords` | 自动提取高频词 |
| `list_adapters` | 列出可用引擎 |
| `cache_stats` | 缓存统计 |

## 插件化架构

### 架构概览

```
┌─────────────────────────────────────────────────┐
│                  MCP Host                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │ Protocol │  │Transport │  │ Bootstrap│      │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘      │
│       └──────────────┼────────────┘             │
│                      ▼                          │
│  ┌──────────────────────────────────────┐       │
│  │         Plugin Registry              │       │
│  │  ┌─────────┐ ┌──────────┐ ┌──────┐ │       │
│  │  │Category │ │Capability│ │Cache │ │       │
│  │  │ Index   │ │  Index   │ │      │ │       │
│  │  └─────────┘ └──────────┘ └──────┘ │       │
│  └──────────────────┬───────────────────┘       │
│                     ▼                           │
│  ┌──────────────────────────────────────┐       │
│  │        Tool Dispatcher               │       │
│  └──────────────────┬───────────────────┘       │
│                     ▼                           │
│  ┌──────────────────────────────────────┐       │
│  │     Capability Adapters              │       │
│  │  PDF│ETL│Database│MiniMax│Remote     │       │
│  └──────────────────┬───────────────────┘       │
│                     ▼                           │
│  ┌──────────────────────────────────────┐       │
│  │        Control Plane                 │       │
│  │  Audit│RateLimit│CircuitBreak│Schema│Metrics│
│  └──────────────────────────────────────┘       │
└─────────────────────────────────────────────────┘
```

### 核心 Trait

| Trait | 说明 |
|-------|------|
| `ToolHandler` | 工具执行标准接口 (execute/validate/capabilities/category) |
| `PluginRegistry` | 工具注册/查询/生命周期管理 |
| `ToolDispatcher` | 工具调度/批量执行/健康检查 |
| `DynamicDiscovery` | 动态发现/热加载/目录扫描 |
| `ControlPlane` | 审计/限流/熔断/Schema/健康状态 |

### 内置适配器

| 适配器 | 类别 | 能力 |
|--------|------|------|
| `PdfExtractorPlugin` | extraction | file_input, text_output, cacheable |
| `EtlWorkflowPlugin` | etl | file_input, json_output, llm_required |
| `DatabasePlugin` | storage | database, transaction |
| `MiniMaxAdapterPlugin` | ai | remote, llm, chat |
| `RemotePluginAdapter` | integration | remote, http, grpc |

### 动态发现机制

- **编译期发现** (`inventory` crate): 零成本抽象，编译时注册
- **运行期发现** (`libloading` crate): 动态加载 `.so`/`.dylib`/`.dll` 插件
- **统一发现器** (`UnifiedDiscovery`): 组合两种机制，优先编译期

### 控制平面组件

| 组件 | 说明 |
|------|------|
| `AuditLogger` | 审计日志记录，敏感字段脱敏，保留期管理 |
| `RateLimiter` | 令牌桶限流，每工具独立配置，突发支持 |
| `CircuitBreaker` | 熔断器状态机 (Closed→Open→HalfOpen→Closed) |
| `SchemaManager` | 版本化 Schema 管理，基础 JSON Schema 验证 |
| `MetricsCollector` | 执行指标收集，Prometheus 文本格式导出 |

### SurrealDB 嵌入式存储

使用 SurrealDB (内存模式) 作为 Schema-less 存储，适合存储不同 PDF 模板产生的动态 JSON 结构。

```rust
use pdf_core::database::{SurrealStore, SurrealStoreConfig};

// 初始化
let store = SurrealStore::with_defaults().await?;

// 保存 ETL 结果
let data = serde_json::json!({"pdf_path": "test.pdf", "pages": 100});
store.save_etl_result("etl_results", "doc_001", data).await?;

// 查询
let results = store.query("etl_results", "pages > 50").await?;

// 审计日志
store.save_audit_log(&audit_log).await?;
let logs = store.query_audit_logs(Some("pdf_extract"), 20).await?;
```

## PDF 引擎

| 引擎 | 特点 | 适用场景 |
|------|------|----------|
| `lopdf` | 布局感知 | 通用文档 |
| `pdf-extract` | 最快 | 简单文档 |
| `pdfium` | 最兼容 | 复杂/特殊编码 |

**别名支持**:
- `pymupdf`, `fitz` → lopdf
- `pdfplumber` → pdf-extract

## 智能路由

系统根据文档特征自动选择最优引擎：

1. **小文档 + 简单布局** → `pdf-extract` (最快)
2. **特殊编码 (CIDFont/Type3)** → `pdfium` (最兼容)
3. **默认** → `lopdf` (布局感知)

## 熔断降级

- 连续 5 次失败后熔断
- 60 秒冷却后尝试恢复
- 自动降级到 `pdfium` 备用引擎

## Docker 部署

```bash
# 构建镜像
docker build -t pdf-module .

# 运行
docker run -p 8000:8000 pdf-module

# 或使用 docker-compose
docker-compose up -d
```

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `PDF_SERVER_HOST` | `127.0.0.1` | 监听地址 |
| `PDF_SERVER_PORT` | `8000` | 监听端口 |
| `PDF_DEFAULT_ADAPTER` | `lopdf` | 默认引擎 |
| `PDF_CACHE_ENABLED` | `true` | 启用缓存 |
| `PDF_CACHE_MAX_SIZE` | `1000` | 缓存最大条目 |
| `PDF_CACHE_TTL_SECONDS` | `3600` | 缓存 TTL |
| `PDF_MAX_FILE_SIZE_MB` | `200` | 最大文件大小 |

## 项目结构

```
pdf-module-rs/
├── Cargo.toml              # Workspace 配置
├── Dockerfile              # Docker 构建文件
├── docker-compose.yml      # Docker Compose
├── .env.example            # 环境变量示例
└── crates/
    ├── pdf-common/          # 公共库 (V3.1 规范化编程)
    │   ├── src/
    │   │   ├── vector.rs           # 向量存储 Trait
    │   │   ├── lancedb_storage.rs  # LanceDB 实现
    │   │   ├── logic_storage.rs    # 逻辑存储 Trait
    │   │   ├── outbox.rs           # Outbox 模式
    │   │   ├── storage.rs          # 通用存储 Trait
    │   │   ├── blocking.rs         # CPU 任务隔离
    │   │   ├── compensation.rs     # 透明补偿
    │   │   ├── backpressure.rs     # 自适应背压
    │   │   ├── reconciliation.rs   # 对账 Worker
    │   │   ├── dual_store.rs       # 双库锚定写入
    │   │   ├── rbac.rs             # RBAC 权限
    │   │   ├── score_calibrator.rs # 分数校准
    │   │   ├── progressive_router.rs # 渐进式路由
    │   │   ├── hybrid_search.rs    # 混合检索流
    │   │   ├── ast_chunker.rs      # AST 分块
    │   │   ├── confidence_interceptor.rs # 置信度拦截
    │   │   ├── graph_storage.rs    # 图关系存储
    │   │   ├── prompt_hint.rs      # Prompt Hint
    │   │   ├── metrics.rs          # 指标记录
    │   │   ├── vector_gc.rs        # 向量 GC
    │   │   ├── config.rs           # 配置
    │   │   ├── error.rs            # 错误处理
    │   │   ├── dto.rs              # 数据模型
    │   │   └── traits.rs           # 公共 Trait
    │   ├── tests/
    │   │   └── integration_tests.rs # 17 个集成测试
    │   └── Cargo.toml
    ├── pdf-core/           # 核心库
    │   ├── src/
    │   │   ├── engine/     # PDF 引擎实现
    │   │   ├── cache.rs    # 缓存
    │   │   ├── config.rs   # 配置
    │   │   ├── dto.rs      # 数据模型
    │   │   ├── error.rs    # 错误处理
    │   │   ├── extractor.rs # 服务编排 + 智能路由 + 熔断
    │   │   ├── keyword.rs  # 关键词提取
    │   │   ├── validator.rs # 文件验证
    │   │   ├── audit/      # 审计日志
    │   │   │   ├── mod.rs
    │   │   │   └── audit_log.rs
    │   │   ├── plugin/     # 插件化架构
    │   │   │   ├── mod.rs
    │   │   │   ├── tool_handler.rs    # ToolHandler trait
    │   │   │   ├── registry.rs        # ToolRegistry 实现
    │   │   │   ├── registry_trait.rs  # PluginRegistry trait
    │   │   │   ├── dispatcher.rs      # ToolDispatcher trait
    │   │   │   ├── discovery.rs       # DynamicDiscovery trait
    │   │   │   ├── metadata_cache.rs  # 元数据缓存 (moka)
    │   │   │   ├── compile_time_discovery.rs  # 编译期发现 (inventory)
    │   │   │   ├── runtime_discovery.rs       # 运行期发现 (libloading)
    │   │   │   ├── unified_discovery.rs       # 统一发现器
    │   │   │   └── adapters/          # 能力适配器
    │   │   │       ├── pdf_extractor.rs
    │   │   │       ├── etl_workflow.rs
    │   │   │       ├── database.rs
    │   │   │       ├── minimax.rs
    │   │   │       └── remote.rs
    │   │   ├── control/    # 控制平面
    │   │   │   ├── mod.rs
    │   │   │   ├── control_plane.rs   # ControlPlane trait
    │   │   │   ├── audit_logger.rs
    │   │   │   ├── rate_limiter.rs
    │   │   │   ├── circuit_breaker.rs
    │   │   │   ├── schema_manager.rs
    │   │   │   └── metrics_collector.rs
    │   │   └── database/   # SurrealDB 存储
    │   │       ├── mod.rs
    │   │       └── surreal_store.rs
    │   └── Cargo.toml
    ├── pdf-rest/           # REST API 服务
    │   ├── src/
    │   │   ├── main.rs     # 入口
    │   │   ├── routes.rs   # PDF 提取路由
    │   │   └── api/        # 插件管理 API
    │   │       ├── mod.rs          # 工具管理
    │   │       ├── audit.rs        # 审计可视化
    │   │       ├── health.rs       # 健康检查
    │   │       └── metrics.rs      # Prometheus 指标
    │   └── Cargo.toml
    ├── pdf-mcp/            # MCP Server
    │   ├── src/
    │   │   ├── main.rs     # 入口
    │   │   ├── server.rs   # stdio 传输
    │   │   ├── sse.rs      # SSE 传输
    │   │   ├── protocol/   # MCP 协议处理
    │   │   ├── transport/  # 传输层抽象
    │   │   ├── mcp_server.rs  # MCP Server
    │   │   └── bootstrap.rs  # 启动引导
    │   └── Cargo.toml
    ├── pdf-etl/            # ETL 工作流
    │   └── Cargo.toml
    └── pdf-python/         # PyO3 绑定 (可选)
        └── Cargo.toml
```

## V3.1 规范化编程

项目已完成 V3.1 "终局生产版" 规范化编程改造，详细规划参见 [V3.1 开发任务规划书](../docs/V3.1_DEVELOPMENT_PLAN.md)。

V3.1 在 `pdf-common` crate 中新增 20 个模块，覆盖向量存储、双库锚定、RBAC、混合检索、AST 分块、图存储等核心能力。详见根目录 README 的 [V3.1 规范化编程](../README.md#v31-规范化编程) 章节。

---

## 测试

```bash
# pdf-common 全部测试 (含 17 个集成测试)
cargo test -p pdf-common

# Clippy 检查
cargo clippy -p pdf-common --all-targets

# 端到端测试 (使用真实 PDF)
cargo test --package pdf-core --test e2e_test -- --nocapture

# 性能测试
cargo test --package pdf-core --test perf_test -- --nocapture

# 压力测试
cargo test --package pdf-core --test stress_test -- --nocapture

# 集成测试
cargo test --package pdf-core --test integration_test

# 全部测试
cargo test --package pdf-core --test e2e_test --test perf_test --test stress_test --test integration_test
```

### 测试覆盖

| 测试类型 | 数量 | 覆盖范围 |
|----------|------|----------|
| 端到端 | 6 | PDF 提取、插件注册、控制平面、SurrealDB、全链路、审计日志 |
| 性能 | 4 | PDF 提取、插件注册、SurrealDB 读写、控制平面 |
| 压力 | 6 | 并发提取、并发写入、高容量注册、熔断恢复、限流突发、指标高负载 |

### 性能基准

| 组件 | 指标 |
|------|------|
| PDF 提取 (缓存命中) | <1ms |
| Plugin Registry (100 工具) | 注册/查询 <1ms |
| SurrealDB 写入 | ~1.7M writes/sec |
| SurrealDB 读取 | ~3.5M reads/sec |
| Rate Limiter (10K checks) | 16ms |
| Circuit Breaker (10K checks) | <1ms |
| Metrics (50K records) | 22ms |
| 并发 PDF 提取 (10 路) | 1ms, 100% 成功 |
| 并发 DB 写入 (50 路) | 15ms, 100% 成功 |

## 许可证

MIT
