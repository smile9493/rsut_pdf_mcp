# CSS 硬编码问题修复总结

## 已完成的工作

### 1. 问题分析
- 识别了7个Vue组件文件中存在CSS硬编码问题
- 主要问题：使用了硬编码的颜色类（slate-*, emerald-*, red-*, blue-*, amber-*等）而非设计令牌

### 2. 创建修复指南
- 创建了 [CSS_REFACTOR_GUIDE.md](./web/CSS_REFACTOR_GUIDE.md)
- 详细记录了颜色、间距、边框等设计令牌的映射规则

### 3. 添加自定义工具类
在 [main.css](./web/src/assets/main.css) 中添加了以下工具类：
- `bg-success/10` - 成功状态背景色
- `bg-error/10` - 错误状态背景色
- `bg-warning/10` - 警告状态背景色
- `bg-primary/10` - 主要状态背景色
- `bg-primary/15` - 主要状态背景色（15%透明度）

### 4. 已修复的组件

#### ✅ HomeView.vue
- 修复了所有硬编码的颜色类
- 替换了所有硬编码的间距类
- 统一使用设计令牌

#### ✅ ExtractView.vue
- 修复了文件上传区域的样式
- 修复了表单元素的样式
- 修复了结果显示区域的样式

#### ✅ SearchView.vue
- 修复了表单输入区域的样式
- 修复了命令预览区域的样式
- 修复了使用说明区域的样式

## 待完成的工作

### 待修复的组件
以下组件仍存在CSS硬编码问题，需要按照相同的模式进行修复：

1. **McpToolsView.vue** - MCP工具测试页面
2. **WikiView.vue** - Wiki页面
3. **SettingsView.vue** - 设置页面
4. **BatchProcessView.vue** - 批处理页面

### 修复方法
参考已修复的组件，按照以下步骤进行：

1. 替换颜色类：
   - `bg-slate-*` → `bg-surface-*` 或 `bg-bg`
   - `text-slate-*` → `text-text-*`
   - `border-slate-*` → `border-border`
   - `bg-emerald-*` → `bg-success`
   - `bg-red-*` → `bg-error`
   - `bg-blue-*` → `bg-primary`
   - `bg-amber-*` → `bg-warning`

2. 替换间距类：
   - `px-8` → `px-2xl`
   - `py-5` → `py-lg`
   - `gap-4` → `gap-lg`
   - 等等...

3. 测试主题切换：
   - 确保在 light 和 dark 主题下都能正常显示

## 修复效果

### 优点
1. **主题一致性**：所有组件现在使用统一的设计令牌，主题切换更加一致
2. **可维护性**：修改主题颜色只需要调整设计令牌，不需要逐个修改组件
3. **视觉一致性**：所有页面使用相同的颜色和间距系统

### 示例对比

**修复前：**
```vue
<div class="bg-slate-50 dark:bg-slate-900">
  <h1 class="text-slate-900 dark:text-slate-100">标题</h1>
</div>
```

**修复后：**
```vue
<div class="bg-bg">
  <h1 class="text-text-primary">标题</h1>
</div>
```

## 下一步建议

1. **继续修复剩余组件**：按照相同模式修复其他4个组件
2. **验证样式效果**：在浏览器中测试所有页面的显示效果
3. **测试主题切换**：确保 light/dark 主题切换正常工作
4. **代码审查**：检查是否有遗漏的硬编码样式
5. **文档更新**：更新开发规范，要求新代码必须使用设计令牌

## 相关文件

- [CSS重构指南](./web/CSS_REFACTOR_GUIDE.md)
- [设计令牌定义](./web/src/assets/main.css)
- [Tailwind配置](./web/tailwind.config.js)
