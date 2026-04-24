<template>
  <div class="p-xl">
    <div class="max-w-6xl mx-auto">
      <!-- Header -->
      <div class="mb-xl">
        <h1 class="text-2xl font-bold text-text-primary mb-sm">{{ t('batch.title') }}</h1>
        <p class="text-text-secondary">{{ t('batch.description') }}</p>
      </div>

      <!-- File Selection -->
      <div class="bg-surface border border-border rounded-lg p-lg mb-lg">
        <h2 class="text-lg font-semibold text-text-primary mb-md">{{ t('batch.selectFiles') }}</h2>
        
        <!-- Upload Area -->
        <div
          @dragover.prevent="isDragging = true"
          @dragleave.prevent="isDragging = false"
          @drop.prevent="handleDrop"
          :class="[
            'border-2 border-dashed rounded-lg p-xl text-center transition-colors',
            isDragging ? 'border-primary bg-primary/5' : 'border-border'
          ]"
        >
          <CloudArrowUpIcon class="w-12 h-12 text-text-muted mx-auto mb-md" />
          <p class="text-text-primary mb-sm">{{ t('batch.dragDrop') }}</p>
          <p class="text-sm text-text-muted mb-md">{{ t('batch.or') }}</p>
          <button
            @click="triggerFileUpload"
            class="px-lg py-sm bg-primary text-white rounded hover:bg-primary-dark transition-colors"
          >
            {{ t('batch.browseFiles') }}
          </button>
          <input
            ref="fileInput"
            type="file"
            accept=".pdf"
            multiple
            @change="handleFileSelect"
            class="hidden"
          />
        </div>

        <!-- Selected Files List -->
        <div v-if="files.length > 0" class="mt-lg">
          <div class="flex items-center justify-between mb-md">
            <h3 class="text-md font-medium text-text-primary">
              {{ t('batch.selectedFiles') }} ({{ files.length }})
            </h3>
            <button
              @click="files = []"
              class="text-sm text-error hover:text-error-dark"
            >
              {{ t('batch.clearAll') }}
            </button>
          </div>
          <div class="space-y-sm max-h-64 overflow-auto">
            <div
              v-for="(file, idx) in files"
              :key="idx"
              class="flex items-center justify-between bg-surface-hover rounded px-md py-sm"
            >
              <div class="flex items-center gap-sm">
                <DocumentTextIcon class="w-5 h-5 text-primary" />
                <span class="text-sm text-text-primary">{{ file.name }}</span>
                <span class="text-xs text-text-muted">({{ formatSize(file.size) }})</span>
              </div>
              <button
                @click="files.splice(idx, 1)"
                class="text-text-muted hover:text-error"
              >
                <XMarkIcon class="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Processing Options -->
      <div class="bg-surface border border-border rounded-lg p-lg mb-lg">
        <h2 class="text-lg font-semibold text-text-primary mb-md">{{ t('batch.options') }}</h2>
        
        <div class="grid grid-cols-1 md:grid-cols-2 gap-md">
          <!-- Operation -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-xs">
              {{ t('batch.operation') }}
            </label>
            <select
              v-model="options.operation"
              class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option value="extract_text">{{ t('batch.ops.extractText') }}</option>
              <option value="extract_structured">{{ t('batch.ops.extractStructured') }}</option>
              <option value="get_info">{{ t('batch.ops.getInfo') }}</option>
            </select>
          </div>

          <!-- Adapter -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-xs">
              {{ t('batch.adapter') }}
            </label>
            <select
              v-model="options.adapter"
              class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option :value="null">{{ t('batch.autoSelect') }}</option>
              <option v-for="adapter in adapters" :key="adapter" :value="adapter">
                {{ adapter }}
              </option>
            </select>
          </div>

          <!-- Parallel Jobs -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-xs">
              {{ t('batch.parallelJobs') }}
            </label>
            <input
              v-model.number="options.parallelJobs"
              type="number"
              min="1"
              max="10"
              class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          <!-- Output Format -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-xs">
              {{ t('batch.outputFormat') }}
            </label>
            <select
              v-model="options.outputFormat"
              class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option value="json">JSON</option>
              <option value="txt">TXT</option>
              <option value="csv">CSV</option>
            </select>
          </div>
        </div>

        <!-- Start Button -->
        <button
          @click="startBatch"
          :disabled="files.length === 0 || processing"
          :class="[
            'w-full mt-lg py-md rounded font-medium transition-colors',
            files.length === 0 || processing
              ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
              : 'bg-primary text-white hover:bg-primary-dark'
          ]"
        >
          <span v-if="processing" class="flex items-center justify-center gap-sm">
            <svg class="animate-spin h-5 w-5" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            {{ t('batch.processing') }}
          </span>
          <span v-else>{{ t('batch.start') }}</span>
        </button>
      </div>

      <!-- Progress -->
      <div v-if="processing || results.length > 0" class="bg-surface border border-border rounded-lg p-lg mb-lg">
        <div class="flex items-center justify-between mb-md">
          <h2 class="text-lg font-semibold text-text-primary">{{ t('batch.progress') }}</h2>
          <span class="text-sm text-text-muted">
            {{ completedCount }} / {{ files.length }}
          </span>
        </div>

        <!-- Progress Bar -->
        <div class="mb-lg">
          <div class="h-2 bg-surface-hover rounded-full overflow-hidden">
            <div
              class="h-full bg-primary transition-all duration-300"
              :style="{ width: `${progress}%` }"
            ></div>
          </div>
          <div class="mt-xs text-sm text-text-muted text-right">{{ progress.toFixed(1) }}%</div>
        </div>

        <!-- Results List -->
        <div class="space-y-sm max-h-96 overflow-auto">
          <div
            v-for="result in results"
            :key="result.file"
            :class="[
              'flex items-center justify-between rounded px-md py-sm',
              result.status === 'success' ? 'bg-success/10' : 'bg-error/10'
            ]"
          >
            <div class="flex items-center gap-sm">
              <CheckCircleIcon v-if="result.status === 'success'" class="w-5 h-5 text-success" />
              <XCircleIcon v-else class="w-5 h-5 text-error" />
              <span class="text-sm text-text-primary">{{ result.file }}</span>
            </div>
            <div class="flex items-center gap-md">
              <span class="text-xs text-text-muted">{{ result.duration }}ms</span>
              <button
                @click="viewResult(result)"
                class="text-primary hover:text-primary-dark text-sm"
              >
                {{ t('batch.view') }}
              </button>
            </div>
          </div>
        </div>

        <!-- Summary -->
        <div v-if="!processing && results.length > 0" class="mt-lg pt-lg border-t border-border">
          <div class="grid grid-cols-3 gap-md text-center">
            <div>
              <div class="text-2xl font-bold text-text-primary">{{ successCount }}</div>
              <div class="text-sm text-text-muted">{{ t('batch.successful') }}</div>
            </div>
            <div>
              <div class="text-2xl font-bold text-error">{{ failedCount }}</div>
              <div class="text-sm text-text-muted">{{ t('batch.failed') }}</div>
            </div>
            <div>
              <div class="text-2xl font-bold text-text-primary">{{ avgDuration }}ms</div>
              <div class="text-sm text-text-muted">{{ t('batch.avgDuration') }}</div>
            </div>
          </div>

          <!-- Export Button -->
          <button
            @click="exportResults"
            class="mt-lg w-full py-md border border-primary text-primary rounded hover:bg-primary/10 transition-colors"
          >
            {{ t('batch.exportResults') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Result Modal -->
    <div v-if="selectedResult" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="selectedResult = null">
      <div class="bg-surface rounded-lg max-w-4xl w-full max-h-[80vh] overflow-auto m-lg" @click.stop>
        <div class="p-lg border-b border-border flex items-center justify-between">
          <h3 class="text-lg font-semibold text-text-primary">{{ selectedResult.file }}</h3>
          <button @click="selectedResult = null" class="text-text-muted hover:text-text-primary">
            <XMarkIcon class="w-5 h-5" />
          </button>
        </div>
        <div class="p-lg">
          <pre class="text-sm text-text-primary whitespace-pre-wrap overflow-auto">{{ selectedResult.data }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  CloudArrowUpIcon,
  DocumentTextIcon,
  XMarkIcon,
  CheckCircleIcon,
  XCircleIcon
} from '@heroicons/vue/24/outline'
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
  const total = results.value.reduce((sum, r) => sum + r.duration, 0)
  return Math.round(total / results.value.length)
})

const formatSize = (bytes) => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

const triggerFileUpload = () => {
  fileInput.value?.click()
}

const handleFileSelect = (e) => {
  const selectedFiles = Array.from(e.target.files)
  files.value.push(...selectedFiles)
}

const handleDrop = (e) => {
  isDragging.value = false
  const droppedFiles = Array.from(e.dataTransfer.files).filter(f => f.name.endsWith('.pdf'))
  files.value.push(...droppedFiles)
}

const loadAdapters = async () => {
  try {
    adapters.value = await pdfApi.listAdapters()
  } catch {
    adapters.value = ['lopdf', 'pdf-extract', 'pdfium']
  }
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
        case 'extract_text':
          result = await pdfApi.extractTextFromFile(file, options.value.adapter)
          break
        case 'extract_structured':
          result = await pdfApi.extractStructuredFromFile(file, options.value.adapter)
          break
        case 'get_info':
          result = await pdfApi.getInfo(file)
          break
      }

      results.value.push({
        file: file.name,
        status: 'success',
        duration: Date.now() - startTime,
        data: result
      })
    } catch (error) {
      results.value.push({
        file: file.name,
        status: 'failed',
        duration: Date.now() - startTime,
        error: error.message
      })
    }
  }

  processing.value = false
}

const viewResult = (result) => {
  selectedResult.value = result
}

const exportResults = () => {
  const data = results.value.map(r => ({
    file: r.file,
    status: r.status,
    duration: r.duration,
    data: r.data
  }))

  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `batch-results-${Date.now()}.json`
  a.click()
  URL.revokeObjectURL(url)
}

onMounted(() => {
  loadAdapters()
})
</script>
