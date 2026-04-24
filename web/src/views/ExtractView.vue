<template>
  <div class="p-2xl">
    <header class="mb-2xl">
      <h1 class="text-h1 font-bold">{{ t('extract.title') }}</h1>
    </header>

    <div class="grid grid-cols-5 gap-xl">
      <!-- Left: Input -->
      <div class="col-span-2 space-y-lg">
        <!-- File Input -->
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('extract.pdfFile') }}</label>
          <div
            class="border-2 border-dashed border-border rounded-lg p-xl text-center cursor-pointer hover:border-primary transition-colors"
            @click="$refs.fileInput.click()"
          >
            <CloudArrowUpIcon class="w-10 h-10 mx-auto mb-md text-text-muted" />
            <div class="text-sm text-text-secondary">{{ t('extract.dropPdf') }}</div>
            <input
              ref="fileInput"
              type="file"
              accept=".pdf"
              class="hidden"
              @change="handleFileSelect"
            />
          </div>
        </div>

        <!-- Selected File -->
        <div v-if="selectedFile" class="bg-surface rounded-lg p-md border border-border">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-sm">
              <DocumentTextIcon class="w-4 h-4 text-primary" />
              <span class="text-sm font-mono">{{ selectedFile.name }}</span>
            </div>
            <button @click="clearFile" class="text-text-muted hover:text-error">
              <XMarkIcon class="w-4 h-4" />
            </button>
          </div>
          <div class="text-micro text-text-muted mt-xs">{{ formatSize(selectedFile.size) }}</div>
        </div>

        <!-- Options -->
        <div class="space-y-md">
          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('extract.engine') }}</label>
            <select v-model="selectedEngine" class="input">
              <option value="">{{ t('extract.autoRouting') }}</option>
              <option value="lopdf">Lopdf - Layout Aware</option>
              <option value="pdf-extract">PDF Extract - Fast</option>
              <option value="pdfium">PDFium - High Compatibility</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('extract.mode') }}</label>
            <div class="flex gap-md">
              <label class="flex items-center gap-sm cursor-pointer">
                <input v-model="extractMode" type="radio" value="text" class="w-4 h-4 text-primary" />
                <span class="text-sm">{{ t('extract.textOnly') }}</span>
              </label>
              <label class="flex items-center gap-sm cursor-pointer">
                <input v-model="extractMode" type="radio" value="structured" class="w-4 h-4 text-primary" />
                <span class="text-sm">{{ t('extract.structured') }}</span>
              </label>
            </div>
          </div>
        </div>

        <!-- Extract Button -->
        <button
          @click="extract"
          :disabled="!selectedFile || isExtracting"
          class="btn-primary w-full"
        >
          <span v-if="isExtracting" class="flex items-center justify-center gap-sm">
            <svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"/>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
            </svg>
            {{ t('extract.processing') }}
          </span>
          <span v-else>{{ t('common.extract') }}</span>
        </button>
      </div>

      <!-- Right: Result -->
      <div class="col-span-3">
        <div v-if="result" class="space-y-lg">
          <!-- Stats Bar -->
          <div class="grid grid-cols-4 gap-md bg-surface rounded-lg p-md border border-border">
            <div class="text-center">
              <div class="text-2xl font-bold text-primary">{{ result.pageCount }}</div>
              <div class="text-micro text-text-muted">{{ t('extract.pages') }}</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-primary">{{ formatNumber(result.textLength) }}</div>
              <div class="text-micro text-text-muted">{{ t('extract.chars') }}</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-primary">{{ result.duration }}</div>
              <div class="text-micro text-text-muted">{{ t('extract.ms') }}</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-primary-light">{{ result.engine }}</div>
              <div class="text-micro text-text-muted">{{ t('extract.engine') }}</div>
            </div>
          </div>

          <!-- Text Output -->
          <div>
            <div class="flex items-center justify-between mb-sm">
              <span class="text-sm font-medium text-text-secondary">{{ t('extract.extractedText') }}</span>
              <button @click="copyText" class="btn-ghost btn-sm">
                {{ copied ? t('common.copied') : t('common.copy') }}
              </button>
            </div>
            <div class="bg-surface rounded-lg border border-border p-md h-96 overflow-auto">
              <pre class="font-mono text-sm text-text-primary whitespace-pre-wrap">{{ result.text }}</pre>
            </div>
          </div>

          <!-- Pages (if structured) -->
          <div v-if="result.pages" class="bg-surface rounded-lg border border-border overflow-hidden">
            <table class="table">
              <thead>
                <tr>
                  <th>Page</th>
                  <th>Characters</th>
                  <th>Bounding Box</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="page in result.pages" :key="page.pageNumber">
                  <td class="font-medium">{{ page.pageNumber }}</td>
                  <td class="font-mono">{{ page.text.length }}</td>
                  <td class="font-mono text-micro text-text-muted">[{{ page.bbox.join(', ') }}]</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- Empty State -->
        <div v-else class="h-full flex items-center justify-center text-center">
          <div>
            <DocumentTextIcon class="w-16 h-16 mx-auto mb-md text-text-muted opacity-30" />
            <div class="text-text-muted">{{ t('extract.selectPdf') }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  DocumentTextIcon,
  CloudArrowUpIcon,
  XMarkIcon
} from '@heroicons/vue/24/outline'
import axios from 'axios'

const { t } = useI18n()

const selectedFile = ref(null)
const selectedEngine = ref('')
const extractMode = ref('text')
const isExtracting = ref(false)
const result = ref(null)
const copied = ref(false)

const handleFileSelect = (e) => {
  const file = e.target.files[0]
  if (file && file.name.endsWith('.pdf')) {
    selectedFile.value = file
  }
}

const clearFile = () => {
  selectedFile.value = null
  result.value = null
}

const formatSize = (bytes) => {
  const k = 1024
  const sizes = ['B', 'KB', 'MB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}

const formatNumber = (num) => {
  return num.toLocaleString()
}

const extract = async () => {
  if (!selectedFile.value) return

  isExtracting.value = true
  const startTime = Date.now()

  try {
    const formData = new FormData()
    formData.append('file', selectedFile.value)
    if (selectedEngine.value) {
      formData.append('adapter', selectedEngine.value)
    }

    const endpoint = extractMode.value === 'structured'
      ? '/api/v1/x2text/extract-json'
      : '/api/v1/x2text/extract'

    const response = await axios.post(endpoint, formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })

    const data = response.data
    result.value = {
      text: data.text || data.extracted_text || '',
      pageCount: data.page_count || 1,
      textLength: (data.text || data.extracted_text || '').length,
      engine: selectedEngine.value || 'auto',
      duration: Date.now() - startTime,
      pages: data.pages
    }
  } catch (err) {
    result.value = {
      text: `Error: ${err.response?.data?.message || err.message}`,
      pageCount: 0,
      textLength: 0,
      engine: 'error',
      duration: Date.now() - startTime
    }
  } finally {
    isExtracting.value = false
  }
}

const copyText = async () => {
  await navigator.clipboard.writeText(result.value.text)
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}
</script>
