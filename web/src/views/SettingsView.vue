<template>
  <div class="p-2xl">
    <header class="mb-2xl">
      <h1 class="text-h1 font-bold">{{ t('settings.title') }}</h1>
    </header>

    <div class="grid grid-cols-2 gap-xl">
      <!-- API -->
      <div class="bg-surface rounded-lg p-lg border border-border">
        <h2 class="text-h2 font-semibold mb-lg">{{ t('settings.apiConfiguration') }}</h2>

        <div class="space-y-lg">
          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.baseUrl') }}</label>
            <input v-model="settings.apiBaseUrl" type="text" class="input-mono" />
          </div>

          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.timeout') }}</label>
            <input v-model.number="settings.timeout" type="number" class="input" min="10" max="300" />
          </div>

          <label class="flex items-center gap-sm cursor-pointer">
            <input v-model="settings.retryOnError" type="checkbox" class="w-4 h-4 text-primary rounded" />
            <span class="text-sm">{{ t('settings.retryOnError') }}</span>
          </label>
        </div>
      </div>

      <!-- LLM Configuration -->
      <div class="bg-surface rounded-lg p-lg border border-border">
        <h2 class="text-h2 font-semibold mb-lg">{{ t('settings.llmConfiguration') }}</h2>

        <div class="space-y-lg">
          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.llmProvider') }}</label>
            <select v-model="settings.llm.provider" class="input">
              <option value="openai">OpenAI</option>
              <option value="glm">智谱AI (GLM-4.6V)</option>
              <option value="azure">Azure OpenAI</option>
              <option value="ollama">Ollama (本地)</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.llmModel') }}</label>
            <select v-model="settings.llm.model" class="input">
              <optgroup v-if="settings.llm.provider === 'openai'" label="OpenAI">
                <option value="gpt-4o">GPT-4o (多模态)</option>
                <option value="gpt-4o-mini">GPT-4o Mini</option>
                <option value="gpt-4-turbo">GPT-4 Turbo</option>
              </optgroup>
              <optgroup v-if="settings.llm.provider === 'glm'" label="智谱AI">
                <option value="glm-4.6v">GLM-4.6V (多模态)</option>
                <option value="glm-4.6v-flashx">GLM-4.6V-FlashX (轻量)</option>
                <option value="glm-4.6v-flash">GLM-4.6V-Flash (免费)</option>
                <option value="glm-4">GLM-4</option>
              </optgroup>
              <optgroup v-if="settings.llm.provider === 'azure'" label="Azure">
                <option value="gpt-4o">GPT-4o</option>
                <option value="gpt-4">GPT-4</option>
              </optgroup>
              <optgroup v-if="settings.llm.provider === 'ollama'" label="Ollama">
                <option value="llama3">Llama 3</option>
                <option value="llava">LLaVA (多模态)</option>
                <option value="mistral">Mistral</option>
              </optgroup>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.llmApiKey') }}</label>
            <input
              v-model="settings.llm.apiKey"
              type="password"
              class="input-mono"
              :placeholder="t('settings.llmApiKeyPlaceholder')"
            />
          </div>

          <div v-if="settings.llm.provider === 'glm'">
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.llmBaseUrl') }}</label>
            <input
              v-model="settings.llm.baseUrl"
              type="text"
              class="input-mono"
              placeholder="https://open.bigmodel.cn/api/paas/v4"
            />
          </div>

          <div v-if="settings.llm.provider === 'ollama'">
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.llmBaseUrl') }}</label>
            <input
              v-model="settings.llm.baseUrl"
              type="text"
              class="input-mono"
              placeholder="http://localhost:11434"
            />
          </div>

          <div class="grid grid-cols-2 gap-md">
            <div>
              <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.llmTemperature') }}</label>
              <input
                v-model.number="settings.llm.temperature"
                type="number"
                class="input"
                min="0"
                max="2"
                step="0.1"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.llmMaxTokens') }}</label>
              <input
                v-model.number="settings.llm.maxTokens"
                type="number"
                class="input"
                min="100"
                max="128000"
              />
            </div>
          </div>

          <label class="flex items-center gap-sm cursor-pointer">
            <input v-model="settings.llm.enableVision" type="checkbox" class="w-4 h-4 text-primary rounded" />
            <span class="text-sm">{{ t('settings.llmEnableVision') }}</span>
          </label>
        </div>
      </div>

      <!-- Defaults -->
      <div class="bg-surface rounded-lg p-lg border border-border">
        <h2 class="text-h2 font-semibold mb-lg">{{ t('settings.defaults') }}</h2>

        <div class="space-y-lg">
          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.engine') }}</label>
            <select v-model="settings.defaultEngine" class="input">
              <option value="">{{ t('common.auto') }}</option>
              <option value="lopdf">Lopdf</option>
              <option value="pdf-extract">PDF Extract</option>
              <option value="pdfium">PDFium</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.mode') }}</label>
            <select v-model="settings.defaultMode" class="input">
              <option value="text">{{ t('extract.textOnly') }}</option>
              <option value="structured">{{ t('extract.structured') }}</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Display -->
      <div class="bg-surface rounded-lg p-lg border border-border">
        <h2 class="text-h2 font-semibold mb-lg">{{ t('settings.display') }}</h2>

        <div class="space-y-lg">
          <div>
            <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('settings.theme') }}</label>
            <div class="flex gap-md">
              <label class="flex items-center gap-sm cursor-pointer">
                <input v-model="settings.theme" type="radio" value="dark" class="w-4 h-4 text-primary" />
                <span class="text-sm">{{ t('common.dark') }}</span>
              </label>
              <label class="flex items-center gap-sm cursor-pointer">
                <input v-model="settings.theme" type="radio" value="light" class="w-4 h-4 text-primary" />
                <span class="text-sm">{{ t('common.light') }}</span>
              </label>
            </div>
          </div>

          <label class="flex items-center gap-sm cursor-pointer">
            <input v-model="settings.showNotifications" type="checkbox" class="w-4 h-4 text-primary rounded" />
            <span class="text-sm">{{ t('settings.showNotifications') }}</span>
          </label>
        </div>
      </div>

      <!-- Storage -->
      <div class="bg-surface rounded-lg p-lg border border-border">
        <h2 class="text-h2 font-semibold mb-lg">{{ t('settings.storage') }}</h2>

        <div class="space-y-lg">
          <label class="flex items-center gap-sm cursor-pointer">
            <input v-model="settings.saveHistory" type="checkbox" class="w-4 h-4 text-primary rounded" />
            <span class="text-sm">{{ t('settings.saveHistory') }}</span>
          </label>

          <label class="flex items-center gap-sm cursor-pointer">
            <input v-model="settings.clearCacheOnExit" type="checkbox" class="w-4 h-4 text-primary rounded" />
            <span class="text-sm">{{ t('settings.clearCacheOnExit') }}</span>
          </label>

          <button @click="clearAllData" class="btn-secondary w-full text-error hover:bg-error/10">
            {{ t('settings.clearAllData') }}
          </button>
        </div>
      </div>

      <!-- Actions -->
      <div class="col-span-2 bg-surface rounded-lg p-lg border border-border">
        <div class="flex items-center justify-between">
          <div>
            <h3 class="text-sm font-medium">{{ t('settings.saveSettings') }}</h3>
            <p class="text-micro text-text-muted">{{ t('settings.settingsSavedToStorage') }}</p>
          </div>
          <div class="flex gap-sm">
            <button @click="resetSettings" class="btn-secondary">{{ t('common.reset') }}</button>
            <button @click="saveSettings" class="btn-primary">{{ t('common.save') }}</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const defaultSettings = {
  apiBaseUrl: 'http://localhost:8000',
  timeout: 60,
  retryOnError: true,
  defaultEngine: '',
  defaultMode: 'text',
  theme: 'dark',
  showNotifications: true,
  saveHistory: true,
  clearCacheOnExit: false,
  llm: {
    provider: 'glm',
    model: 'glm-4.6v',
    apiKey: '',
    baseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    temperature: 0.0,
    maxTokens: 4096,
    enableVision: true
  }
}

const settings = ref({ ...defaultSettings })

const loadSettings = () => {
  const saved = localStorage.getItem('pdf-module-settings')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      settings.value = {
        ...defaultSettings,
        ...parsed,
        llm: { ...defaultSettings.llm, ...(parsed.llm || {}) }
      }
    } catch {
      settings.value = { ...defaultSettings }
    }
  }
}

const saveSettings = () => {
  localStorage.setItem('pdf-module-settings', JSON.stringify(settings.value))
}

const resetSettings = () => {
  settings.value = { ...defaultSettings }
}

const clearAllData = () => {
  if (confirm(t('settings.clearAllConfirm'))) {
    localStorage.clear()
    sessionStorage.clear()
    loadSettings()
  }
}

onMounted(() => {
  loadSettings()
})
</script>
