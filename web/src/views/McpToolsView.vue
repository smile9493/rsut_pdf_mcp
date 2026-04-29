<template>
  <div class="p-xl">
    <div class="max-w-7xl mx-auto">
      <header class="mb-xl">
        <h1 class="text-2xl font-bold text-text-primary mb-sm">
          {{ t('mcp.title') }}
        </h1>
        <p class="text-text-secondary">
          {{ t('mcp.description') }}
        </p>
      </header>

      <div class="grid grid-cols-4 gap-lg mb-xl">
        <div class="bg-surface border border-border rounded-lg p-lg">
          <div class="text-sm text-text-secondary mb-xs">
            {{ t('mcp.monitor.status') }}
          </div>
          <div class="flex items-center gap-sm">
            <div :class="['w-3 h-3 rounded-full', mcpStatus === 'ready' ? 'bg-success' : 'bg-error']" />
            <span class="font-medium">{{ mcpStatus === 'ready' ? t('mcp.status.connected') : t('mcp.status.disconnected') }}</span>
          </div>
        </div>
        
        <div class="bg-surface border border-border rounded-lg p-lg">
          <div class="text-sm text-text-secondary mb-xs">
            {{ t('mcp.monitor.toolCount') }}
          </div>
          <div class="text-2xl font-bold text-primary">
            {{ tools.length }}
          </div>
        </div>
        
        <div class="bg-surface border border-border rounded-lg p-lg">
          <div class="text-sm text-text-secondary mb-xs">
            {{ t('mcp.monitor.totalCalls') }}
          </div>
          <div class="text-2xl font-bold text-primary">
            {{ executionLog.length }}
          </div>
        </div>
        
        <div class="bg-surface border border-border rounded-lg p-lg">
          <div class="text-sm text-text-secondary mb-xs">
            {{ t('mcp.monitor.successRate') }}
          </div>
          <div
            class="text-2xl font-bold"
            :class="successRate >= 90 ? 'text-success' : 'text-warning'"
          >
            {{ successRate }}%
          </div>
        </div>
      </div>

      <div class="grid grid-cols-3 gap-xl">
        <div class="col-span-2 space-y-lg">
          <section class="bg-surface border border-border rounded-lg p-lg">
            <h2 class="text-lg font-semibold text-text-primary mb-md">
              {{ t('mcp.selectTool') }}
            </h2>
            <div class="grid grid-cols-2 gap-md">
              <button
                v-for="tool in tools"
                :key="tool.name"
                :class="[
                  'p-md rounded-lg border text-left transition-all',
                  selectedTool?.name === tool.name
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'
                ]"
                @click="selectedTool = tool"
              >
                <div class="flex items-center gap-sm mb-xs">
                  <component
                    :is="tool.icon"
                    class="w-5 h-5 text-primary"
                  />
                  <span class="font-mono text-sm">{{ tool.name }}</span>
                </div>
                <p class="text-xs text-text-secondary">
                  {{ tool.description }}
                </p>
              </button>
            </div>
          </section>

          <section
            v-if="selectedTool"
            class="bg-surface border border-border rounded-lg p-lg"
          >
            <h2 class="text-lg font-semibold text-text-primary mb-md">
              {{ t('mcp.configureTool') }}
            </h2>
            
            <div class="space-y-md">
              <div>
                <label class="block text-sm font-medium text-text-primary mb-xs">{{ t('mcp.filePath') }}</label>
                <div class="flex gap-sm">
                  <input
                    v-model="filePath"
                    type="text"
                    :placeholder="t('mcp.filePathPlaceholder')"
                    class="flex-1 input font-mono"
                  >
                  <button
                    class="btn-secondary"
                    @click="triggerFileUpload"
                  >
                    {{ t('mcp.browse') }}
                  </button>
                  <input
                    ref="fileInput"
                    type="file"
                    accept=".pdf"
                    class="hidden"
                    @change="handleFileSelect"
                  >
                </div>
              </div>

              <div
                v-if="selectedTool.name === 'search_keywords'"
                class="grid grid-cols-2 gap-md"
              >
                <div>
                  <label class="block text-sm font-medium text-text-primary mb-xs">{{ t('mcp.keywords') }}</label>
                  <input
                    v-model="keywords"
                    type="text"
                    :placeholder="t('mcp.keywordsPlaceholder')"
                    class="input"
                  >
                </div>
                <div class="flex items-end pb-sm">
                  <label class="flex items-center gap-xs">
                    <input
                      v-model="caseSensitive"
                      type="checkbox"
                      class="rounded"
                    >
                    <span class="text-sm">{{ t('mcp.caseSensitive') }}</span>
                  </label>
                </div>
              </div>

              <button
                :disabled="loading || !filePath"
                class="btn-primary w-full"
                @click="executeTool"
              >
                <span
                  v-if="loading"
                  class="flex items-center justify-center gap-sm"
                >
                  <svg
                    class="animate-spin h-5 w-5"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      class="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      stroke-width="4"
                      fill="none"
                    />
                    <path
                      class="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                  </svg>
                  {{ t('mcp.executing') }}
                </span>
                <span v-else>{{ t('mcp.execute') }}</span>
              </button>
            </div>
          </section>

          <section
            v-if="result"
            class="bg-surface border border-border rounded-lg p-lg"
          >
            <div class="flex items-center justify-between mb-md">
              <h2 class="text-lg font-semibold text-text-primary">
                {{ t('mcp.results') }}
              </h2>
              <div class="flex items-center gap-md">
                <span class="text-sm text-text-muted">{{ result.duration }}ms</span>
                <button
                  class="btn-ghost btn-sm"
                  @click="copyResult"
                >
                  {{ t('common.copy') }}
                </button>
              </div>
            </div>

            <div
              v-if="selectedTool.name === 'search_keywords'"
              class="space-y-md"
            >
              <div class="flex items-center gap-md text-sm">
                <span>{{ t('mcp.totalMatches') }}: <strong>{{ result.data.total_matches }}</strong></span>
                <span>{{ t('mcp.pagesWithMatches') }}: <strong>{{ result.data.matches?.length || 0 }}</strong></span>
              </div>
              <div class="max-h-96 overflow-auto space-y-sm">
                <div
                  v-for="(m, idx) in result.data.matches?.slice(0, 50)"
                  :key="idx"
                  class="bg-bg rounded p-sm"
                >
                  <div class="flex items-center gap-sm mb-xs">
                    <span class="badge badge-primary text-xs">{{ m.keyword }}</span>
                    <span class="text-xs text-text-muted">{{ t('mcp.page') }} {{ m.page }}</span>
                  </div>
                  <p class="text-sm text-text-primary font-mono">
                    {{ m.context }}
                  </p>
                </div>
              </div>
            </div>

            <div
              v-else
              class="bg-bg rounded p-md max-h-96 overflow-auto"
            >
              <pre class="text-sm font-mono whitespace-pre-wrap">{{ typeof result.data === 'string' ? result.data : JSON.stringify(result.data, null, 2) }}</pre>
            </div>
          </section>

          <section
            v-if="error"
            class="bg-error/10 border border-error rounded-lg p-lg"
          >
            <div class="flex items-center gap-sm">
              <ExclamationCircleIcon class="w-5 h-5 text-error" />
              <span class="text-error font-medium">{{ t('mcp.error') }}</span>
            </div>
            <p class="mt-sm text-sm text-error">
              {{ error }}
            </p>
          </section>
        </div>

        <div class="space-y-lg">
          <section class="bg-surface border border-border rounded-lg p-lg">
            <h3 class="text-sm font-semibold text-text-secondary mb-md">
              {{ t('mcp.monitor.tools') }}
            </h3>
            <div class="space-y-sm">
              <div
                v-for="tool in tools"
                :key="tool.name"
                class="flex items-center gap-sm"
              >
                <div class="w-2 h-2 rounded-full bg-success" />
                <span class="text-sm font-mono flex-1">{{ tool.name }}</span>
              </div>
            </div>
          </section>

          <section class="bg-surface border border-border rounded-lg p-lg">
            <div class="flex items-center justify-between mb-md">
              <h3 class="text-sm font-semibold text-text-secondary">
                {{ t('mcp.monitor.logs') }}
              </h3>
              <button
                class="text-xs text-text-muted hover:text-error"
                @click="clearLog"
              >
                {{ t('common.clear') }}
              </button>
            </div>
            <div class="space-y-sm max-h-96 overflow-auto">
              <div
                v-for="(log, idx) in executionLog.slice(-20).reverse()"
                :key="idx"
                class="p-sm rounded text-xs"
                :class="log.success ? 'bg-success/10' : 'bg-error/10'"
              >
                <div class="flex items-center justify-between mb-xs">
                  <span class="font-mono font-medium">{{ log.tool }}</span>
                  <span :class="log.success ? 'text-success' : 'text-error'">{{ log.success ? '✓' : '✗' }}</span>
                </div>
                <div class="text-text-muted">
                  {{ log.duration }}ms · {{ formatTime(log.timestamp) }}
                </div>
              </div>
              <div
                v-if="executionLog.length === 0"
                class="text-xs text-text-muted text-center py-md"
              >
                {{ t('mcp.monitor.noLogs') }}
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  DocumentTextIcon,
  CubeIcon,
  DocumentIcon,
  MagnifyingGlassIcon,
  ExclamationCircleIcon
} from '@heroicons/vue/24/outline'

const { t } = useI18n()

interface Tool {
  name: string
  description: string
  icon: any
}

interface LogEntry {
  tool: string
  success: boolean
  duration: number
  timestamp: number
  error?: string
}

const tools: Tool[] = [
  { name: 'extract_text', description: '提取纯文本', icon: DocumentTextIcon },
  { name: 'extract_structured', description: '提取结构化数据 + bbox', icon: CubeIcon },
  { name: 'get_page_count', description: '获取页数', icon: DocumentIcon },
  { name: 'search_keywords', description: '关键词搜索', icon: MagnifyingGlassIcon }
]

const mcpStatus = ref<'ready' | 'error'>('ready')
const selectedTool = ref<Tool | null>(null)
const filePath = ref('')
const keywords = ref('')
const caseSensitive = ref(false)
const loading = ref(false)
const result = ref<any>(null)
const error = ref<string | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const executionLog = ref<LogEntry[]>([])

const successRate = computed(() => {
  if (executionLog.value.length === 0) return 100
  const success = executionLog.value.filter(l => l.success).length
  return Math.round((success / executionLog.value.length) * 100)
})

const triggerFileUpload = () => fileInput.value?.click()

const handleFileSelect = (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) filePath.value = file.name
}

const formatTime = (ts: number) => {
  const d = new Date(ts)
  return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}:${d.getSeconds().toString().padStart(2, '0')}`
}

const loadLog = () => {
  const saved = localStorage.getItem('mcp-execution-log')
  if (saved) {
    try { executionLog.value = JSON.parse(saved) } catch {}
  }
}

const saveLog = (entry: LogEntry) => {
  executionLog.value.push(entry)
  localStorage.setItem('mcp-execution-log', JSON.stringify(executionLog.value.slice(-100)))
}

const clearLog = () => {
  executionLog.value = []
  localStorage.removeItem('mcp-execution-log')
}

const executeTool = async () => {
  if (!filePath.value || !selectedTool.value) return

  loading.value = true
  error.value = null
  result.value = null

  const startTime = Date.now()

  try {
    const config = localStorage.getItem('mcp-config')
    const mcpCommand = config ? JSON.parse(config).serverCommand || 'pdf-mcp' : 'pdf-mcp'
    
    const args: Record<string, any> = { file_path: filePath.value }
    if (selectedTool.value.name === 'search_keywords') {
      if (!keywords.value) throw new Error(t('mcp.noKeywords'))
      args.keywords = keywords.value.split(',').map(k => k.trim())
      args.case_sensitive = caseSensitive.value
    }

    const cmd = {
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/call',
      params: { name: selectedTool.value.name, arguments: args }
    }

    console.log('[MCP] Executing:', cmd)
    
    const response = await fetch('/api/mcp', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ command: mcpCommand, request: cmd })
    })

    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    
    const data = await response.json()
    const text = data.result?.content?.[0]?.text || JSON.stringify(data.result)
    
    result.value = {
      data: selectedTool.value.name === 'search_keywords' ? JSON.parse(text) : text,
      duration: Date.now() - startTime
    }

    saveLog({
      tool: selectedTool.value.name,
      success: true,
      duration: result.value.duration,
      timestamp: Date.now()
    })
  } catch (err: any) {
    error.value = err.message || t('mcp.executionFailed')
    saveLog({
      tool: selectedTool.value!.name,
      success: false,
      duration: Date.now() - startTime,
      timestamp: Date.now(),
      error: err.message
    })
  } finally {
    loading.value = false
  }
}

const copyResult = async () => {
  if (!result.value) return
  const text = typeof result.value.data === 'string' 
    ? result.value.data 
    : JSON.stringify(result.value.data, null, 2)
  await navigator.clipboard.writeText(text)
}

onMounted(() => {
  loadLog()
  const saved = localStorage.getItem('mcp-config')
  if (saved) {
    try {
      const cfg = JSON.parse(saved)
      mcpStatus.value = cfg.serverCommand ? 'ready' : 'error'
    } catch {}
  }
})
</script>
