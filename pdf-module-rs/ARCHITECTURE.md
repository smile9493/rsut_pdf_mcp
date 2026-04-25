# MCP Plugin-Driven Architecture

本文档描述 PDF Module 的插件化架构设计，基于 MCP (Model Context Protocol) 协议实现动态工具加载与编排。

## 设计原则

1. **开闭原则**: 对扩展开放，对修改关闭。新工具通过插件注册，无需修改核心代码。
2. **依赖倒置**: 核心层定义 Trait 接口，适配器层提供具体实现。
3. **单一职责**: 每个组件只负责一个关注点（审计、限流、熔断等）。
4. **零成本抽象**: 编译期发现使用 `inventory` crate，运行时无额外开销。

## 架构分层

```
┌─────────────────────────────────────────────────────────┐
│                    接入层 (Entry)                        │
│  REST API │ MCP stdio │ MCP SSE │ PyO3                  │
├─────────────────────────────────────────────────────────┤
│                    编排层 (Orchestration)                │
│  McpServer │ McpProtocolHandler │ Bootstrap              │
├─────────────────────────────────────────────────────────┤
│                    注册层 (Registry)                     │
│  ToolRegistry │ PluginRegistry trait │ MetadataCache     │
├─────────────────────────────────────────────────────────┤
│                    发现层 (Discovery)                    │
│  CompileTimeDiscovery │ RuntimeDiscovery │ Unified       │
├─────────────────────────────────────────────────────────┤
│                    适配层 (Adapters)                     │
│  PDF │ ETL │ Database │ MiniMax │ Remote                │
├─────────────────────────────────────────────────────────┤
│                    控制层 (Control Plane)                │
│  Audit │ RateLimit │ CircuitBreak │ Schema │ Metrics    │
├─────────────────────────────────────────────────────────┤
│                    存储层 (Storage)                      │
│  FileStorage │ SurrealDB │ Cache                        │
└─────────────────────────────────────────────────────────┘
```

## 核心 Trait 定义

### ToolHandler

工具执行的标准接口，所有插件必须实现：

```rust
#[async_trait]
pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn definition(&self) -> &ToolDefinition;
    fn spec(&self) -> &ToolSpec;
    fn variables(&self) -> &RuntimeVariables;

    async fn execute(
        &self,
        params: HashMap<String, Value>,
        streamer: Option<&mut dyn MessageStreamer>,
    ) -> PdfResult<ToolExecutionResult>;

    fn validate_params(&self, params: &HashMap<String, Value>) -> PdfResult<()>;
    fn metadata(&self) -> ExecutionMetadata;

    // 生命周期钩子 (带默认实现)
    async fn on_register(&self) -> PdfResult<()> { Ok(()) }
    async fn on_unregister(&self) -> PdfResult<()> { Ok(()) }
    fn capabilities(&self) -> Vec<String> { vec![] }
    fn category(&self) -> String { "general".to_string() }
    fn dependencies(&self) -> Vec<String> { vec![] }
}
```

### PluginRegistry

工具注册与查询接口：

```rust
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    async fn register(&self, tool: Arc<dyn ToolHandler>) -> PdfResult<()>;
    async fn unregister(&self, name: &str) -> PdfResult<()>;
    async fn get(&self, name: &str) -> PdfResult<Arc<dyn ToolHandler>>;
    async fn is_registered(&self, name: &str) -> bool;
    async fn list_tools(&self) -> Vec<Arc<dyn ToolHandler>>;
    async fn list_definitions(&self) -> Vec<ToolDefinition>;
    async fn query_by_capability(&self, capability: &str) -> Vec<Arc<dyn ToolHandler>>;
    async fn query_by_category(&self, category: &str) -> Vec<Arc<dyn ToolHandler>>;
    async fn count(&self) -> usize;
    async fn clear(&self) -> PdfResult<()>;
}
```

### ControlPlane

控制平面统一接口：

```rust
#[async_trait]
pub trait ControlPlane: Send + Sync {
    async fn log_audit(&self, log: AuditLog) -> PdfResult<()>;
    async fn query_audit_logs(&self, filter: &AuditFilter) -> PdfResult<Vec<AuditLog>>;
    async fn register_schema(&self, schema: SchemaDefinition) -> PdfResult<()>;
    async fn get_schema(&self, name: &str) -> PdfResult<Option<SchemaDefinition>>;
    async fn check_rate_limit(&self, tool_name: &str) -> bool;
    async fn record_metrics(&self, metric: &ExecutionMetric);
    async fn get_health_status(&self) -> HealthStatus;
}
```

## 动态发现机制

### 编译期发现 (inventory)

```rust
// 在插件 crate 中声明
use pdf_core::plugin::compile_time_discovery::submit_tool_registration;

submit_tool_registration!(MyToolFactory);

// 在主程序中收集
let tools = pdf_core::plugin::compile_time_discovery::discover_compile_time();
```

### 运行期发现 (libloading)

```rust
let mut discovery = RuntimeDiscovery::new("./plugins");
discovery.scan_plugins().await?;

// 加载特定插件
let tool = discovery.load_plugin("my_plugin.so").await?;
registry.register(tool).await?;
```

### 统一发现器

```rust
let config = UnifiedDiscoveryConfig::builder()
    .plugin_dir("./plugins")
    .enable_compile_time(true)
    .enable_runtime(true)
    .build();

let discovery = UnifiedDiscovery::new(config);
let tools = discovery.discover().await?;
```

## 控制平面详解

### 限流器 (RateLimiter)

基于令牌桶算法，每工具独立配置：

```rust
let limiter = RateLimiter::new();
limiter.configure("pdf_extract".to_string(), RateLimitConfig {
    requests_per_second: 100,
    burst_size: 10,
});

if limiter.check("pdf_extract") {
    // 允许请求
} else {
    // 限流拒绝
}
```

### 熔断器 (CircuitBreaker)

三状态机：Closed → Open → HalfOpen → Closed

```rust
let cb = CircuitBreaker::new(CircuitBreakerConfig {
    failure_threshold: 5,    // 5 次失败后熔断
    success_threshold: 3,    // 3 次成功后恢复
    timeout_ms: 30000,       // 30 秒冷却期
});

if cb.allow_call() {
    match execute().await {
        Ok(r) => cb.record_success(),
        Err(e) => cb.record_failure(),
    }
}
```

### Schema 管理器 (SchemaManager)

版本化 Schema 管理，支持基础 JSON Schema 验证：

```rust
let sm = SchemaManager::new();
sm.register(SchemaDefinition::new(
    "pdf_extract".to_string(),
    "1.0.0".to_string(),
    json!({"type": "object", "required": ["pdf_path"]}),
)).await?;

sm.validate("pdf_extract", &params).await?;
```

### 指标收集器 (MetricsCollector)

Prometheus 文本格式导出：

```rust
let metrics = MetricsCollector::new();
metrics.record_execution(&metric).await;
metrics.increment_cache_hit();

// Prometheus 端点
let output = metrics.export_prometheus().await;
```

## SurrealDB 存储

Schema-less 嵌入式数据库，适合存储不同 PDF 模板的动态 JSON 输出：

```rust
let store = SurrealStore::new(SurrealStoreConfig {
    path: PathBuf::from("data/pdf_mcp.db"),
    namespace: "mcp".to_string(),
    database: "etl".to_string(),
}).await?;

// CRUD 操作
store.save_etl_result("results", "doc_001", data).await?;
store.get("results:doc_001").await?;
store.update("results:doc_001", new_data).await?;
store.delete("results:doc_001").await?;

// 查询
store.query("results", "pages > 50").await?;
store.execute_query("SELECT count() FROM results GROUP ALL").await?;
```

## 扩展指南

### 添加新工具

1. 实现 `ToolHandler` trait
2. 使用 `submit_tool_registration!` 宏注册 (编译期)
3. 或打包为动态库放入 `plugins/` 目录 (运行期)

```rust
struct MyTool { /* ... */ }

#[async_trait]
impl ToolHandler for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "My custom tool" }
    // ... 实现其他方法
}

submit_tool_registration!(MyTool);
```

### 添加新适配器

在 `pdf-core/src/plugin/adapters/` 下创建新文件，实现 `ToolHandler`，然后在 `mod.rs` 中导出。

### 添加新控制平面组件

1. 在 `pdf-core/src/control/` 下创建新模块
2. 在 `ControlPlane` trait 中添加新方法
3. 在 `mod.rs` 中导出
