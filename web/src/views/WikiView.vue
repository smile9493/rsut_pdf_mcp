<template>
  <div class="h-full overflow-auto bg-slate-50 dark:bg-slate-900">
    <div class="sticky top-0 z-10 bg-slate-50/90 dark:bg-slate-900/90 backdrop-blur-sm px-8 py-4 border-b border-slate-200/50 dark:border-slate-700/50">
      <div class="flex items-center justify-between">
        <h1 class="text-sm font-semibold text-slate-900 dark:text-slate-100">Wiki 管理</h1>
        <button class="px-3 py-1 text-xs font-medium text-slate-600 dark:text-slate-300 bg-white dark:bg-slate-700 border border-slate-200 dark:border-slate-600 rounded-md hover:bg-slate-50 dark:hover:bg-slate-600" @click="refreshWiki">刷新</button>
      </div>
    </div>

    <div class="flex h-[calc(100vh-53px)]">
      <div class="w-80 flex-shrink-0 border-r border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800/50 overflow-y-auto">
        <div class="p-4 space-y-5">
          <div>
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">存储结构</p>
            <div class="font-mono text-xs space-y-0.5 px-2 py-2 rounded-md bg-slate-50 dark:bg-slate-700/30">
              <div class="text-slate-900 dark:text-slate-100 font-semibold">wiki/</div>
              <div class="ml-3 flex items-center gap-1.5">
                <FolderIcon class="w-3.5 h-3.5 text-emerald-500" />
                <span class="text-emerald-600 dark:text-emerald-400">raw/</span>
                <span class="text-slate-400">物理提取产物</span>
              </div>
              <div class="ml-3 flex items-center gap-1.5">
                <FolderIcon class="w-3.5 h-3.5 text-blue-500" />
                <span class="text-blue-600 dark:text-blue-400">wiki/</span>
                <span class="text-slate-400">精炼实体页面</span>
              </div>
              <div class="ml-3 flex items-center gap-1.5">
                <FolderIcon class="w-3.5 h-3.5 text-amber-500" />
                <span class="text-amber-600 dark:text-amber-400">scheme/</span>
                <span class="text-slate-400">强类型约束</span>
              </div>
              <div class="ml-3 flex items-center gap-1.5">
                <DocumentTextIcon class="w-3.5 h-3.5 text-purple-500" />
                <span class="text-purple-600 dark:text-purple-400 font-semibold">MAP.md</span>
                <span class="text-slate-400">语义地图</span>
              </div>
            </div>
          </div>

          <div>
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">质量探测</p>
            <div class="space-y-1">
              <div class="flex items-center justify-between px-2.5 py-1.5 rounded-md bg-emerald-50 dark:bg-emerald-500/5">
                <div class="flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                  <span class="text-xs font-medium text-emerald-700 dark:text-emerald-300">Digital</span>
                </div>
                <span class="text-[11px] text-emerald-600 dark:text-emerald-400">Pdfium 本地提取</span>
              </div>
              <div class="flex items-center justify-between px-2.5 py-1.5 rounded-md bg-amber-50 dark:bg-amber-500/5">
                <div class="flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full bg-amber-500" />
                  <span class="text-xs font-medium text-amber-700 dark:text-amber-300">Scanned</span>
                </div>
                <span class="text-[11px] text-amber-600 dark:text-amber-400">VLM 多模态增强</span>
              </div>
              <div class="flex items-center justify-between px-2.5 py-1.5 rounded-md bg-blue-50 dark:bg-blue-500/5">
                <div class="flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full bg-blue-500" />
                  <span class="text-xs font-medium text-blue-700 dark:text-blue-300">Hybrid</span>
                </div>
                <span class="text-[11px] text-blue-600 dark:text-blue-400">Pdfium + VLM</span>
              </div>
            </div>
          </div>

          <div>
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">快速构建</p>
            <div class="space-y-3">
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">PDF 文件路径</label>
                <input v-model="filePath" type="text" placeholder="/path/to/document.pdf" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500">
              </div>
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">Wiki 存储路径</label>
                <input v-model="wikiBasePath" type="text" placeholder="./wiki" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500">
              </div>
              <div class="grid grid-cols-2 gap-2">
                <button :disabled="!filePath || building" class="py-1.5 rounded-md text-xs font-medium text-white bg-emerald-600 hover:bg-emerald-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors" @click="buildServerWiki">
                  <span v-if="building && buildMode === 'server'" class="flex items-center justify-center gap-1">
                    <svg class="animate-spin h-3 w-3" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" /><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" /></svg>
                    构建中
                  </span>
                  <span v-else>服务端构建</span>
                </button>
                <button :disabled="!filePath || building" class="py-1.5 rounded-md text-xs font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors" @click="buildAgentPayload">
                  <span v-if="building && buildMode === 'agent'" class="flex items-center justify-center gap-1">
                    <svg class="animate-spin h-3 w-3" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" /><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" /></svg>
                    构建中
                  </span>
                  <span v-else>本地投影</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-w-0">
        <template v-if="buildResult">
          <div class="flex-shrink-0 px-6 py-3 border-b border-slate-100 dark:border-slate-700/50 flex items-center gap-3">
            <span :class="buildResult.success ? 'text-emerald-500' : 'text-red-500'">{{ buildResult.success ? '✓' : '✗' }}</span>
            <span class="text-xs font-medium text-slate-900 dark:text-slate-100">{{ buildResult.mode === 'server' ? '服务端构建' : '本地投影' }}</span>
            <span class="text-[11px] text-slate-400">{{ buildResult.duration }}ms</span>
          </div>
          <div class="flex-1 overflow-auto px-6 py-4">
            <template v-if="buildResult.success && buildResult.mode === 'server' && buildResult.data">
              <div class="grid grid-cols-3 gap-4 mb-4">
                <div class="px-3 py-2 rounded-md bg-slate-50 dark:bg-slate-700/30">
                  <p class="text-[11px] text-slate-400 uppercase tracking-wider">Raw 路径</p>
                  <p class="text-xs font-mono text-slate-900 dark:text-slate-100 mt-0.5">{{ buildResult.data?.raw_path }}</p>
                </div>
                <div class="px-3 py-2 rounded-md bg-slate-50 dark:bg-slate-700/30">
                  <p class="text-[11px] text-slate-400 uppercase tracking-wider">MAP 路径</p>
                  <p class="text-xs font-mono text-slate-900 dark:text-slate-100 mt-0.5">{{ buildResult.data?.map_path }}</p>
                </div>
                <div class="px-3 py-2 rounded-md bg-slate-50 dark:bg-slate-700/30">
                  <p class="text-[11px] text-slate-400 uppercase tracking-wider">页数</p>
                  <p class="text-xs font-semibold text-slate-900 dark:text-slate-100 mt-0.5">{{ buildResult.data?.page_count }}</p>
                </div>
              </div>
            </template>
            <pre class="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap font-mono bg-slate-50 dark:bg-slate-700/50 rounded-md p-4 max-h-[50vh] overflow-auto leading-relaxed">{{ formatBuildResult(buildResult.data) }}</pre>
          </div>
        </template>

        <template v-else-if="buildError">
          <div class="flex-1 flex items-start justify-center pt-16 px-6">
            <div class="p-4 rounded-md bg-red-50 dark:bg-red-500/10 border border-red-200 dark:border-red-500/20 max-w-lg w-full">
              <p class="text-xs font-medium text-red-800 dark:text-red-200 mb-1">构建失败</p>
              <p class="text-xs text-red-600 dark:text-red-400">{{ buildError }}</p>
              <button class="mt-2 text-[11px] font-medium text-red-700 dark:text-red-300 hover:underline" @click="buildError = null">关闭</button>
            </div>
          </div>
        </template>

        <div v-else class="flex-1 flex items-center justify-center text-slate-400 dark:text-slate-600">
          <span class="text-sm">输入 PDF 路径后选择构建模式</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { DocumentTextIcon, FolderIcon } from '@heroicons/vue/24/outline'

const { t } = useI18n()

interface BuildResult {
  success: boolean
  mode: 'server' | 'agent'
  duration: number
  data: any
}

const filePath = ref('')
const wikiBasePath = ref('./wiki')
const building = ref(false)
const buildMode = ref<'server' | 'agent'>('server')
const buildResult = ref<BuildResult | null>(null)
const buildError = ref<string | null>(null)

const formatBuildResult = (data: any) => {
  if (typeof data === 'string') return data
  return JSON.stringify(data, null, 2)
}

const refreshWiki = () => {
  buildResult.value = null
  buildError.value = null
}

const executeMcpTool = async (toolName: string, args: Record<string, any>): Promise<{ data: any; duration: number }> => {
  const config = localStorage.getItem('mcp-config')
  const cmd = config ? JSON.parse(config).serverCommand || 'pdf-mcp' : 'pdf-mcp'
  const request = { jsonrpc: '2.0', id: Date.now(), method: 'tools/call', params: { name: toolName, arguments: args } }
  const start = Date.now()
  const response = await fetch('/api/mcp', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ command: cmd, request }) })
  if (!response.ok) throw new Error(`HTTP ${response.status}`)
  const data = await response.json()
  const text = data.result?.content?.[0]?.text || JSON.stringify(data.result)
  let parsed: any
  try { parsed = JSON.parse(text) } catch { parsed = text }
  return { data: parsed, duration: Date.now() - start }
}

const buildServerWiki = async () => {
  if (!filePath.value) return
  building.value = true; buildMode.value = 'server'; buildError.value = null; buildResult.value = null
  try {
    const { data, duration } = await executeMcpTool('extrude_to_server_wiki', { file_path: filePath.value, wiki_base_path: wikiBasePath.value })
    buildResult.value = { success: true, mode: 'server', duration, data }
  } catch (err: any) {
    buildError.value = err.message || '构建失败'
    buildResult.value = { success: false, mode: 'server', duration: 0, data: null }
  } finally { building.value = false }
}

const buildAgentPayload = async () => {
  if (!filePath.value) return
  building.value = true; buildMode.value = 'agent'; buildError.value = null; buildResult.value = null
  try {
    const { data, duration } = await executeMcpTool('extrude_to_agent_payload', { file_path: filePath.value })
    buildResult.value = { success: true, mode: 'agent', duration, data }
  } catch (err: any) {
    buildError.value = err.message || '构建失败'
    buildResult.value = { success: false, mode: 'agent', duration: 0, data: null }
  } finally { building.value = false }
}
</script>
