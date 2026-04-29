<template>
  <div class="h-full overflow-auto bg-slate-50 dark:bg-slate-900">
    <div class="sticky top-0 z-10 bg-slate-50/90 dark:bg-slate-900/90 backdrop-blur-sm px-8 py-4 border-b border-slate-200/50 dark:border-slate-700/50">
      <div class="flex items-center justify-between">
        <h1 class="text-sm font-semibold text-slate-900 dark:text-slate-100">关键词搜索</h1>
        <span class="text-[11px] font-medium px-2 py-0.5 rounded-full" :class="canGenerate ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400' : 'bg-slate-100 text-slate-400 dark:bg-slate-700 dark:text-slate-500'">
          {{ canGenerate ? '已就绪' : '待输入' }}
        </span>
      </div>
    </div>

    <div class="flex h-[calc(100vh-53px)]">
      <div class="w-72 flex-shrink-0 border-r border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800/50 overflow-y-auto">
        <div class="p-4 space-y-5">
          <div>
            <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">PDF 文件路径</label>
            <input v-model="filePath" type="text" placeholder="/path/to/document.pdf" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
          </div>

          <div>
            <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">关键词</label>
            <textarea v-model="keywordsInput" rows="5" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 resize-none" :placeholder="'每行一个关键词'" />
            <p class="text-[11px] text-slate-400 mt-1">{{ keywordCount > 0 ? `${keywordCount} 个关键词` : '每行一个关键词' }}</p>
          </div>

          <div class="space-y-2">
            <label class="flex items-center gap-2 cursor-pointer">
              <input v-model="caseSensitive" type="checkbox" class="w-3.5 h-3.5 rounded text-blue-600">
              <span class="text-xs text-slate-600 dark:text-slate-300">区分大小写</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input v-model="showContext" type="checkbox" class="w-3.5 h-3.5 rounded text-blue-600">
              <span class="text-xs text-slate-600 dark:text-slate-300">显示上下文</span>
            </label>
          </div>

          <button :disabled="!canGenerate" class="w-full py-2 rounded-md text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors" @click="copyCommand">
            复制 MCP 命令
          </button>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-w-0">
        <div class="flex-shrink-0 px-6 py-3 border-b border-slate-100 dark:border-slate-700/50">
          <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">命令预览</span>
        </div>
        <div class="flex-1 overflow-auto px-6 py-4">
          <div class="rounded-md bg-slate-900 dark:bg-slate-950 border border-slate-800 p-4">
            <pre class="text-xs font-mono text-emerald-400 whitespace-pre-wrap leading-relaxed">{{ mcpCommand }}</pre>
          </div>

          <div class="mt-6 rounded-md bg-slate-50 dark:bg-slate-700/30 border border-slate-200 dark:border-slate-700 p-4">
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-3">使用说明</p>
            <ol class="space-y-2">
              <li class="flex items-start gap-2">
                <span class="flex-shrink-0 w-4 h-4 rounded-full bg-blue-600 text-white text-[10px] flex items-center justify-center font-semibold">1</span>
                <span class="text-xs text-slate-600 dark:text-slate-300">填写 PDF 文件路径和关键词</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="flex-shrink-0 w-4 h-4 rounded-full bg-blue-600 text-white text-[10px] flex items-center justify-center font-semibold">2</span>
                <span class="text-xs text-slate-600 dark:text-slate-300">点击「复制 MCP 命令」生成调用命令</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="flex-shrink-0 w-4 h-4 rounded-full bg-blue-600 text-white text-[10px] flex items-center justify-center font-semibold">3</span>
                <span class="text-xs text-slate-600 dark:text-slate-300">在终端执行命令获取搜索结果</span>
              </li>
            </ol>
            <div class="mt-3 pt-3 border-t border-slate-200 dark:border-slate-600">
              <p class="text-[11px] text-slate-400 flex items-start gap-1.5">
                <LightBulbIcon class="w-3 h-3 flex-shrink-0 mt-0.5 text-amber-500" />
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
