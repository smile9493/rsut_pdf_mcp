# CSS 硬编码问题修复指南

## 问题概述

项目中存在大量硬编码的CSS类名，未使用设计令牌（Design Tokens），导致：
- 主题切换不一致
- 样式维护困难
- 视觉一致性差

## 设计令牌映射规则

### 颜色映射

#### 背景色 (Background)
```
bg-slate-50 / dark:bg-slate-900     → bg-bg
bg-slate-100                        → bg-surface-100
bg-white / dark:bg-slate-800        → bg-surface
bg-slate-200                        → bg-surface-200

bg-emerald-50 / dark:bg-emerald-500/10  → bg-success/10 (需要自定义)
bg-red-50 / dark:bg-red-500/10          → bg-error/10 (需要自定义)
bg-blue-50 / dark:bg-blue-500/10        → bg-primary/10 (需要自定义)
bg-amber-50 / dark:bg-amber-500/10      → bg-warning/10 (需要自定义)
```

#### 文本色 (Text)
```
text-slate-900 / dark:text-slate-100    → text-text-primary
text-slate-600 / dark:text-slate-300    → text-text-secondary
text-slate-400 / dark:text-slate-500    → text-text-muted
text-slate-500                          → text-text-secondary

text-emerald-700 / dark:text-emerald-400  → text-success
text-red-700 / dark:text-red-400          → text-error
text-blue-600 / dark:text-blue-400        → text-primary
text-amber-600                            → text-warning
```

#### 边框色 (Border)
```
border-slate-200 / dark:border-slate-700    → border-border
border-slate-100                            → border-border (或更浅)
border-slate-600                            → border-surface-300

border-emerald-400  → border-success
border-red-400      → border-error
border-blue-400     → border-primary
border-amber-400    → border-warning
```

### 间距映射

```
px-8 / py-5     → px-2xl / py-lg
px-6 / py-4     → px-xl / py-md
px-4 / py-3     → px-lg / py-sm
px-3 / py-2     → px-md / py-xs
px-2 / py-1     → px-sm / py-xs

gap-6   → gap-xl
gap-4   → gap-lg
gap-3   → gap-md
gap-2   → gap-sm
```

### 特殊状态类

#### 成功状态
```
bg-emerald-50 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400
→ bg-success/10 text-success (需要添加自定义类)
```

#### 错误状态
```
bg-red-50 text-red-700 dark:bg-red-500/10 dark:text-red-400
→ bg-error/10 text-error (需要添加自定义类)
```

#### 主要操作
```
bg-blue-600 hover:bg-blue-700
→ bg-primary hover:bg-primary-dark
```

## 需要添加的自定义工具类

在 `main.css` 的 `@layer utilities` 中添加：

```css
/* 状态背景色 */
.bg-success\/10 {
  background-color: rgba(16, 185, 129, 0.1);
}

.bg-error\/10 {
  background-color: rgba(239, 68, 68, 0.1);
}

.bg-warning\/10 {
  background-color: rgba(245, 158, 11, 0.1);
}

.bg-primary\/10 {
  background-color: rgba(13, 148, 136, 0.1);
}

.bg-primary\/15 {
  background-color: rgba(13, 148, 136, 0.15);
}
```

## 修复优先级

1. **高优先级**：HomeView.vue (主要页面)
2. **中优先级**：ExtractView.vue, SearchView.vue, BatchProcessView.vue
3. **低优先级**：其他视图和组件

## 注意事项

1. 保持暗色模式支持：所有替换都要考虑 `dark:` 前缀
2. 测试主题切换：修复后要测试 light/dark 主题切换
3. 视觉一致性：确保修复后的视觉效果与原来一致
4. 渐进式修复：可以逐个文件修复，不必一次性全部修改
