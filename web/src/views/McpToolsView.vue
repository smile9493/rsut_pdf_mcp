<template>
  <div class="p-xl">
    <div class="max-w-6xl mx-auto">
      <!-- Header -->
      <div class="mb-xl">
        <h1 class="text-2xl font-bold text-text-primary mb-sm">{{ t('mcp.title') }}</h1>
        <p class="text-text-secondary">{{ t('mcp.description') }}</p>
      </div>

      <!-- Tool Selection -->
      <div class="bg-surface border border-border rounded-lg p-lg mb-lg">
        <h2 class="text-lg font-semibold text-text-primary mb-md">{{ t('mcp.selectTool') }}</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-md">
          <button
            v-for="tool in availableTools"
            :key="tool.name"
            @click="selectedTool = tool"
            :class="[
              'p-md rounded-lg border text-left transition-all',
              selectedTool?.name === tool.name
                ? 'border-primary bg-primary/10'
                : 'border-border hover:border-primary/50 hover:bg-surface-hover'
            ]"
          >
            <div class="flex items-center gap-sm mb-xs">
              <component :is="tool.icon" class="w-5 h-5 text-primary" />
              <span class="font-medium text-text-primary">{{ tool.name }}</span>
            </div>
            <p class="text-sm text-text-secondary">{{ tool.description }}</p>
          </button>
        </div>
      </div>

      <!-- Tool Configuration -->
      <div v-if="selectedTool" class="bg-surface border border-border rounded-lg p-lg mb-lg">
        <h2 class="text-lg font-semibold text-text-primary mb-md">{{ t('mcp.configureTool') }}</h2>
        
        <!-- File Input -->
        <div class="mb-md">
          <label class="block text-sm font-medium text-text-primary mb-xs">
            {{ t('mcp.filePath') }}
          </label>
          <div class="flex gap-sm">
            <input
              v-model="filePath"
              type="text"
              :placeholder="t('mcp.filePathPlaceholder')"
              class="flex-1 px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
            />
            <button
              @click="triggerFileUpload"
              class="px-md py-sm bg-primary text-white rounded hover:bg-primary-dark transition-colors"
            >
              {{ t('mcp.browse') }}
            </button>
            <input
              ref="fileInput"
              type="file"
              accept=".pdf"
              @change="handleFileSelect"
              class="hidden"
            />
          </div>
        </div>

        <!-- Tool-specific parameters -->
        <div v-if="selectedTool.name === 'search_keywords'" class="mb-md">
          <label class="block text-sm font-medium text-text-primary mb-xs">
            {{ t('mcp.keywords') }}
          </label>
          <input
            v-model="keywords"
            type="text"
            :placeholder="t('mcp.keywordsPlaceholder')"
            class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
          />
          <div class="mt-xs flex items-center gap-md">
            <label class="flex items-center gap-xs">
              <input v-model="caseSensitive" type="checkbox" class="rounded" />
              <span class="text-sm text-text-secondary">{{ t('mcp.caseSensitive') }}</span>
            </label>
          </div>
        </div>

        <div v-if="selectedTool.name === 'extract_keywords'" class="mb-md">
          <label class="block text-sm font-medium text-text-primary mb-xs">
            {{ t('mcp.topN') }}
          </label>
          <input
            v-model.number="topN"
            type="number"
            min="1"
            max="100"
            class="w-32 px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
          />
        </div>

        <!-- Adapter Selection -->
        <div class="mb-md">
          <label class="block text-sm font-medium text-text-primary mb-xs">
            {{ t('mcp.adapter') }}
          </label>
          <select
            v-model="selectedAdapter"
            class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
          >
            <option :value="null">{{ t('mcp.autoSelect') }}</option>
            <option v-for="adapter in adapters" :key="adapter" :value="adapter">
              {{ adapter }}
            </option>
          </select>
        </div>

        <!-- Execute Button -->
        <button
          @click="executeTool"
          :disabled="loading || !filePath"
          :class="[
            'w-full py-md rounded font-medium transition-colors',
            loading || !filePath
              ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
              : 'bg-primary text-white hover:bg-primary-dark'
          ]"
        >
          <span v-if="loading" class="flex items-center justify-center gap-sm">
            <svg class="animate-spin h-5 w-5" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            {{ t('mcp.executing') }}
          </span>
          <span v-else>{{ t('mcp.execute') }}</span>
        </button>
      </div>

      <!-- Results -->
      <div v-if="result" class="bg-surface border border-border rounded-lg p-lg">
        <div class="flex items-center justify-between mb-md">
          <h2 class="text-lg font-semibold text-text-primary">{{ t('mcp.results') }}</h2>
          <div class="flex items-center gap-sm text-sm text-text-muted">
            <ClockIcon class="w-4 h-4" />
            <span>{{ result.duration }}ms</span>
          </div>
        </div>

        <!-- Result Display based on tool type -->
        <div v-if="selectedTool.name === 'extract_text'" class="bg-surface-hover rounded p-md">
          <pre class="text-sm text-text-primary whitespace-pre-wrap">{{ result.data }}</pre>
        </div>

        <div v-else-if="selectedTool.name === 'search_keywords'" class="space-y-md">
          <div class="flex items-center gap-md text-sm">
            <span class="text-text-secondary">
              {{ t('mcp.totalMatches') }}: <strong class="text-text-primary">{{ result.data.total_matches }}</strong>
            </span>
            <span class="text-text-secondary">
              {{ t('mcp.pagesWithMatches') }}: <strong class="text-text-primary">{{ result.data.pages_with_matches?.length || 0 }}</strong>
            </span>
          </div>
          <div v-for="(match, idx) in result.data.matches" :key="idx" class="bg-surface-hover rounded p-md">
            <div class="flex items-center gap-sm mb-xs">
              <span class="px-sm py-xs bg-primary/20 text-primary rounded text-xs font-medium">
                {{ match.keyword }}
              </span>
              <span class="text-xs text-text-muted">
                {{ t('mcp.page') }} {{ match.page_number }}
              </span>
            </div>
            <p class="text-sm text-text-primary">{{ match.text }}</p>
          </div>
        </div>

        <div v-else-if="selectedTool.name === 'extract_keywords'" class="space-y-sm">
          <div
            v-for="[keyword, count] in result.data.keywords"
            :key="keyword"
            class="flex items-center justify-between bg-surface-hover rounded px-md py-sm"
          >
            <span class="text-text-primary">{{ keyword }}</span>
            <span class="px-sm py-xs bg-primary/20 text-primary rounded text-sm font-medium">
              {{ count }}
            </span>
          </div>
        </div>

        <div v-else class="bg-surface-hover rounded p-md">
          <pre class="text-sm text-text-primary whitespace-pre-wrap">{{ JSON.stringify(result.data, null, 2) }}</pre>
        </div>
      </div>

      <!-- Error Display -->
      <div v-if="error" class="bg-error/10 border border-error rounded-lg p-lg">
        <div class="flex items-center gap-sm">
          <ExclamationCircleIcon class="w-5 h-5 text-error" />
          <span class="text-error font-medium">{{ t('mcp.error') }}</span>
        </div>
        <p class="mt-sm text-sm text-error">{{ error }}</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  DocumentTextIcon,
  MagnifyingGlassIcon,
  TagIcon,
  ClockIcon,
  ExclamationCircleIcon
} from '@heroicons/vue/24/outline'
import { pdfApi } from '@/composables/useApi'

const { t } = useI18n()

const availableTools = [
  {
    name: 'extract_text',
    description: t('mcp.tools.extractText'),
    icon: DocumentTextIcon
  },
  {
    name: 'search_keywords',
    description: t('mcp.tools.searchKeywords'),
    icon: MagnifyingGlassIcon
  },
  {
    name: 'extract_keywords',
    description: t('mcp.tools.extractKeywords'),
    icon: TagIcon
  }
]

const selectedTool = ref(null)
const filePath = ref('')
const keywords = ref('')
const caseSensitive = ref(false)
const topN = ref(10)
const selectedAdapter = ref(null)
const adapters = ref([])
const loading = ref(false)
const result = ref(null)
const error = ref(null)
const fileInput = ref(null)

const triggerFileUpload = () => {
  fileInput.value?.click()
}

const handleFileSelect = (e) => {
  const file = e.target.files[0]
  if (file) {
    filePath.value = file.name
  }
}

const loadAdapters = async () => {
  try {
    adapters.value = await pdfApi.listAdapters()
  } catch {
    adapters.value = ['lopdf', 'pdf-extract', 'pdfium']
  }
}

const executeTool = async () => {
  if (!filePath.value || !selectedTool.value) return

  loading.value = true
  error.value = null
  result.value = null

  try {
    let response
    const startTime = Date.now()

    switch (selectedTool.value.name) {
      case 'extract_text':
        response = await pdfApi.extractTextFromPath(filePath.value, selectedAdapter.value)
        break

      case 'search_keywords':
        if (!keywords.value) {
          throw new Error(t('mcp.noKeywords'))
        }
        response = await pdfApi.searchKeywords(
          filePath.value,
          keywords.value.split(',').map(k => k.trim()),
          { caseSensitive: caseSensitive.value }
        )
        break

      case 'extract_keywords':
        response = await pdfApi.extractKeywords(filePath.value, topN.value)
        break

      default:
        throw new Error(t('mcp.unknownTool'))
    }

    result.value = {
      data: response,
      duration: Date.now() - startTime
    }
  } catch (err) {
    error.value = err.message || t('mcp.executionFailed')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadAdapters()
})
</script>
