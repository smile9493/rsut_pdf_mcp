# Changelog

## [0.2.0] - 2026-04-25

### Added - 插件化架构

- **Plugin Registry**: `ToolRegistry` 实现，支持分类索引和能力索引
- **ToolHandler trait**: 扩展生命周期钩子 (on_register/on_unregister)、能力声明、分类、依赖
- **PluginRegistry trait**: 统一注册/查询接口
- **ToolDispatcher trait**: 工具调度/批量执行/健康检查
- **DynamicDiscovery trait**: 动态发现/热加载/目录扫描

### Added - 动态发现机制

- **CompileTimeDiscovery**: 基于 `inventory` crate 的编译期工具注册
- **RuntimeDiscovery**: 基于 `libloading` crate 的运行期动态库加载
- **UnifiedDiscovery**: 组合两种发现机制

### Added - 能力适配器

- **PdfExtractorPlugin**: PDF 文本提取适配器
- **EtlWorkflowPlugin**: ETL 工作流适配器
- **DatabasePlugin**: 数据库操作适配器
- **MiniMaxAdapterPlugin**: MiniMax LLM 适配器
- **RemotePluginAdapter**: 远程服务代理适配器

### Added - 控制平面

- **AuditLogger**: 审计日志记录，敏感字段脱敏
- **RateLimiter**: 令牌桶限流，每工具独立配置
- **CircuitBreaker**: 熔断器状态机 (Closed/Open/HalfOpen)
- **SchemaManager**: 版本化 Schema 管理，JSON Schema 验证
- **MetricsCollector**: 执行指标收集，Prometheus 格式导出

### Added - MCP Host 集成

- **McpProtocolHandler**: JSON-RPC 2.0 协议处理
- **Transport trait**: 传输层抽象 (Stdio/SSE/Http)
- **McpServer**: MCP Server 启动/关闭
- **Bootstrap**: 启动引导，自动注册内置适配器

### Added - Web 管理层

- **Plugin Management API**: GET /api/v1/tools, POST /api/v1/tools/{name}/execute
- **Audit Visualization API**: 查询/导出/统计
- **Health Check**: /health, /health/live, /health/ready
- **Prometheus Metrics**: /metrics 端点

### Added - SurrealDB 嵌入式存储

- **SurrealStore**: Schema-less JSON 存储 (内存模式)
- CRUD 操作: save/query/get/update/delete
- 审计日志存储: save_audit_log/query_audit_logs
- 原始 SurrealQL 执行: execute_query

### Added - 测试

- 6 个端到端测试 (真实 PDF 文件)
- 4 个性能测试
- 6 个压力测试

### Changed

- 扩展 `PdfModuleError` 新增 8 个错误变体
- 扩展 `dto.rs` 新增 PluginType/RetryPolicy/RateLimitConfig/ExecutionMetric 等类型
- 扩展 `ToolRegistry` 支持分类和能力索引

## [0.1.0] - 2024-04-22

### Added

- PDF 文本提取 (lopdf/pdf-extract/pdfium 三引擎)
- 智能路由与熔断降级
- REST API 服务
- MCP Server (stdio/SSE)
- 缓存 (moka)
- 关键词提取 (jieba-rs)
- 文件验证
