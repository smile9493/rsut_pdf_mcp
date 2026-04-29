# Changelog

## [0.3.0] - 2026-04-29

### 重大变更：奥卡姆剃刀重构

移除所有冗余模块，收敛至最小可运行架构：

### Removed

- **REST API**: MCP stdio 是最终契约，旁路冗余
- **Python SDK**: 官方 MCP SDK 足矣
- **多引擎抽象**: 单一 pdfium 胜任所有场景
- **缓存模块**: 大模型自带 Prompt Caching
- **熔断器**: 本地 I/O 无需网络熔断
- **SSE 传输**: stdio 是 MCP 标准
- **智能路由**: 无路由 = 无分支预测惩罚
- **插件系统**: 过度工程化
- **审计日志**: 简化为日志输出
- **存储抽象**: 无状态设计

### Added

- **Web Dashboard**: Vue 3 监控面板
  - MCP 配置页面 (服务器 + VLM API Key 配置)
  - MCP 监控仪表板 (连接状态、工具调用、日志)
  - MCP 工具测试页面
- **VLM 条件升级**: GPT-4o / Claude 3.5 Sonnet 集成
  - 扫描件检测
  - 混沌布局降级
  - `catch_unwind` FFI 安全
- **GitHub CI/CD**:
  - 多平台构建 (Windows/Linux/macOS)
  - Docker 镜像 (mcp + web)
  - Release 自动发布

### Changed

- **MCP 工具收敛至 3 个**:
  - `extract_text`: 提取纯文本
  - `extract_structured`: 提取结构化数据
  - `get_page_count`: 获取页数
- **前端类型与后端 DTO 完全对齐**
- **移除所有无用页面**: Outbox, Reconciliation, RBAC, VectorRoutes, AuditLogs

### 架构

```
AI Agent (Cursor/Claude)
    │
    └──► pdf-mcp (stdio) ──► PdfiumEngine ──► PDF

Web Dashboard (Vue 3)
    │
    ├── MCP 配置
    ├── MCP 监控
    └── 工具测试
```

---

## [0.2.0] - 2026-04-25

### Added - 插件化架构

- **Plugin Registry**: `ToolRegistry` 实现，支持分类索引和能力索引
- **ToolHandler trait**: 扩展生命周期钩子
- **PluginRegistry trait**: 统一注册/查询接口
- **ToolDispatcher trait**: 工具调度/批量执行/健康检查

### Added - 控制平面

- **AuditLogger**: 审计日志记录
- **RateLimiter**: 令牌桶限流
- **CircuitBreaker**: 熔断器状态机
- **MetricsCollector**: 执行指标收集

---

## [0.1.0] - 2024-04-22

### Added

- PDF 文本提取 (pdfium 引擎)
- MCP Server (stdio)
- 文件验证
