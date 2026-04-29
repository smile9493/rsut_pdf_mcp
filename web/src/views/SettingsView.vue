<template>
  <div class="p-2xl">
    <header class="mb-2xl">
      <h1 class="text-h1 font-bold">
        {{ t('settings.title') }}
      </h1>
      <p class="text-text-secondary mt-sm">
        {{ t('settings.subtitle') }}
      </p>
    </header>

    <div class="space-y-xl">
      <!-- MCP Server Configuration -->
      <section class="bg-surface rounded-lg p-xl border border-border">
        <h2 class="text-lg font-semibold mb-lg">
          {{ t('settings.mcp.title') }}
        </h2>
        
        <div class="grid grid-cols-2 gap-xl">
          <div class="space-y-lg">
            <div>
              <label class="block text-sm font-medium text-text-secondary mb-sm">
                {{ t('settings.mcp.serverPath') }}
              </label>
              <input
                v-model="mcpConfig.serverCommand"
                type="text"
                class="input font-mono"
                placeholder="pdf-mcp"
              >
              <p class="text-micro text-text-muted mt-xs">
                {{ t('settings.mcp.serverPathHint') }}
              </p>
            </div>

            <div>
              <label class="block text-sm font-medium text-text-secondary mb-sm">
                {{ t('settings.mcp.timeout') }}
              </label>
              <input
                v-model.number="mcpConfig.timeout"
                type="number"
                class="input"
                min="1000"
                max="300000"
                step="1000"
              >
              <span class="text-sm text-text-muted ml-sm">ms</span>
            </div>
          </div>

          <div class="space-y-lg">
            <h3 class="text-sm font-medium text-text-secondary">
              {{ t('settings.mcp.tools') }}
            </h3>
            <div class="space-y-sm">
              <div
                v-for="tool in mcpTools"
                :key="tool.name"
                class="flex items-center gap-sm"
              >
                <div class="w-2 h-2 rounded-full bg-success" />
                <span class="text-sm font-mono">{{ tool.name }}</span>
                <span class="text-xs text-text-muted">- {{ tool.desc }}</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- VLM Configuration -->
      <section class="bg-surface rounded-lg p-xl border border-border">
        <h2 class="text-lg font-semibold mb-lg flex items-center gap-sm">
          {{ t('settings.vlm.title') }}
          <span class="badge badge-warning text-micro">可选</span>
        </h2>
        
        <div class="grid grid-cols-2 gap-xl">
          <div class="space-y-lg">
            <div>
              <label class="block text-sm font-medium text-text-secondary mb-sm">
                {{ t('settings.vlm.provider') }}
              </label>
              <select
                v-model="vlmConfig.provider"
                class="input"
              >
                <option value="">
                  {{ t('settings.vlm.disabled') }}
                </option>
                <option value="openai">
                  OpenAI (GPT-4o)
                </option>
                <option value="anthropic">
                  Anthropic (Claude 3.5)
                </option>
                <option value="glm">
                  智谱AI (GLM-4V)
                </option>
                <option value="custom">
                  {{ t('settings.vlm.custom') }}
                </option>
              </select>
            </div>

            <div v-if="vlmConfig.provider">
              <label class="block text-sm font-medium text-text-secondary mb-sm">
                {{ t('settings.vlm.apiKey') }}
              </label>
              <div class="relative">
                <input
                  v-model="vlmConfig.apiKey"
                  :type="showApiKey ? 'text' : 'password'"
                  class="input font-mono pr-10"
                  :placeholder="t('settings.vlm.apiKeyPlaceholder')"
                >
                <button
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary"
                  @click="showApiKey = !showApiKey"
                >
                  {{ showApiKey ? '🙈' : '👁️' }}
                </button>
              </div>
            </div>

            <div v-if="vlmConfig.provider === 'custom'">
              <label class="block text-sm font-medium text-text-secondary mb-sm">
                {{ t('settings.vlm.endpoint') }}
              </label>
              <input
                v-model="vlmConfig.endpoint"
                type="text"
                class="input font-mono"
                placeholder="https://api.openai.com/v1/chat/completions"
              >
            </div>
          </div>

          <div
            v-if="vlmConfig.provider"
            class="space-y-lg"
          >
            <div class="grid grid-cols-2 gap-md">
              <div>
                <label class="block text-sm font-medium text-text-secondary mb-sm">
                  {{ t('settings.vlm.timeout') }}
                </label>
                <input
                  v-model.number="vlmConfig.timeout"
                  type="number"
                  class="input"
                  min="5"
                  max="120"
                >
                <span class="text-sm text-text-muted ml-xs">s</span>
              </div>
              <div>
                <label class="block text-sm font-medium text-text-secondary mb-sm">
                  {{ t('settings.vlm.maxConcurrency') }}
                </label>
                <input
                  v-model.number="vlmConfig.maxConcurrency"
                  type="number"
                  class="input"
                  min="1"
                  max="20"
                >
              </div>
            </div>

            <div class="p-md bg-warning/10 rounded border border-warning/30">
              <p class="text-sm text-warning">
                ⚠️ {{ t('settings.vlm.keyWarning') }}
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Generated Config Preview -->
      <section class="bg-surface rounded-lg p-xl border border-border">
        <div class="flex items-center justify-between mb-lg">
          <h2 class="text-lg font-semibold">
            {{ t('settings.preview') }}
          </h2>
          <div class="flex gap-sm">
            <button
              class="btn-secondary btn-sm"
              @click="copyConfig"
            >
              {{ t('common.copy') }}
            </button>
            <button
              class="btn-primary btn-sm"
              @click="downloadConfig"
            >
              {{ t('settings.downloadConfig') }}
            </button>
          </div>
        </div>
        
        <pre class="bg-bg rounded-lg p-lg font-mono text-sm overflow-x-auto max-h-96"><code>{{ configJson }}</code></pre>
      </section>

      <!-- Actions -->
      <section class="flex gap-lg">
        <button
          class="btn-primary"
          @click="saveConfig"
        >
          {{ t('common.save') }}
        </button>
        <button
          class="btn-ghost"
          @click="resetConfig"
        >
          {{ t('common.reset') }}
        </button>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const showApiKey = ref(false)

const mcpTools = [
  { name: 'extract_text', desc: '提取纯文本' },
  { name: 'extract_structured', desc: '提取结构化数据' },
  { name: 'get_page_count', desc: '获取页数' },
  { name: 'search_keywords', desc: '关键词搜索' }
]

const mcpConfig = ref({
  serverCommand: 'pdf-mcp',
  timeout: 30000
})

const vlmConfig = ref({
  provider: '',
  apiKey: '',
  endpoint: '',
  timeout: 30,
  maxConcurrency: 5
})

watch(() => vlmConfig.value.provider, (p) => {
  if (p === 'openai') {
    vlmConfig.value.endpoint = 'https://api.openai.com/v1/chat/completions'
  } else if (p === 'anthropic') {
    vlmConfig.value.endpoint = 'https://api.anthropic.com/v1/messages'
  } else if (p === 'glm') {
    vlmConfig.value.endpoint = 'https://open.bigmodel.cn/api/paas/v4/chat/completions'
  }
})

const configJson = computed(() => {
  const env: Record<string, string> = {}
  
  if (vlmConfig.value.provider && vlmConfig.value.apiKey) {
    env.VLM_API_KEY = vlmConfig.value.apiKey
  }
  if (vlmConfig.value.endpoint) {
    env.VLM_ENDPOINT = vlmConfig.value.endpoint
  }
  if (vlmConfig.value.provider === 'anthropic') {
    env.VLM_MODEL = 'claude-3.5-sonnet'
  } else if (vlmConfig.value.provider === 'glm') {
    env.VLM_MODEL = 'glm-4v'
  } else if (vlmConfig.value.provider === 'openai') {
    env.VLM_MODEL = 'gpt-4o'
  }
  if (vlmConfig.value.provider) {
    env.VLM_TIMEOUT_SECS = String(vlmConfig.value.timeout)
    env.VLM_MAX_CONCURRENCY = String(vlmConfig.value.maxConcurrency)
  }

  return JSON.stringify({
    mcpServers: {
      'pdf-module': {
        command: mcpConfig.value.serverCommand,
        env: Object.keys(env).length > 0 ? env : undefined
      }
    }
  }, null, 2)
})

function saveConfig() {
  localStorage.setItem('mcp-config', JSON.stringify(mcpConfig.value))
  localStorage.setItem('vlm-config', JSON.stringify(vlmConfig.value))
}

function resetConfig() {
  mcpConfig.value = { serverCommand: 'pdf-mcp', timeout: 30000 }
  vlmConfig.value = { provider: '', apiKey: '', endpoint: '', timeout: 30, maxConcurrency: 5 }
}

async function copyConfig() {
  await navigator.clipboard.writeText(configJson.value)
}

function downloadConfig() {
  const blob = new Blob([configJson.value], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'claude_desktop_config.json'
  a.click()
  URL.revokeObjectURL(url)
}

// Load saved config
const savedMcp = localStorage.getItem('mcp-config')
if (savedMcp) {
  try { Object.assign(mcpConfig.value, JSON.parse(savedMcp)) } catch {}
}
const savedVlm = localStorage.getItem('vlm-config')
if (savedVlm) {
  try { Object.assign(vlmConfig.value, JSON.parse(savedVlm)) } catch {}
}
</script>
