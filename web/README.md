# PDF MCP Web Dashboard

Vue 3 + TypeScript + Tailwind CSS 前端监控面板。

## 功能

- **MCP 配置** (`/mcp-config`): 配置 MCP 服务器 + VLM API Key
- **MCP 监控** (`/mcp-monitor`): 实时监控客户端连接、工具调用、日志
- **MCP 工具** (`/mcp-tools`): 测试 MCP 工具调用
- **文本提取** (`/extract`): PDF 文本提取
- **关键词搜索** (`/search`): PDF 关键词搜索
- **批量处理** (`/batch`): 批量 PDF 处理

## 开发

```bash
npm install
npm run dev
```

访问 http://localhost:5173

## 构建

```bash
npm run build
```

## 技术栈

- Vue 3.5+
- TypeScript
- Pinia (状态管理)
- Vue Router
- Tailwind CSS
- vue-i18n (国际化)
- Heroicons (图标)

## 项目结构

```
src/
├── views/
│   ├── McpConfigView.vue    # MCP 配置
│   ├── McpMonitorView.vue   # MCP 监控
│   ├── McpToolsView.vue     # MCP 工具测试
│   ├── ExtractView.vue      # 文本提取
│   ├── SearchView.vue       # 关键词搜索
│   └── BatchProcessView.vue # 批量处理
├── stores/
│   ├── mcpStore.ts          # MCP 状态
│   └── pdfStore.ts          # PDF 状态
├── composables/
│   ├── useApi.ts            # API 客户端
│   └── useAsyncAction.ts    # 统一异步处理
├── types/
│   └── api.ts               # TypeScript 类型定义
└── locales/
    ├── zh.js                # 中文
    └── en.js                # 英文
```

## 配置说明

### MCP 配置页面

可配置：
- MCP 服务器命令/路径
- VLM 提供商 (OpenAI/Anthropic)
- API Key
- API 端点
- 超时和并发设置

生成配置可直接下载用于 Cursor/Claude Desktop。
