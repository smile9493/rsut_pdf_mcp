# PDF Web UI

Vue 3 前端界面，用于 PDF Module MCP Server 的可视化管理。

## 技术栈

- **Vue 3** - 渐进式 JavaScript 框架
- **Vite** - 下一代前端构建工具
- **Pinia** - Vue 状态管理
- **Vue Router** - 官方路由管理器
- **Tailwind CSS** - 实用优先的 CSS 框架
- **Axios** - HTTP 客户端
- **Heroicons** - 精美的 SVG 图标

## 功能特性

### 首页
- 项目介绍和核心特性展示
- 性能指标概览
- 架构图可视化

### 文本提取
- PDF 文件路径输入
- 引擎选择（支持智能路由）
- 结构化提取选项
- 实时结果显示
- 文本复制功能

### 引擎管理
- 三大引擎状态监控
- 熔断器状态显示
- 智能路由策略说明
- 引擎对比表格

### 性能统计
- 缓存命中率可视化
- 性能指标展示
- 优化效果对比
- 缓存策略说明

## 开发

### 安装依赖

```bash
npm install
```

### 启动开发服务器

```bash
npm run dev
```

访问 http://localhost:3001

### 构建生产版本

```bash
npm run build
```

### 预览生产构建

```bash
npm run preview
```

## 配置

### API 代理

开发环境下，API 请求会自动代理到 `http://localhost:8000`。

修改 `vite.config.js` 中的 proxy 配置以指向不同的后端地址：

```javascript
server: {
  proxy: {
    '/api': {
      target: 'http://your-backend-url',
      changeOrigin: true
    }
  }
}
```

### 环境变量

创建 `.env` 文件：

```env
VITE_API_BASE_URL=http://localhost:8000
```

## 设计系统

### 颜色

采用 OKLCH 色彩空间，限制色度策略：
- 主色调：蓝绿色系
- 强调色：Teal/Green（性能/成功）
- 语义色：Error/Warning/Info

### 字体

- **主字体**：Inter（系统 UI 字体）
- **等宽字体**：JetBrains Mono（代码、路径）

### 布局

- 最大内容宽度：1200px
- 响应式断点：640px / 1024px
- 间距系统：xs / sm / md / lg / xl / 2xl

## 项目结构

```
pdf-web-ui/
├── src/
│   ├── assets/          # 静态资源
│   │   └── main.css     # 全局样式
│   ├── components/      # 组件
│   │   ├── AppHeader.vue
│   │   ├── AppFooter.vue
│   │   ├── FeatureCard.vue
│   │   └── StatCard.vue
│   ├── views/           # 页面视图
│   │   ├── HomeView.vue
│   │   ├── ExtractView.vue
│   │   ├── EnginesView.vue
│   │   └── StatsView.vue
│   ├── composables/     # 组合式函数
│   │   └── useApi.js
│   ├── stores/          # Pinia 状态管理
│   │   └── pdfStore.js
│   ├── router/          # 路由配置
│   │   └── index.js
│   ├── App.vue          # 根组件
│   └── main.js          # 入口文件
├── index.html
├── package.json
├── vite.config.js
├── tailwind.config.js
└── postcss.config.js
```

## 使用说明

1. 确保 PDF Module 后端服务正在运行
2. 启动前端开发服务器
3. 在浏览器中访问 http://localhost:3001
4. 使用"文本提取"功能处理 PDF 文件
5. 在"引擎管理"中查看引擎状态
6. 在"性能统计"中查看缓存和性能数据

## 注意事项

- 文件路径需要是绝对路径
- 后端服务需要配置允许访问的文件路径
- 大文件处理可能需要较长时间
- 缓存结果会自动复用
