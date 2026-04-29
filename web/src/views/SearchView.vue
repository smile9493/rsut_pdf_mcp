<template>
  <div class="p-2xl">
    <header class="mb-2xl">
      <h1 class="text-h1 font-bold">
        {{ t('search.title') }}
      </h1>
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
          >
        </div>

        <div>
          <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('search.keywords') }}</label>
          <textarea
            v-model="keywordsInput"
            class="input min-h-[120px] resize-y"
            :placeholder="t('search.oneKeywordPerLine')"
          />
        </div>

        <div class="space-y-sm">
          <label class="flex items-center gap-sm cursor-pointer">
            <input
              v-model="caseSensitive"
              type="checkbox"
              class="w-4 h-4 text-primary rounded"
            >
            <span class="text-sm">{{ t('search.caseSensitive') }}</span>
          </label>
          <label class="flex items-center gap-sm cursor-pointer">
            <input
              v-model="showContext"
              type="checkbox"
              class="w-4 h-4 text-primary rounded"
            >
            <span class="text-sm">{{ t('search.showContext') }}</span>
          </label>
        </div>

        <button
          :disabled="!filePath || !keywordsInput"
          class="btn-primary w-full"
          @click="copyCommand"
        >
          {{ t('common.copy') }} {{ t('mcp.config.download') }}
        </button>
        
        <div
          v-if="filePath && keywordsInput"
          class="mt-lg"
        >
          <label class="block text-sm font-medium text-text-secondary mb-sm">MCP Command</label>
          <pre class="bg-bg rounded-lg p-md font-mono text-sm overflow-x-auto border border-border">{{ mcpCommand }}</pre>
        </div>
      </div>

      <!-- Right: Instructions -->
      <div class="col-span-3">
        <div class="bg-surface rounded-lg border border-border p-lg">
          <h2 class="text-lg font-semibold mb-md">
            {{ t('search.instructions') }}
          </h2>
          <ol class="list-decimal list-inside space-y-sm text-sm text-text-secondary">
            <li>{{ t('search.step1') }}</li>
            <li>{{ t('search.step2') }}</li>
            <li>{{ t('search.step3') }}</li>
          </ol>
          
          <div class="mt-lg p-md bg-primary/10 rounded border border-primary/30">
            <p class="text-sm text-primary">
              <strong>{{ t('search.tip') }}:</strong> {{ t('search.tipText') }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { MagnifyingGlassIcon } from '@heroicons/vue/24/outline'

const { t } = useI18n()

const filePath = ref('')
const keywordsInput = ref('')
const caseSensitive = ref(false)
const showContext = ref(true)

const keywords = computed(() => {
  return keywordsInput.value.split('\n').map(k => k.trim()).filter(k => k)
})

const mcpCommand = computed(() => {
  const args = {
    file_path: filePath.value,
    keywords: keywords.value,
    case_sensitive: caseSensitive.value,
    context_length: showContext.value ? 50 : 0
  }
  return `echo '${JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/call',
    params: { name: 'search_keywords', arguments: args }
  })}' | pdf-mcp`
})

const copyCommand = async () => {
  await navigator.clipboard.writeText(mcpCommand.value)
}
</script>
