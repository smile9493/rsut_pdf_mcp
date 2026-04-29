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
          :disabled="!selectedFile || loading"
          class="btn-primary w-full"
        >
          <span v-if="loading" class="flex items-center justify-center gap-sm">
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
              <div class="text-2xl font-bold text-primary">{{ result.textLength.toLocaleString() }}</div>
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
                <tr v-for="page in result.pages" :key="page.page_number">
                  <td class="font-medium">{{ page.page_number }}</td>
                  <td class="font-mono">{{ page.text.length }}</td>
                  <td class="font-mono text-micro text-text-muted">[{{ page.bbox?.join(', ') }}]</td>
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

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import {
  DocumentTextIcon,
  CloudArrowUpIcon,
  XMarkIcon
} from '@heroicons/vue/24/outline'
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

const selectedFile = ref<File | null>(null)
const selectedEngine = ref('')
const extractMode = ref<'text' | 'structured'>('text')
const result = ref<ExtractResult | null>(null)
const copied = ref(false)

const handleFileSelect = (e: Event): void => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
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

  const text = (data as { text?: string; extracted_text?: string }).text
    || (data as { extracted_text?: string }).extracted_text
    || ''

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
