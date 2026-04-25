# PDF Module 前端界面

基于 unstract 设计模式的现代化前端交互界面。

## 特性

- 🎨 **现代化设计** - 参考 unstract 的专业UI设计
- 🌓 **主题切换** - 支持浅色/深色/自动主题
- 🌍 **国际化** - 支持中文和英文
- 📱 **响应式** - 适配不同屏幕尺寸
- ⚡ **高性能** - Vite 构建,快速加载
- 🎯 **模块化** - 组件化设计,易于维护

## 快速开始

### 安装依赖
```bash
npm install
```

### 启动开发服务器
```bash
npm run dev
```

### 构建生产版本
```bash
npm run build
```

## 项目结构

```
web/
├── src/
│   ├── components/          # 组件目录
│   │   ├── PageLayout.vue   # 页面布局
│   │   ├── SideNavBar.vue   # 侧边栏导航
│   │   ├── TopNavBar.vue    # 顶部导航栏
│   │   └── NotificationToast.vue  # 通知组件
│   ├── views/               # 页面视图
│   ├── assets/              # 静态资源
│   │   └── main.css         # 全局样式
│   ├── locales/             # 国际化文件
│   │   ├── zh.js            # 中文
│   │   └── en.js            # 英文
│   ├── App.vue              # 应用入口
│   ├── i18n.js              # 国际化配置
│   └── theme.js             # 主题配置
├── DESIGN.md                # 设计文档
└── package.json
```

## 核心组件

### PageLayout
统一的页面布局容器,集成顶部导航和侧边栏。

```vue
<template>
  <PageLayout>
    <router-view />
  </PageLayout>
</template>
```

### SideNavBar
可折叠的侧边栏导航,支持分组菜单。

**特性**:
- 折叠/展开动画
- 分组导航结构
- 语言和主题切换
- API状态指示

### TopNavBar
顶部导航栏,提供快捷操作和通知功能。

**特性**:
- 面包屑导航
- 全局搜索
- 通知中心
- 用户菜单

### NotificationToast
全局通知提示组件。

```javascript
// 使用示例
import { ref } from 'vue'
const toast = ref(null)

// 显示成功通知
toast.value.success('操作成功', '文件已保存')

// 显示错误通知
toast.value.error('操作失败', '请重试')
```

## 样式系统

### 设计令牌
使用 CSS 变量定义设计令牌,支持主题切换。

```css
/* 颜色 */
--color-primary: #0d9488;
--color-bg: #f8fafc;
--color-text-primary: #0f172a;

/* 间距 */
p-xs  /* 4px */
p-sm  /* 8px */
p-md  /* 16px */
p-lg  /* 24px */
p-xl  /* 32px */
```

### 组件样式类
```css
/* 按钮 */
.btn-primary
.btn-secondary
.btn-ghost

/* 输入框 */
.input
.input-mono

/* 徽章 */
.badge-success
.badge-error
.badge-warning

/* 卡片 */
.card
.card-hover
```

## 国际化

支持中文和英文切换。

```vue
<template>
  <div>{{ t('common.loading') }}</div>
</template>

<script setup>
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>
```

## 主题系统

支持三种主题模式:
- `light` - 浅色主题
- `dark` - 深色主题
- `auto` - 跟随系统

```javascript
import { setTheme } from './theme'
setTheme('dark')
```

## 技术栈

- **Vue 3** - 渐进式JavaScript框架
- **Vite** - 下一代前端构建工具
- **Tailwind CSS** - 实用优先的CSS框架
- **Pinia** - Vue状态管理
- **Vue Router** - Vue路由
- **Vue I18n** - Vue国际化
- **Heroicons** - 精美的SVG图标

## 设计参考

本设计参考了 [unstract](https://github.com/unstract/unstract) 的优秀实践:
- 模块化组件架构
- 分组导航设计
- 主题系统实现
- 国际化支持
- 响应式布局

详细设计文档请查看 [DESIGN.md](./DESIGN.md)

## 开发指南

### 添加新页面
1. 在 `src/views/` 创建页面组件
2. 在 `src/router/index.js` 添加路由
3. 在 `src/components/SideNavBar.vue` 添加导航项
4. 在 `src/locales/` 添加翻译文本

### 添加新组件
1. 在 `src/components/` 创建组件
2. 使用 Composition API
3. 添加 Props 类型定义
4. 使用 scoped 样式

### 添加新样式
1. 在 `src/assets/main.css` 添加样式类
2. 使用 Tailwind 工具类
3. 遵循设计令牌规范

## 许可证

MIT
