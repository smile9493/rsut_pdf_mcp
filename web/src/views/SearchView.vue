<template>
  <div class="h-full overflow-auto bg-bg">
    <div class="sticky top-0 z-10 bg-bg/90 backdrop-blur-sm px-2xl py-lg border-b border-border/50">
      <div class="flex items-center justify-between">
        <h1 class="text-sm font-semibold text-text-primary">关键词搜索</h1>
        <span class="text-[11px] font-medium px-sm py-xs rounded-full" :class="canGenerate ? 'bg-success/10 text-success' : 'bg-surface-100 text-text-muted'">
          {{ canGenerate ? '已就绪' : '待输入' }}
        </span>
      </div>
    </div>

    <div class="flex h-[calc(100vh-53px)]">
      <div class="w-72 flex-shrink-0 border-r border-border bg-surface overflow-y-auto">
        <div class="p-lg space-y-xl">
          <div>
            <label class="block text-[11px] font-semibold text-text-muted uppercase tracking-wider mb-xs">PDF 文件路径</label>
            <input v-model="filePath" type="text" placeholder="/path/to/document.pdf" class="w-full px-md py-sm rounded-md border border-border bg-surface text-sm font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary">
          </div>

          <div>
            <label class="block text-[11px] font-semibold text-text-muted uppercase tracking-wider mb-xs">关键词</label>
            <textarea v-model="keywordsInput" rows="5" class="w-full px-md py-sm rounded-md border border-border bg-surface text-sm font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary resize-none" :placeholder="'每行一个关键词'" />
            <p class="text-[11px] text-text-muted mt-xs">{{ keywordCount > 0 ? `${keywordCount} 个关键词` : '每行一个关键词' }}</p>
          </div>

          <div class="space-y-sm">
            <label class="flex items-center gap-sm cursor-pointer">
              <input v-model="caseSensitive" type="checkbox" class="w-3.5 h-3.5 rounded text-primary">
              <span class="text-xs text-text-secondary">区分大小写</span>
            </label>
            <label class="flex items-center gap-sm cursor-pointer">
              <input v-model="showContext" type="checkbox" class="w-3.5 h-3.5 rounded text-primary">
              <span class="text-xs text-text-secondary">显示上下文</span>
            </label>
          </div>

          <button :disabled="!canGenerate" class="w-full py-sm rounded-md text-sm font-medium text-white bg-primary hover:bg-primary-dark disabled:opacity-40 disabled:cursor-not-allowed transition-colors" @click="copyCommand">
            复制 MCP 命令
          </button>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-w-0">
        <div class="flex-shrink-0 px-xl py-sm border-b border-border">
          <span class="text-[11px] font-semibold text-text-muted uppercase tracking-wider">命令预览</span>
        </div>
        <div class="flex-1 overflow-auto px-xl py-lg">
          <div class="rounded-md bg-surface-0 border border-border p-lg">
            <pre class="text-xs font-mono text-success whitespace-pre-wrap leading-relaxed">{{ mcpCommand }}</pre>
          </div>

          <div class="mt-xl rounded-md bg-surface-100 border border-border p-lg">
            <p class="text-[11px] font-semibold text-text-muted uppercase tracking-wider mb-md">使用说明</p>
            <ol class="space-y-sm">
              <li class="flex items-start gap-sm">
                <span class="flex-shrink-0 w-4 h-4 rounded-full bg-primary text-white text-[10px] flex items-center justify-center font-semibold">1</span>
                <span class="text-xs text-text-secondary">填写 PDF 文件路径和关键词</span>
              </li>
              <li class="flex items-start gap-sm">
                <span class="flex-shrink-0 w-4 h-4 rounded-full bg-primary text-white text-[10px] flex items-center justify-center font-semibold">2</span>
                <span class="text-xs text-text-secondary">点击「复制 MCP 命令」生成调用命令</span>
              </li>
              <li class="flex items-start gap-sm">
                <span class="flex-shrink-0 w-4 h-4 rounded-full bg-primary text-white text-[10px] flex items-center justify-center font-semibold">3</span>
                <span class="text-xs text-text-secondary">在终端执行命令获取搜索结果</span>
              </li>
            </ol>
            <div class="mt-md pt-md border-t border-border">
              <p class="text-[11px] text-text-muted flex items-start gap-xs">
                <LightBulbIcon class="w-3 h-3 flex-shrink-0 mt-xs text-warning" />
                关键词搜索通过 MCP search_keywords 工具实现，需要本地 pdf-mcp 二进制文件
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { LightBulbIcon } from '@heroicons/vue/24/outline'

const { t } = useI18n()

const filePath = ref('')
const keywordsInput = ref('')
const caseSensitive = ref(false)
const showContext = ref(true)

const keywordCount = computed(() => keywords.value.length)

const keywords = computed(() => {
  return keywordsInput.value.split('\n').map(k => k.trim()).filter(k => k)
})

const canGenerate = computed(() => {
  return filePath.value && keywords.value.length > 0
})

const mcpCommand = computed(() => {
  if (!canGenerate.value) {
    return '# 请先输入文件路径和关键词'
  }

  const args = {
    file_path: filePath.value,
    keywords: keywords.value,
    case_sensitive: caseSensitive.value,
    context_length: showContext.value ? 50 : 0
  }
  return `echo '${JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/call',
    params: { name: 'search_keywords', arguments: args }
  })}' | pdf-mcp`
})

const copyCommand = async () => {
  await navigator.clipboard.writeText(mcpCommand.value)
}
</script>
