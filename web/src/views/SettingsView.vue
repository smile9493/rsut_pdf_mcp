<template>
  <div class="h-full overflow-auto bg-slate-50 dark:bg-slate-900">
    <div class="sticky top-0 z-10 bg-slate-50/90 dark:bg-slate-900/90 backdrop-blur-sm px-8 py-4 border-b border-slate-200/50 dark:border-slate-700/50">
      <div class="flex items-center justify-between">
        <h1 class="text-sm font-semibold text-slate-900 dark:text-slate-100">系统设置</h1>
        <div class="flex gap-2">
          <button class="px-3 py-1 text-xs font-medium text-slate-600 dark:text-slate-300 bg-white dark:bg-slate-700 border border-slate-200 dark:border-slate-600 rounded-md hover:bg-slate-50 dark:hover:bg-slate-600" @click="resetConfig">重置</button>
          <button class="px-3 py-1 text-xs font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-md" @click="saveConfig">保存</button>
        </div>
      </div>
    </div>

    <div class="flex h-[calc(100vh-53px)]">
      <div class="w-72 flex-shrink-0 border-r border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800/50 overflow-y-auto">
        <div class="p-4 space-y-5">
          <div>
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-3">MCP 服务器</p>
            <div class="space-y-3">
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">命令路径</label>
                <input v-model="mcpConfig.serverCommand" type="text" placeholder="pdf-mcp" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
                <p class="text-[11px] text-slate-400 mt-1">MCP 服务器可执行文件路径</p>
              </div>
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">超时时间</label>
                <div class="flex items-center gap-2">
                  <input v-model.number="mcpConfig.timeout" type="number" min="1000" max="300000" step="1000" class="flex-1 px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
                  <span class="text-xs text-slate-400 font-mono">ms</span>
                </div>
              </div>
            </div>
          </div>

          <div>
            <div class="flex items-center gap-2 mb-3">
              <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">VLM 视觉模型</p>
              <span class="px-1.5 py-0.5 rounded text-[9px] font-bold bg-amber-100 text-amber-700 dark:bg-amber-500/20 dark:text-amber-400">可选</span>
            </div>
            <div class="space-y-3">
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">模型提供商</label>
                <select v-model="vlmConfig.provider" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
                  <option value="">未启用</option>
                  <option value="openai">OpenAI (GPT-4o)</option>
                  <option value="anthropic">Anthropic (Claude 3.5)</option>
                  <option value="glm-4.6v">智谱AI (GLM-4.6V 高性能)</option>
                  <option value="glm-4.6v-flashx">智谱AI (GLM-4.6V-FlashX 轻量高速)</option>
                  <option value="glm-4.6v-flash">智谱AI (GLM-4.6V-Flash 免费)</option>
                  <option value="glm-ocr">智谱AI (GLM-OCR 专业OCR)</option>
                  <option value="custom">自定义</option>
                </select>
              </div>

              <div v-if="vlmConfig.provider">
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">API Key</label>
                <div class="relative">
                  <input v-model="vlmConfig.apiKey" :type="showApiKey ? 'text' : 'password'" placeholder="sk-..." class="w-full px-3 py-1.5 pr-8 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
                  <button class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600" @click="showApiKey = !showApiKey">
                    <EyeIcon v-if="!showApiKey" class="w-3.5 h-3.5" />
                    <EyeSlashIcon v-else class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              <div v-if="vlmConfig.provider === 'custom'">
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">端点地址</label>
                <input v-model="vlmConfig.endpoint" type="text" placeholder="https://api.openai.com/v1/chat/completions" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
              </div>
            </div>
          </div>

          <div v-if="vlmConfig.provider">
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-3">高级选项</p>
            <div class="space-y-3">
              <div class="grid grid-cols-2 gap-2">
                <div>
                  <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">超时</label>
                  <div class="flex items-center gap-1">
                    <input v-model.number="vlmConfig.timeout" type="number" min="5" max="120" class="flex-1 px-2 py-1 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-xs text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500">
                    <span class="text-[11px] text-slate-400">s</span>
                  </div>
                </div>
                <div>
                  <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">并发数</label>
                  <input v-model.number="vlmConfig.maxConcurrency" type="number" min="1" max="20" class="w-full px-2 py-1 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-xs text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500">
                </div>
              </div>

              <div class="space-y-1.5">
                <label class="flex items-center justify-between px-2.5 py-1.5 rounded-md bg-slate-50 dark:bg-slate-700/30 cursor-pointer">
                  <span class="text-xs text-slate-600 dark:text-slate-300">思维链</span>
                  <input v-model="vlmConfig.enableThinking" type="checkbox" class="w-3.5 h-3.5 rounded text-blue-600">
                </label>
                <label class="flex items-center justify-between px-2.5 py-1.5 rounded-md bg-slate-50 dark:bg-slate-700/30 cursor-pointer">
                  <span class="text-xs text-slate-600 dark:text-slate-300">函数调用</span>
                  <input v-model="vlmConfig.enableFunctionCall" type="checkbox" class="w-3.5 h-3.5 rounded text-blue-600">
                </label>
                <label class="flex items-center justify-between px-2.5 py-1.5 rounded-md bg-slate-50 dark:bg-slate-700/30 cursor-pointer">
                  <span class="text-xs text-slate-600 dark:text-slate-300">多模型路由</span>
                  <input v-model="vlmConfig.enableMultiModelRouting" type="checkbox" class="w-3.5 h-3.5 rounded text-blue-600">
                </label>
              </div>

              <div class="p-2.5 rounded-md bg-amber-50 dark:bg-amber-500/5 border border-amber-200 dark:border-amber-500/20">
                <p class="text-[11px] text-amber-700 dark:text-amber-400">API Key 仅保存在本地浏览器中，不会上传到服务器。</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-w-0">
        <div class="flex-shrink-0 px-6 py-3 border-b border-slate-100 dark:border-slate-700/50 flex items-center justify-between">
          <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">配置预览</span>
          <div class="flex gap-2">
            <button class="px-2 py-0.5 text-[11px] font-medium text-slate-500 hover:text-slate-700 dark:hover:text-slate-300" @click="copyConfig">复制</button>
            <button class="px-2 py-0.5 text-[11px] font-medium text-slate-500 hover:text-slate-700 dark:hover:text-slate-300" @click="downloadConfig">下载</button>
          </div>
        </div>
        <div class="flex-1 overflow-auto px-6 py-4">
          <div class="rounded-md bg-slate-900 dark:bg-slate-950 border border-slate-800 p-4">
            <pre class="text-xs font-mono text-emerald-400 whitespace-pre-wrap leading-relaxed">{{ configJson }}</pre>
          </div>

          <div class="mt-6">
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-3">可用工具</p>
            <div class="grid grid-cols-2 gap-2">
              <div v-for="tool in mcpTools" :key="tool.name" class="flex items-center gap-2 px-3 py-2 rounded-md bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                <span class="text-xs font-mono text-slate-900 dark:text-slate-100">{{ tool.name }}</span>
                <span class="text-[11px] text-slate-400 ml-auto">{{ tool.desc }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { EyeIcon, EyeSlashIcon } from '@heroicons/vue/24/outline'

const { t } = useI18n()

const showApiKey = ref(false)

const mcpTools = [
  { name: 'extract_text', desc: '提取纯文本' },
  { name: 'extract_structured', desc: '提取结构化数据' },
  { name: 'get_page_count', desc: '获取页数' },
  { name: 'search_keywords', desc: '关键词搜索' },
  { name: 'extrude_to_server_wiki', desc: '服务端Wiki构建' },
  { name: 'extrude_to_agent_payload', desc: '本地Wiki投影' }
]

const mcpConfig = ref({
  serverCommand: 'pdf-mcp',
  timeout: 30000
})

const vlmConfig = ref({
  provider: '',
  apiKey: '',
  endpoint: '',
  timeout: 30,
  maxConcurrency: 5,
  maxRetries: 3,
  retryBaseSecs: 1,
  enableThinking: true,
  enableFunctionCall: false,
  enableMultiModelRouting: true
})

const isGlmModel = computed(() => {
  return vlmConfig.value.provider?.startsWith('glm-') || vlmConfig.value.provider === 'glm'
})

watch(() => vlmConfig.value.provider, (p) => {
  if (p === 'openai') {
    vlmConfig.value.endpoint = 'https://api.openai.com/v1/chat/completions'
  } else if (p === 'anthropic') {
    vlmConfig.value.endpoint = 'https://api.anthropic.com/v1/messages'
  } else if (p?.startsWith('glm-')) {
    vlmConfig.value.endpoint = 'https://open.bigmodel.cn/api/paas/v4/chat/completions'
    if (p === 'glm-ocr') {
      vlmConfig.value.enableThinking = false
      vlmConfig.value.enableFunctionCall = false
      vlmConfig.value.enableMultiModelRouting = false
    }
  }
})

const configJson = computed(() => {
  const env: Record<string, string | boolean> = {}

  if (vlmConfig.value.provider && vlmConfig.value.apiKey) {
    env.VLM_API_KEY = vlmConfig.value.apiKey
  }
  if (vlmConfig.value.endpoint) {
    env.VLM_ENDPOINT = vlmConfig.value.endpoint
  }
  if (vlmConfig.value.provider?.startsWith('glm-')) {
    env.VLM_MODEL = vlmConfig.value.provider
    env.VLM_ENABLE_THINKING = vlmConfig.value.enableThinking
    env.VLM_ENABLE_FUNCTION_CALL = vlmConfig.value.enableFunctionCall
    env.VLM_ENABLE_MULTI_MODEL_ROUTING = vlmConfig.value.enableMultiModelRouting
    env.VLM_MAX_RETRIES = String(vlmConfig.value.maxRetries)
    env.VLM_RETRY_DELAY_BASE_SECS = String(vlmConfig.value.retryBaseSecs)
  } else if (vlmConfig.value.provider === 'anthropic') {
    env.VLM_MODEL = 'claude-3.5-sonnet'
  } else if (vlmConfig.value.provider === 'openai') {
    env.VLM_MODEL = 'gpt-4o'
  }
  if (vlmConfig.value.provider) {
    env.VLM_TIMEOUT_SECS = String(vlmConfig.value.timeout)
    env.VLM_MAX_CONCURRENCY = String(vlmConfig.value.maxConcurrency)
  }

  return JSON.stringify({
    mcpServers: {
      'pdf-module': {
        command: mcpConfig.value.serverCommand,
        env: Object.keys(env).length > 0 ? env : undefined
      }
    }
  }, null, 2)
})

function saveConfig() {
  localStorage.setItem('mcp-config', JSON.stringify(mcpConfig.value))
  localStorage.setItem('vlm-config', JSON.stringify(vlmConfig.value))
}

function resetConfig() {
  mcpConfig.value = { serverCommand: 'pdf-mcp', timeout: 30000 }
  vlmConfig.value = {
    provider: '', apiKey: '', endpoint: '', timeout: 30, maxConcurrency: 5,
    maxRetries: 3, retryBaseSecs: 1, enableThinking: true, enableFunctionCall: false, enableMultiModelRouting: true
  }
}

async function copyConfig() {
  await navigator.clipboard.writeText(configJson.value)
}

function downloadConfig() {
  const blob = new Blob([configJson.value], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url; a.download = 'claude_desktop_config.json'; a.click()
  URL.revokeObjectURL(url)
}

const savedMcp = localStorage.getItem('mcp-config')
if (savedMcp) { try { Object.assign(mcpConfig.value, JSON.parse(savedMcp)) } catch {} }
const savedVlm = localStorage.getItem('vlm-config')
if (savedVlm) { try { Object.assign(vlmConfig.value, JSON.parse(savedVlm)) } catch {} }
</script>
