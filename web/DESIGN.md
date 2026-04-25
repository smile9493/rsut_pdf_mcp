# PDF Module 前端交互设计文档

## 设计概述

本设计参考了 unstract 的现代化前端架构,为 PDF Module 创建了一个专业、高效、用户友好的交互界面。

## 核心设计理念

### 1. 模块化架构
- **组件化设计**: 每个UI元素都是独立的、可复用的组件
- **清晰的职责分离**: 布局、导航、内容区域各司其职
- **插件化扩展**: 支持动态加载和功能扩展

### 2. 用户体验优先
- **响应式设计**: 适配不同屏幕尺寸
- **流畅的动画**: 过渡动画提升交互体验
- **即时反馈**: 操作状态实时展示
- **无障碍支持**: 符合WCAG标准

### 3. 性能优化
- **按需加载**: 路由级别的代码分割
- **缓存策略**: 智能缓存提升响应速度
- **虚拟滚动**: 大数据列表性能优化

## 技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue 3 | 3.4.0 | 核心框架 |
| Vite | 5.0.0 | 构建工具 |
| Tailwind CSS | 3.4.0 | 样式系统 |
| Pinia | 2.1.7 | 状态管理 |
| Vue Router | 4.2.5 | 路由管理 |
| Vue I18n | 9.14.5 | 国际化 |
| Heroicons | 2.1.1 | 图标库 |

## 组件架构

### 1. 布局组件 (Layout Components)

#### PageLayout
**位置**: `src/components/PageLayout.vue`

**功能**:
- 统一的页面容器
- 集成顶部导航和侧边栏
- 支持加载和错误状态
- 可配置的内容区域

**Props**:
```typescript
{
  hideTopNav: boolean,      // 隐藏顶部导航
  hideSideNav: boolean,     // 隐藏侧边栏
  contentPadding: boolean,  // 内容区域内边距
  loading: boolean,         // 加载状态
  error: string | null      // 错误信息
}
```

**Slots**:
- `default`: 主内容区域
- `header`: 页面头部
- `footer`: 页面底部

#### SideNavBar
**位置**: `src/components/SideNavBar.vue`

**功能**:
- 可折叠的侧边栏导航
- 分组导航菜单
- 语言和主题切换
- API状态指示器
- 悬停提示(折叠状态)

**特性**:
- 本地存储记忆折叠状态
- 平滑的展开/折叠动画
- 分组导航结构(参考unstract)
- 实时健康检查

#### TopNavBar
**位置**: `src/components/TopNavBar.vue`

**功能**:
- 面包屑导航
- 全局搜索
- 通知中心
- 快捷操作
- 用户菜单

**特性**:
- 下拉菜单交互
- 通知计数徽章
- 刷新动画效果

### 2. 通知组件 (Notification Components)

#### NotificationToast
**位置**: `src/components/NotificationToast.vue`

**功能**:
- 全局通知提示
- 多类型支持(success/error/warning/info)
- 自动消失
- 进度条显示
- 堆叠管理

**API**:
```javascript
// 添加通知
toast.addToast({
  type: 'success',
  title: '操作成功',
  message: '文件已保存',
  duration: 5000
})

// 快捷方法
toast.success('成功', '操作完成')
toast.error('错误', '操作失败')
toast.warning('警告', '请注意')
toast.info('提示', '新消息')
```

## 样式系统

### 设计令牌 (Design Tokens)

**位置**: `src/assets/main.css`

#### 颜色系统
```css
/* 主色调 */
--color-primary: #0d9488;        /* 主色 */
--color-primary-light: #14b8a6;  /* 浅主色 */
--color-primary-dark: #0f766e;   /* 深主色 */

/* 背景色 */
--color-bg: #f8fafc;             /* 页面背景 */
--color-surface: #ffffff;        /* 表面背景 */
--color-surface-hover: #f1f5f9;  /* 悬停背景 */
--color-border: #e2e8f0;         /* 边框颜色 */

/* 文本色 */
--color-text-primary: #0f172a;   /* 主要文本 */
--color-text-secondary: #475569; /* 次要文本 */
--color-text-muted: #94a3b8;     /* 弱化文本 */

/* 状态色 */
--color-success: #22c55e;        /* 成功 */
--color-error: #ef4444;          /* 错误 */
--color-warning: #f59e0b;        /* 警告 */
--color-info: #3b82f6;           /* 信息 */
```

#### 间距系统
```css
/* Tailwind 间距比例 */
xs: 0.25rem   /* 4px */
sm: 0.5rem    /* 8px */
md: 1rem      /* 16px */
lg: 1.5rem    /* 24px */
xl: 2rem      /* 32px */
2xl: 3rem     /* 48px */
```

#### 字体系统
```css
/* 字体大小 */
text-micro: 0.75rem    /* 12px */
text-sm: 0.875rem      /* 14px */
text-base: 1rem        /* 16px */
text-lg: 1.125rem      /* 18px */
text-xl: 1.25rem       /* 20px */
text-h1: 2rem          /* 32px */
text-h2: 1.5rem        /* 24px */
text-display: 3rem     /* 48px */
```

### 组件样式类

#### 按钮
```css
.btn-primary    /* 主要按钮 */
.btn-secondary  /* 次要按钮 */
.btn-ghost      /* 幽灵按钮 */
.btn-sm         /* 小尺寸 */
.btn-lg         /* 大尺寸 */
```

#### 输入框
```css
.input          /* 标准输入框 */
.input-mono     /* 等宽字体输入框 */
```

#### 徽章
```css
.badge-success  /* 成功徽章 */
.badge-error    /* 错误徽章 */
.badge-warning  /* 警告徽章 */
.badge-info     /* 信息徽章 */
.badge-primary  /* 主色徽章 */
```

#### 卡片
```css
.card           /* 标准卡片 */
.card-hover     /* 可悬停卡片 */
```

### 动画系统

```css
/* 淡入 */
.animate-fade-in

/* 从右滑入 */
.animate-slide-in-right

/* 从上滑入 */
.animate-slide-in-up

/* 脉冲 */
.animate-pulse

/* 旋转 */
.animate-spin
```

## 国际化 (i18n)

### 支持语言
- 中文 (zh)
- 英文 (en)

### 使用方式
```vue
<template>
  <div>{{ t('common.loading') }}</div>
</template>

<script setup>
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>
```

### 翻译键结构
```javascript
{
  common: {},      // 通用文本
  nav: {},         // 导航文本
  dashboard: {},   // 仪表盘
  extract: {},     // 提取功能
  search: {},      // 搜索功能
  engines: {},     // 引擎管理
  performance: {}, // 性能统计
  settings: {},    // 设置
  notifications: {}// 通知
}
```

## 主题系统

### 主题类型
- `light`: 浅色主题
- `dark`: 深色主题
- `auto`: 跟随系统

### 切换方式
```javascript
import { setTheme } from './theme'

// 设置主题
setTheme('dark')

// 主题会自动保存到 localStorage
// 并应用到 document.documentElement
```

## 路由结构

```javascript
/                    // 仪表盘
/extract             // 文本提取
/search              // 关键词搜索
/batch               // 批量处理
/mcp-tools           // MCP工具
/engines             // 引擎管理
/stats               // 性能统计
/audit-logs          // 审计日志
/settings            // 系统设置
```

## 最佳实践

### 1. 组件开发
- 使用 Composition API
- Props 定义类型
- 事件使用 defineEmits
- 样式使用 scoped

### 2. 状态管理
- 使用 Pinia stores
- 避免直接修改状态
- 使用 actions 处理异步

### 3. 性能优化
- 使用 v-show 替代 v-if (频繁切换)
- 使用 computed 缓存计算结果
- 大列表使用虚拟滚动

### 4. 可访问性
- 语义化 HTML
- ARIA 属性
- 键盘导航支持
- 焦点管理

## 与 unstract 的对比

### 相似之处
1. **模块化设计**: 组件高度模块化,支持复用
2. **分组导航**: 侧边栏采用分组结构
3. **主题系统**: 支持明暗主题切换
4. **国际化**: 多语言支持
5. **状态管理**: 使用现代化状态管理方案
6. **响应式设计**: 适配不同屏幕尺寸

### 差异之处
1. **框架选择**: Vue 3 vs React 18
2. **UI库**: Tailwind CSS vs Ant Design
3. **构建工具**: Vite vs Vite (相同)
4. **状态管理**: Pinia vs Zustand
5. **图标库**: Heroicons vs Ant Design Icons

## 未来扩展

### 计划功能
1. **WebSocket 实时通信**: 实时日志和状态更新
2. **PDF 预览器**: 集成 PDF 查看功能
3. **图表可视化**: 性能数据可视化
4. **拖拽上传**: 文件拖拽上传
5. **快捷键系统**: 键盘快捷操作
6. **插件系统**: 动态功能扩展

### 性能优化
1. **虚拟滚动**: 大数据列表优化
2. **懒加载**: 图片和组件懒加载
3. **Service Worker**: 离线支持
4. **预加载**: 关键资源预加载

## 开发指南

### 启动开发服务器
```bash
cd web
npm run dev
```

### 构建生产版本
```bash
npm run build
```

### 代码规范
- 使用 ESLint 进行代码检查
- 使用 Prettier 进行代码格式化
- 遵循 Vue 官方风格指南

## 总结

本设计参考了 unstract 的优秀实践,结合 PDF Module 的实际需求,创建了一个现代化、高性能、用户友好的前端交互界面。通过模块化组件、设计令牌系统、国际化支持等特性,确保了代码的可维护性和可扩展性。

---

**设计者**: CodeArts Agent  
**参考项目**: unstract  
**创建日期**: 2026-04-24  
**版本**: v1.0.0
