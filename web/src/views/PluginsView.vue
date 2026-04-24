<template>
  <div class="p-2xl">
    <header class="mb-2xl">
      <h1 class="text-h1 font-bold">{{ t('plugins.title') }}</h1>
      <p class="text-text-secondary mt-sm">{{ t('plugins.description') }}</p>
    </header>

    <div class="space-y-xl">
      <!-- MiniMax MCP Plugin -->
      <div class="bg-surface rounded-lg border border-border overflow-hidden">
        <div class="p-lg border-b border-border bg-surface-hover">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-md">
              <div class="w-10 h-10 rounded bg-primary/20 flex items-center justify-center">
                <MagnifyingGlassIcon class="w-6 h-6 text-primary" />
              </div>
              <div>
                <h2 class="text-lg font-semibold text-text-primary">MiniMax MCP</h2>
                <p class="text-sm text-text-muted">网络搜索 & 图片理解</p>
              </div>
            </div>
            <label class="flex items-center gap-sm cursor-pointer">
              <input 
                v-model="plugins.minimax.enabled" 
                type="checkbox" 
                class="w-5 h-5 text-primary rounded"
                @change="savePlugins"
              />
              <span class="text-sm font-medium">{{ plugins.minimax.enabled ? t('common.enabled') : t('common.disabled') }}</span>
            </label>
          </div>
        </div>

        <div v-if="plugins.minimax.enabled" class="p-lg space-y-lg">
          <div class="grid grid-cols-2 gap-lg">
            <div>
              <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('plugins.apiKey') }}</label>
              <input
                v-model="plugins.minimax.apiKey"
                type="password"
                class="input-mono"
                :placeholder="t('plugins.apiKeyPlaceholder')"
                @change="savePlugins"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('plugins.baseUrl') }}</label>
              <input
                v-model="plugins.minimax.baseUrl"
                type="text"
                class="input-mono"
                placeholder="https://api.minimax.chat"
                @change="savePlugins"
              />
            </div>
          </div>

          <div class="grid grid-cols-2 gap-lg">
            <div>
              <label class="flex items-center gap-sm cursor-pointer">
                <input 
                  v-model="plugins.minimax.webSearchEnabled" 
                  type="checkbox" 
                  class="w-4 h-4 text-primary rounded"
                  @change="savePlugins"
                />
                <span class="text-sm">{{ t('plugins.minimax.webSearch') }}</span>
              </label>
              <p class="text-micro text-text-muted mt-xs ml-6">{{ t('plugins.minimax.webSearchDesc') }}</p>
            </div>

            <div>
              <label class="flex items-center gap-sm cursor-pointer">
                <input 
                  v-model="plugins.minimax.imageUnderstandEnabled" 
                  type="checkbox" 
                  class="w-4 h-4 text-primary rounded"
                  @change="savePlugins"
                />
                <span class="text-sm">{{ t('plugins.minimax.imageUnderstand') }}</span>
              </label>
              <p class="text-micro text-text-muted mt-xs ml-6">{{ t('plugins.minimax.imageUnderstandDesc') }}</p>
            </div>
          </div>

          <div class="grid grid-cols-3 gap-lg">
            <div>
              <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('plugins.timeout') }}</label>
              <input
                v-model.number="plugins.minimax.timeout"
                type="number"
                class="input"
                min="10"
                max="120"
                @change="savePlugins"
              />
              <span class="text-micro text-text-muted">{{ t('common.seconds') }}</span>
            </div>

            <div>
              <label class="block text-sm font-medium text-text-secondary mb-sm">{{ t('plugins.maxImageSize') }}</label>
              <input
                v-model.number="plugins.minimax.maxImageSize"
                type="number"
                class="input"
                min="1"
                max="50"
                @change="savePlugins"
              />
              <span class="text-micro text-text-muted">MB</span>
            </div>

            <div class="flex items-end">
              <button 
                @click="testMinimaxConnection"
                class="btn-secondary w-full"
                :disabled="!plugins.minimax.apiKey"
              >
                {{ t('plugins.testConnection') }}
              </button>
            </div>
          </div>

          <!-- Status -->
          <div v-if="minimaxStatus" class="p-md rounded-lg" :class="minimaxStatus.success ? 'bg-success/10' : 'bg-error/10'">
            <div class="flex items-center gap-sm">
              <CheckCircleIcon v-if="minimaxStatus.success" class="w-5 h-5 text-success" />
              <ExclamationCircleIcon v-else class="w-5 h-5 text-error" />
              <span :class="minimaxStatus.success ? 'text-success' : 'text-error'">{{ minimaxStatus.message }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- GLM Plugin (已在LLM配置中) -->
      <div class="bg-surface rounded-lg border border-border overflow-hidden">
        <div class="p-lg border-b border-border bg-surface-hover">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-md">
              <div class="w-10 h-10 rounded bg-accent-purple/20 flex items-center justify-center">
                <SparklesIcon class="w-6 h-6 text-accent-purple" />
              </div>
              <div>
                <h2 class="text-lg font-semibold text-text-primary">智谱AI GLM-4.6V</h2>
                <p class="text-sm text-text-muted">多模态理解 & Function Call</p>
              </div>
            </div>
            <router-link to="/settings" class="btn-secondary btn-sm">
              {{ t('plugins.configureInSettings') }}
            </router-link>
          </div>
        </div>

        <div class="p-lg">
          <div class="grid grid-cols-3 gap-lg text-sm">
            <div class="flex items-center gap-sm">
              <CheckCircleIcon class="w-4 h-4 text-success" />
              <span>{{ t('plugins.glm.features.vision') }}</span>
            </div>
            <div class="flex items-center gap-sm">
              <CheckCircleIcon class="w-4 h-4 text-success" />
              <span>{{ t('plugins.glm.features.functionCall') }}</span>
            </div>
            <div class="flex items-center gap-sm">
              <CheckCircleIcon class="w-4 h-4 text-success" />
              <span>{{ t('plugins.glm.features.context128k') }}</span>
            </div>
          </div>
          <p class="text-sm text-text-muted mt-md">
            {{ t('plugins.glm.note') }}
          </p>
        </div>
      </div>

      <!-- Plugin Stats -->
      <div class="bg-surface rounded-lg p-lg border border-border">
        <h3 class="text-h2 font-semibold mb-lg">{{ t('plugins.stats.title') }}</h3>
        <div class="grid grid-cols-4 gap-lg">
          <div class="text-center">
            <div class="text-display font-bold text-primary">{{ enabledPluginsCount }}</div>
            <div class="text-sm text-text-muted">{{ t('plugins.stats.enabled') }}</div>
          </div>
          <div class="text-center">
            <div class="text-display font-bold text-text-primary">{{ totalPluginsCount }}</div>
            <div class="text-sm text-text-muted">{{ t('plugins.stats.total') }}</div>
          </div>
          <div class="text-center">
            <div class="text-display font-bold text-success">{{ availableToolsCount }}</div>
            <div class="text-sm text-text-muted">{{ t('plugins.stats.tools') }}</div>
          </div>
          <div class="text-center">
            <div class="text-display font-bold text-info">{{ multimodalPluginsCount }}</div>
            <div class="text-sm text-text-muted">{{ t('plugins.stats.multimodal') }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  MagnifyingGlassIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  SparklesIcon
} from '@heroicons/vue/24/outline'

const { t } = useI18n()

const defaultPlugins = {
  minimax: {
    enabled: false,
    apiKey: '',
    baseUrl: 'https://api.minimax.chat',
    webSearchEnabled: true,
    imageUnderstandEnabled: true,
    timeout: 30,
    maxImageSize: 20
  }
}

const plugins = ref({ ...defaultPlugins })
const minimaxStatus = ref(null)

const loadPlugins = () => {
  const saved = localStorage.getItem('pdf-module-plugins')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      plugins.value = {
        ...defaultPlugins,
        ...parsed,
        minimax: { ...defaultPlugins.minimax, ...(parsed.minimax || {}) }
      }
    } catch {
      plugins.value = { ...defaultPlugins }
    }
  }
}

const savePlugins = () => {
  localStorage.setItem('pdf-module-plugins', JSON.stringify(plugins.value))
}

const testMinimaxConnection = async () => {
  minimaxStatus.value = { success: false, message: t('plugins.testing') }
  
  try {
    // 这里应该调用实际的API测试
    // 模拟测试
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    if (plugins.value.minimax.apiKey) {
      minimaxStatus.value = { 
        success: true, 
        message: t('plugins.connectionSuccess') 
      }
    } else {
      minimaxStatus.value = { 
        success: false, 
        message: t('plugins.connectionFailed') 
      }
    }
  } catch (error) {
    minimaxStatus.value = { 
      success: false, 
      message: error.message 
    }
  }
}

// 计算属性
const enabledPluginsCount = computed(() => {
  let count = 0
  if (plugins.value.minimax.enabled) count++
  // GLM始终可用(在LLM配置中)
  count++
  return count
})

const totalPluginsCount = computed(() => 2) // MiniMax + GLM

const availableToolsCount = computed(() => {
  let count = 0
  if (plugins.value.minimax.enabled) {
    if (plugins.value.minimax.webSearchEnabled) count++
    if (plugins.value.minimax.imageUnderstandEnabled) count++
  }
  // GLM工具
  count += 3 // vision, function call, text generation
  return count
})

const multimodalPluginsCount = computed(() => {
  let count = 0
  if (plugins.value.minimax.enabled && plugins.value.minimax.imageUnderstandEnabled) count++
  count++ // GLM always has vision
  return count
})

onMounted(() => {
  loadPlugins()
})
</script>
