<template>
  <div class="h-full overflow-auto bg-slate-50 dark:bg-slate-900">
    <div class="sticky top-0 z-10 bg-slate-50/90 dark:bg-slate-900/90 backdrop-blur-sm px-8 py-4 border-b border-slate-200/50 dark:border-slate-700/50">
      <div class="flex items-center justify-between">
        <h1 class="text-sm font-semibold text-slate-900 dark:text-slate-100">文本提取</h1>
        <button v-if="result" class="px-3 py-1 text-xs font-medium text-slate-600 dark:text-slate-300 bg-white dark:bg-slate-700 border border-slate-200 dark:border-slate-600 rounded-md hover:bg-slate-50 dark:hover:bg-slate-600" @click="copyText">
          {{ copied ? '已复制' : '复制' }}
        </button>
      </div>
    </div>

    <div class="flex h-[calc(100vh-53px)]">
      <div class="w-72 flex-shrink-0 border-r border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800/50 overflow-y-auto">
        <div class="p-4 space-y-5">
          <div>
            <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-2">PDF 文件</label>
            <div
              class="border border-dashed rounded-md p-5 text-center cursor-pointer transition-colors"
              :class="selectedFile ? 'border-emerald-400 bg-emerald-50/50 dark:bg-emerald-500/5' : 'border-slate-200 dark:border-slate-600 hover:border-blue-400'"
              @click="fileInput?.click()"
              @dragover.prevent
              @drop.prevent="handleDrop"
            >
              <input ref="fileInput" type="file" accept=".pdf" class="hidden" @change="handleFileSelect">
              <template v-if="selectedFile">
                <div class="flex items-center justify-center gap-2">
                  <DocumentTextIcon class="w-5 h-5 text-emerald-600 dark:text-emerald-400" />
                  <div class="text-left min-w-0">
                    <p class="text-xs font-medium text-slate-900 dark:text-slate-100 truncate">{{ selectedFile.name }}</p>
                    <p class="text-[11px] text-slate-400">{{ formatSize(selectedFile.size) }}</p>
                  </div>
                </div>
                <button class="absolute top-1.5 right-1.5 text-slate-400 hover:text-red-500" @click.stop="clearFile">
                  <XMarkIcon class="w-3.5 h-3.5" />
                </button>
              </template>
              <template v-else>
                <CloudArrowUpIcon class="w-6 h-6 text-slate-300 dark:text-slate-600 mx-auto mb-1.5" />
                <p class="text-xs text-slate-500">拖拽或点击上传</p>
              </template>
            </div>
          </div>

          <div>
            <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-2">引擎</label>
            <select v-model="selectedEngine" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
              <option value="">自动（智能路由）</option>
              <option value="lopdf">Lopdf — 布局感知</option>
              <option value="pdf-extract">PDF Extract — 快速</option>
              <option value="pdfium">PDFium — 高兼容性</option>
            </select>
          </div>

          <div>
            <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-2">模式</label>
            <div class="space-y-1.5">
              <label
                v-for="m in modes"
                :key="m.value"
                class="flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer transition-colors"
                :class="extractMode === m.value ? 'bg-blue-50 dark:bg-blue-500/10 border border-blue-200 dark:border-blue-500/30' : 'border border-transparent hover:bg-slate-50 dark:hover:bg-slate-700/50'"
              >
                <input v-model="extractMode" type="radio" :value="m.value" class="w-3.5 h-3.5 text-blue-600">
                <div>
                  <span class="text-xs font-medium text-slate-900 dark:text-slate-100">{{ m.label }}</span>
                  <p class="text-[11px] text-slate-400">{{ m.desc }}</p>
                </div>
              </label>
            </div>
          </div>

          <button
            :disabled="!selectedFile || loading"
            class="w-full py-2 rounded-md text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            @click="extract"
          >
            <template v-if="loading">
              <svg class="animate-spin h-3.5 w-3.5 inline mr-1.5" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" /><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" /></svg>
              处理中
            </template>
            <template v-else>提取</template>
          </button>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-w-0">
        <template v-if="result">
          <div class="flex-shrink-0 px-6 py-3 border-b border-slate-100 dark:border-slate-700/50 flex items-center gap-6 text-xs text-slate-500">
            <span>{{ result.pageCount }} 页</span>
            <span>{{ result.textLength.toLocaleString() }} 字符</span>
            <span>{{ result.duration }}ms</span>
            <span class="ml-auto font-mono">{{ result.engine }}</span>
          </div>
          <div class="flex-1 overflow-auto px-6 py-4">
            <pre class="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap font-mono leading-relaxed">{{ result.text }}</pre>
          </div>
          <div v-if="result.pages" class="flex-shrink-0 border-t border-slate-200 dark:border-slate-700 max-h-48 overflow-auto">
            <table class="min-w-full text-xs">
              <thead class="bg-slate-50 dark:bg-slate-700/30 sticky top-0">
                <tr>
                  <th class="px-4 py-2 text-left font-semibold text-slate-500 uppercase tracking-wider">页码</th>
                  <th class="px-4 py-2 text-left font-semibold text-slate-500 uppercase tracking-wider">字符数</th>
                  <th class="px-4 py-2 text-left font-semibold text-slate-500 uppercase tracking-wider">Bounding Box</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-100 dark:divide-slate-700/50">
                <tr v-for="page in result.pages" :key="page.page_number" class="hover:bg-slate-50 dark:hover:bg-slate-700/20">
                  <td class="px-4 py-1.5 font-medium text-slate-900 dark:text-slate-100">{{ page.page_number }}</td>
                  <td class="px-4 py-1.5 font-mono text-slate-500">{{ page.text.length.toLocaleString() }}</td>
                  <td class="px-4 py-1.5 font-mono text-slate-400">[{{ page.bbox?.join(', ') }}]</td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
        <div v-else class="flex-1 flex items-center justify-center text-slate-400 dark:text-slate-600">
          <span class="text-sm">上传 PDF 文件后开始提取</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import { DocumentTextIcon, CloudArrowUpIcon, XMarkIcon } from '@heroicons/vue/24/outline'
import { usePdfStore } from '@/stores/pdfStore'
import type { PageMetadata } from '@/types/api'

const { t } = useI18n()
const store = usePdfStore()
const { loading } = storeToRefs(store)

interface ExtractResult {
  text: string
  pageCount: number
  textLength: number
  engine: string
  duration: number
  pages?: PageMetadata[]
}

const modes = [
  { value: 'text', label: '纯文本', desc: '快速提取纯文本' },
  { value: 'structured', label: '结构化数据', desc: '含 Bounding Box' },
  { value: 'markdown', label: 'Markdown', desc: 'Markdown 格式输出' }
]

const selectedFile = ref<File | null>(null)
const selectedEngine = ref('')
const extractMode = ref<'text' | 'structured' | 'markdown'>('text')
const result = ref<ExtractResult | null>(null)
const copied = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

const handleFileSelect = (e: Event): void => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (file?.name.endsWith('.pdf')) selectedFile.value = file
}

const handleDrop = (e: DragEvent): void => {
  const file = e.dataTransfer?.files?.[0]
  if (file?.name.endsWith('.pdf')) selectedFile.value = file
}

const clearFile = (): void => {
  selectedFile.value = null
  result.value = null
}

const formatSize = (bytes: number): string => {
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return `${(bytes / Math.pow(1024, i) * 100).toFixed(2)} ${['B', 'KB', 'MB'][i]}`
}

const extract = async (): Promise<void> => {
  if (!selectedFile.value) return
  const startTime = Date.now()
  const adapter = selectedEngine.value || null
  const data = extractMode.value === 'structured'
    ? await store.extractStructuredFromFile(selectedFile.value, adapter)
    : await store.extractTextFromFile(selectedFile.value, adapter)
  if (!data) return
  const text = (data as { text?: string; extracted_text?: string }).text || (data as { extracted_text?: string }).extracted_text || ''
  result.value = {
    text,
    pageCount: (data as { page_count?: number }).page_count || 1,
    textLength: text.length,
    engine: selectedEngine.value || 'auto',
    duration: Date.now() - startTime,
    pages: (data as { pages?: PageMetadata[] }).pages
  }
}

const copyText = async (): Promise<void> => {
  if (!result.value) return
  await navigator.clipboard.writeText(result.value.text)
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}
</script>
