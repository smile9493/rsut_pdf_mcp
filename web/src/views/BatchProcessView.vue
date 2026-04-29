<template>
  <div class="h-full overflow-auto bg-slate-50 dark:bg-slate-900">
    <div class="sticky top-0 z-10 bg-slate-50/90 dark:bg-slate-900/90 backdrop-blur-sm px-8 py-4 border-b border-slate-200/50 dark:border-slate-700/50">
      <div class="flex items-center justify-between">
        <h1 class="text-sm font-semibold text-slate-900 dark:text-slate-100">批量处理</h1>
        <button v-if="results.length > 0" class="px-3 py-1 text-xs font-medium text-slate-600 dark:text-slate-300 bg-white dark:bg-slate-700 border border-slate-200 dark:border-slate-600 rounded-md hover:bg-slate-50 dark:hover:bg-slate-600" @click="exportResults">导出结果</button>
      </div>
    </div>

    <div class="flex h-[calc(100vh-53px)]">
      <div class="w-72 flex-shrink-0 border-r border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800/50 overflow-y-auto">
        <div class="p-4 space-y-5">
          <div>
            <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-2">选择文件</label>
            <div
              class="border border-dashed rounded-md p-5 text-center cursor-pointer transition-colors"
              :class="isDragging ? 'border-blue-400 bg-blue-50 dark:bg-blue-500/10' : 'border-slate-200 dark:border-slate-600 hover:border-blue-400'"
              @dragover.prevent="isDragging = true"
              @dragleave.prevent="isDragging = false"
              @drop.prevent="handleDrop"
              @click="triggerFileUpload"
            >
              <input ref="fileInput" type="file" accept=".pdf" multiple class="hidden" @change="handleFileSelect">
              <CloudArrowUpIcon class="w-6 h-6 text-slate-300 dark:text-slate-600 mx-auto mb-1.5" />
              <p class="text-xs text-slate-500">拖拽或点击上传</p>
              <p class="text-[11px] text-slate-400 mt-0.5">支持多文件</p>
            </div>

            <div v-if="files.length > 0" class="mt-3 space-y-1 max-h-40 overflow-auto">
              <div v-for="(file, idx) in files" :key="idx" class="flex items-center justify-between px-2.5 py-1.5 rounded-md bg-slate-50 dark:bg-slate-700/50">
                <div class="flex items-center gap-1.5 min-w-0 flex-1">
                  <DocumentTextIcon class="w-3.5 h-3.5 text-slate-400 flex-shrink-0" />
                  <span class="text-xs text-slate-700 dark:text-slate-300 truncate">{{ file.name }}</span>
                </div>
                <button class="text-slate-400 hover:text-red-500 flex-shrink-0 ml-1" @click="files.splice(idx, 1)">
                  <XMarkIcon class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </div>

          <div>
            <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-2">操作类型</label>
            <select v-model="options.operation" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
              <option value="extract_text">提取文本</option>
              <option value="extract_structured">提取结构化数据</option>
              <option value="get_info">获取文件信息</option>
            </select>
          </div>

          <div>
            <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-2">引擎</label>
            <select v-model="options.adapter" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
              <option :value="null">自动选择</option>
              <option v-for="adapter in adapters" :key="adapter" :value="adapter">{{ adapter }}</option>
            </select>
          </div>

          <div>
            <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-2">并行数</label>
            <input v-model.number="options.parallelJobs" type="number" min="1" max="10" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
          </div>

          <button
            :disabled="files.length === 0 || processing"
            class="w-full py-2 rounded-md text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            @click="startBatch"
          >
            <template v-if="processing">
              <svg class="animate-spin h-3.5 w-3.5 inline mr-1.5" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" /><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" /></svg>
              处理中
            </template>
            <template v-else>开始处理</template>
          </button>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-w-0">
        <template v-if="processing || results.length > 0">
          <div class="flex-shrink-0 px-6 py-3 border-b border-slate-100 dark:border-slate-700/50">
            <div class="flex items-center gap-6 text-xs text-slate-500">
              <span>{{ completedCount }} / {{ files.length }}</span>
              <div v-if="results.length > 0" class="flex items-center gap-4">
                <span class="text-emerald-600 dark:text-emerald-400">{{ successCount }} 成功</span>
                <span v-if="failedCount > 0" class="text-red-600 dark:text-red-400">{{ failedCount }} 失败</span>
                <span>均耗 {{ avgDuration }}ms</span>
              </div>
              <div v-if="processing" class="ml-auto flex items-center gap-2">
                <div class="w-32 h-1.5 bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden">
                  <div class="h-full bg-blue-500 transition-all duration-300" :style="{ width: `${progress}%` }" />
                </div>
                <span>{{ progress.toFixed(0) }}%</span>
              </div>
            </div>
          </div>

          <div class="flex-1 overflow-auto">
            <div class="divide-y divide-slate-100 dark:divide-slate-700/50">
              <div v-for="r in results" :key="r.file" class="flex items-center justify-between px-6 py-2.5 hover:bg-slate-50 dark:hover:bg-slate-700/20 transition-colors">
                <div class="flex items-center gap-2.5">
                  <span :class="r.status === 'success' ? 'text-emerald-500' : 'text-red-500'">{{ r.status === 'success' ? '✓' : '✗' }}</span>
                  <span class="text-xs text-slate-900 dark:text-slate-100">{{ r.file }}</span>
                </div>
                <div class="flex items-center gap-3">
                  <span class="text-[11px] text-slate-400">{{ r.duration }}ms</span>
                  <button class="text-[11px] font-medium text-blue-600 dark:text-blue-400 hover:underline" @click="viewResult(r)">查看</button>
                </div>
              </div>
            </div>
          </div>
        </template>
        <div v-else class="flex-1 flex items-center justify-center text-slate-400 dark:text-slate-600">
          <span class="text-sm">上传文件后开始批量处理</span>
        </div>
      </div>
    </div>

    <div v-if="selectedResult" class="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4" @click="selectedResult = null">
      <div class="bg-white dark:bg-slate-800 rounded-lg max-w-4xl w-full max-h-[80vh] overflow-hidden shadow-xl" @click.stop>
        <div class="flex items-center justify-between px-5 py-3 border-b border-slate-200 dark:border-slate-700">
          <h3 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{{ selectedResult.file }}</h3>
          <button class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-300" @click="selectedResult = null">
            <XMarkIcon class="w-4 h-4" />
          </button>
        </div>
        <div class="p-5 overflow-auto max-h-[calc(80vh-52px)]">
          <pre class="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap font-mono bg-slate-50 dark:bg-slate-700/50 rounded-md p-4">{{ selectedResult.data }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { CloudArrowUpIcon, DocumentTextIcon, XMarkIcon } from '@heroicons/vue/24/outline'
import { pdfApi } from '@/composables/useApi'

const { t } = useI18n()

const files = ref([])
const adapters = ref([])
const isDragging = ref(false)
const processing = ref(false)
const results = ref([])
const selectedResult = ref(null)
const fileInput = ref(null)

const options = ref({
  operation: 'extract_text',
  adapter: null,
  parallelJobs: 3,
  outputFormat: 'json'
})

const completedCount = computed(() => results.value.length)
const successCount = computed(() => results.value.filter(r => r.status === 'success').length)
const failedCount = computed(() => results.value.filter(r => r.status === 'failed').length)
const progress = computed(() => {
  if (files.value.length === 0) return 0
  return (completedCount.value / files.value.length) * 100
})
const avgDuration = computed(() => {
  if (results.value.length === 0) return 0
  return Math.round(results.value.reduce((sum, r) => sum + r.duration, 0) / results.value.length)
})

const triggerFileUpload = () => { fileInput.value?.click() }
const handleFileSelect = (e) => { files.value.push(...Array.from(e.target.files)) }
const handleDrop = (e) => {
  isDragging.value = false
  files.value.push(...Array.from(e.dataTransfer.files).filter(f => f.name.endsWith('.pdf')))
}

const loadAdapters = async () => {
  try { adapters.value = await pdfApi.listAdapters() }
  catch { adapters.value = ['lopdf', 'pdf-extract', 'pdfium'] }
}

const startBatch = async () => {
  if (files.value.length === 0) return
  processing.value = true
  results.value = []
  for (const file of files.value) {
    const startTime = Date.now()
    try {
      let result
      switch (options.value.operation) {
        case 'extract_text': result = await pdfApi.extractTextFromFile(file, options.value.adapter); break
        case 'extract_structured': result = await pdfApi.extractStructuredFromFile(file, options.value.adapter); break
        case 'get_info': result = await pdfApi.getInfo(file); break
      }
      results.value.push({ file: file.name, status: 'success', duration: Date.now() - startTime, data: result })
    } catch (error) {
      results.value.push({ file: file.name, status: 'failed', duration: Date.now() - startTime, error: error.message })
    }
  }
  processing.value = false
}

const viewResult = (result) => { selectedResult.value = result }

const exportResults = () => {
  const blob = new Blob([JSON.stringify(results.value.map(r => ({ file: r.file, status: r.status, duration: r.duration, data: r.data })), null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url; a.download = `batch-results-${Date.now()}.json`; a.click()
  URL.revokeObjectURL(url)
}

onMounted(() => { loadAdapters() })
</script>
