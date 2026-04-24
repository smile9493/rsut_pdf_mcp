<template>
  <div class="p-2xl">
    <header class="mb-2xl">
      <h1 class="text-h1 font-bold">{{ t('search.title') }}</h1>
    </header>

    <div class="grid grid-cols-5 gap-xl">
      <!-- Left: Input -->
      <div class="col-span-2 space-y-lg">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('search.pdfFile') }}</label>
          <input
            v-model="filePath"
            type="text"
            class="input-mono"
            placeholder="/path/to/file.pdf"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('search.keywords') }}</label>
          <textarea
            v-model="keywordsInput"
            class="input min-h-[120px] resize-y"
            :placeholder="t('search.oneKeywordPerLine')"
          ></textarea>
        </div>

        <div class="space-y-sm">
          <label class="flex items-center gap-sm cursor-pointer">
            <input v-model="caseSensitive" type="checkbox" class="w-4 h-4 text-primary rounded" />
            <span class="text-sm">{{ t('search.caseSensitive') }}</span>
          </label>
          <label class="flex items-center gap-sm cursor-pointer">
            <input v-model="showContext" type="checkbox" class="w-4 h-4 text-primary rounded" />
            <span class="text-sm">{{ t('search.showContext') }}</span>
          </label>
        </div>

        <button
          @click="search"
          :disabled="!filePath || !keywordsInput || isSearching"
          class="btn-primary w-full"
        >
          <span v-if="isSearching" class="flex items-center justify-center gap-sm">
            <svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"/>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
            </svg>
            {{ t('search.searching') }}
          </span>
          <span v-else>{{ t('common.search') }}</span>
        </button>
      </div>

      <!-- Right: Results -->
      <div class="col-span-3">
        <div v-if="result" class="space-y-lg">
          <!-- Stats -->
          <div class="grid grid-cols-4 gap-md bg-surface rounded-lg p-md border border-border">
            <div class="text-center">
              <div class="text-2xl font-bold text-primary">{{ result.totalMatches }}</div>
              <div class="text-micro text-text-muted">{{ t('search.matches') }}</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-primary">{{ result.pagesWithMatches?.length || 0 }}</div>
              <div class="text-micro text-text-muted">{{ t('search.pages') }}</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-primary">{{ keywords.length }}</div>
              <div class="text-micro text-text-muted">{{ t('search.keywordsLabel') }}</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-primary">{{ result.duration }}ms</div>
              <div class="text-micro text-text-muted">{{ t('search.time') }}</div>
            </div>
          </div>

          <!-- Matches by Keyword -->
          <div v-for="keyword in keywords" :key="keyword" class="bg-surface rounded-lg border border-border overflow-hidden">
            <div class="p-md border-b border-border">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-sm">
                  <MagnifyingGlassIcon class="w-4 h-4 text-primary" />
                  <span class="font-medium">"{{ keyword }}"</span>
                </div>
                <span class="badge-info">{{ getMatches(keyword).length }} {{ t('search.found') }}</span>
              </div>
            </div>
            
            <div class="max-h-64 overflow-auto">
              <div
                v-for="(match, idx) in getMatches(keyword)"
                :key="idx"
                class="p-md border-b border-border/50 last:border-0"
              >
                <div class="flex items-center gap-md mb-sm text-micro text-text-muted">
                  <span>{{ t('search.page') }} {{ match.pageNumber }}</span>
                  <span class="font-mono">{{ match.startIndex }}-{{ match.endIndex }}</span>
                </div>
                <div v-if="match.text" class="text-sm" v-html="highlight(match.text, keyword)"></div>
              </div>
              
              <div v-if="getMatches(keyword).length === 0" class="p-md text-center text-text-muted text-sm">
                {{ t('search.noMatches') }}
              </div>
            </div>
          </div>
        </div>

        <!-- Empty -->
        <div v-else class="h-full flex items-center justify-center text-center">
          <div>
            <MagnifyingGlassIcon class="w-16 h-16 mx-auto mb-md text-text-muted opacity-30" />
            <div class="text-text-muted">{{ t('search.enterPathAndKeywords') }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { MagnifyingGlassIcon } from '@heroicons/vue/24/outline'
import axios from 'axios'

const { t } = useI18n()

const filePath = ref('')
const keywordsInput = ref('')
const caseSensitive = ref(false)
const showContext = ref(true)
const isSearching = ref(false)
const result = ref(null)

const keywords = computed(() => {
  return keywordsInput.value.split('\n').map(k => k.trim()).filter(k => k)
})

const search = async () => {
  if (!filePath.value || keywords.value.length === 0) return

  isSearching.value = true
  const startTime = Date.now()

  try {
    // Extract text first
    const extractResponse = await axios.post('/api/v1/x2text/extract-json', {
      file_path: filePath.value
    })
    
    const text = extractResponse.data.extracted_text || ''
    const pages = extractResponse.data.pages || []
    
    // Search in frontend
    const matches = []
    keywords.value.forEach(keyword => {
      const regex = new RegExp(
        keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'),
        caseSensitive.value ? 'g' : 'gi'
      )
      let match
      while ((match = regex.exec(text)) !== null) {
        const startIndex = match.index
        const endIndex = startIndex + keyword.length
        const contextLen = showContext.value ? 50 : 0
        
        let pageNumber = 1
        if (pages.length > 0) {
          let charCount = 0
          for (const page of pages) {
            charCount += page.text.length
            if (startIndex < charCount) {
              pageNumber = page.page_number
              break
            }
          }
        }
        
        matches.push({
          keyword,
          pageNumber,
          text: text.substring(Math.max(0, startIndex - contextLen), Math.min(text.length, endIndex + contextLen)),
          startIndex,
          endIndex
        })
      }
    })
    
    result.value = {
      keywords: keywords.value,
      matches,
      totalMatches: matches.length,
      pagesWithMatches: [...new Set(matches.map(m => m.pageNumber))],
      duration: Date.now() - startTime
    }
  } catch (err) {
    result.value = {
      totalMatches: 0,
      matches: [],
      duration: Date.now() - startTime
    }
  } finally {
    isSearching.value = false
  }
}

const getMatches = (keyword) => {
  return result.value?.matches?.filter(m => m.keyword === keyword) || []
}

const highlight = (text, keyword) => {
  const regex = new RegExp(
    `(${keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`,
    caseSensitive.value ? 'g' : 'gi'
  )
  return text.replace(regex, '<mark class="bg-primary/30 text-primary px-xs rounded">$1</mark>')
}
</script>
