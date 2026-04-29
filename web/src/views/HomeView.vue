<template>
  <div class="h-full overflow-auto bg-slate-50 dark:bg-slate-900">
    <!-- Minimal header -->
    <div class="sticky top-0 z-10 bg-slate-50/90 dark:bg-slate-900/90 backdrop-blur-sm px-8 py-5">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <h1 class="text-lg font-semibold text-slate-900 dark:text-slate-100">
            {{ t('dashboard.title') }}
          </h1>
          <span class="text-sm text-slate-400 dark:text-slate-500">{{ t('dashboard.subtitle') }}</span>
          <div class="h-5 w-px bg-slate-200 dark:bg-slate-700" />
          <div class="flex items-center gap-2">
            <div
              class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium"
              :class="health.mcpHealthy ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400' : 'bg-red-50 text-red-700 dark:bg-red-500/10 dark:text-red-400'"
            >
              <span class="w-1.5 h-1.5 rounded-full" :class="health.mcpHealthy ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'" />
              {{ health.mcpHealthy ? 'MCP 健康' : 'MCP 异常' }}
            </div>
            <span class="text-xs text-slate-400 dark:text-slate-500">
              客户端 <strong class="text-slate-600 dark:text-slate-300">{{ health.clientConnections }}</strong>
            </span>
            <span class="text-xs text-slate-400 dark:text-slate-500">
              运行 <strong class="text-slate-600 dark:text-slate-300">{{ uptimeStr }}</strong>
            </span>
          </div>
        </div>
        <button
          class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-slate-600 dark:text-slate-300 bg-white dark:bg-slate-700 border border-slate-200 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-600 transition-colors"
          @click="refreshAll"
          :disabled="loading"
        >
          <ArrowPathIcon class="w-3.5 h-3.5" :class="{ 'animate-spin': loading }" />
          {{ t('common.refresh') }}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="px-8 pb-12 max-w-7xl mx-auto space-y-8">
      <!-- Error -->
      <div
        v-if="error"
        class="flex items-start gap-3 p-4 rounded-lg bg-red-50 dark:bg-red-500/10 border border-red-200 dark:border-red-500/20"
      >
        <ExclamationTriangleIcon class="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
        <div class="flex-1">
          <p class="text-sm font-medium text-red-800 dark:text-red-200">连接错误</p>
          <p class="text-xs text-red-600 dark:text-red-400 mt-1">{{ error }}</p>
        </div>
        <button class="text-xs font-medium text-red-700 dark:text-red-300 hover:underline" @click="refreshAll">重试</button>
      </div>

      <!-- Loading -->
      <div v-if="loading && metrics.totalCalls === 0" class="flex items-center justify-center py-24">
        <div class="relative w-12 h-12">
          <div class="absolute inset-0 rounded-full border-2 border-slate-200 dark:border-slate-600" />
          <div class="absolute inset-0 rounded-full border-2 border-blue-600 border-t-transparent animate-spin" />
        </div>
        <p class="ml-4 text-sm text-slate-500">{{ t('common.loading') }}</p>
      </div>

      <template v-else>
        <!-- Metric cards -->
        <div class="grid grid-cols-5 gap-4">
          <div class="p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider">{{ t('dashboard.tools') }}</p>
            <p class="text-2xl font-semibold text-slate-900 dark:text-slate-100 mt-2">{{ activeToolsCount }}</p>
            <p class="text-xs text-slate-400 dark:text-slate-500 mt-1">
              {{ AVAILABLE_TOOLS.map(item => item.label).join(' · ') }}
            </p>
          </div>
          <div class="p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider">{{ t('dashboard.totalCalls') }}</p>
            <p class="text-2xl font-semibold text-slate-900 dark:text-slate-100 mt-2">{{ metrics.totalCalls.toLocaleString() }}</p>
            <p class="text-xs text-slate-400 dark:text-slate-500 mt-1">累计调用次数</p>
          </div>
          <div class="p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider">{{ t('dashboard.avgLatency') }}</p>
            <p class="text-2xl font-semibold text-slate-900 dark:text-slate-100 mt-2">{{ metrics.avgLatency }}<span class="text-sm font-normal text-slate-400">ms</span></p>
            <div class="mt-2 flex items-center gap-2">
              <div class="flex-1 h-1 bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden">
                <div
                  class="h-full rounded-full transition-all duration-500"
                  :class="metrics.successRate >= 95 ? 'bg-emerald-500' : metrics.successRate >= 80 ? 'bg-amber-500' : 'bg-red-500'"
                  :style="{ width: metrics.successRate + '%' }"
                />
              </div>
              <span class="text-xs font-medium text-slate-500">{{ metrics.successRate.toFixed(1) }}%</span>
            </div>
          </div>
          <div class="p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider">{{ t('dashboard.tokenUsage') }}</p>
            <p class="text-2xl font-semibold text-slate-900 dark:text-slate-100 mt-2">~{{ (metrics.totalCalls * 1500).toLocaleString() }}</p>
            <p class="text-xs text-slate-400 dark:text-slate-500 mt-1">{{ t('dashboard.tokenEstimate') }}</p>
          </div>
          <div class="p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider">{{ t('dashboard.filesProcessed') }}</p>
            <p class="text-2xl font-semibold text-slate-900 dark:text-slate-100 mt-2">{{ metrics.filesProcessed }}</p>
            <p class="text-xs text-slate-400 dark:text-slate-500 mt-1">累计处理文件数</p>
          </div>
        </div>

        <!-- VLM status -->
        <div
          v-if="status.vlmEnabled"
          class="p-5 rounded-lg bg-zinc-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700"
        >
          <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-4">{{ t('dashboard.vlmStatus') }}</p>
          <div class="grid grid-cols-4 gap-6">
            <div>
              <p class="text-xs text-slate-400 dark:text-slate-500 mb-1">{{ t('dashboard.vlmModel') }}</p>
              <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">{{ status.vlmModel }}</p>
              <p class="text-xs text-slate-400 dark:text-slate-500 mt-0.5">{{ vlmModelName }}</p>
            </div>
            <div>
              <p class="text-xs text-slate-400 dark:text-slate-500 mb-1">{{ t('dashboard.multiModelRouting') }}</p>
              <p class="text-sm font-semibold" :class="status.vlmMultiModelRouting ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-400'">
                {{ status.vlmMultiModelRouting ? '智能路由' : '固定模型' }}
              </p>
            </div>
            <div>
              <p class="text-xs text-slate-400 dark:text-slate-500 mb-1">{{ t('dashboard.thinking') }}</p>
              <p class="text-sm font-semibold" :class="status.vlmThinking ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-400'">
                {{ status.vlmThinking ? '已开启' : '未开启' }}
              </p>
            </div>
            <div>
              <p class="text-xs text-slate-400 dark:text-slate-500 mb-1">{{ t('dashboard.functionCall') }}</p>
              <p class="text-sm font-semibold" :class="status.vlmFunctionCall ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-400'">
                {{ status.vlmFunctionCall ? '已开启' : '未开启' }}
              </p>
            </div>
          </div>
        </div>

        <!-- Quality Probe & Pipeline -->
        <div class="grid grid-cols-2 gap-6">
          <div class="p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-4">{{ t('dashboard.qualityProbe') }}</p>
            <div class="space-y-3">
              <div class="flex items-center justify-between p-3 rounded-lg bg-emerald-50 dark:bg-emerald-500/10">
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                  <span class="text-xs font-medium text-emerald-700 dark:text-emerald-300">mmap 零拷贝</span>
                </div>
                <span class="text-xs font-semibold text-emerald-700 dark:text-emerald-300">就绪</span>
              </div>
              <div class="flex items-center justify-between p-3 rounded-lg" :class="status.pdfiumReady ? 'bg-emerald-50 dark:bg-emerald-500/10' : 'bg-red-50 dark:bg-red-500/10'">
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full" :class="status.pdfiumReady ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'" />
                  <span class="text-xs font-medium" :class="status.pdfiumReady ? 'text-emerald-700 dark:text-emerald-300' : 'text-red-700 dark:text-red-300'">质量探测</span>
                </div>
                <span class="text-xs font-semibold" :class="status.pdfiumReady ? 'text-emerald-700 dark:text-emerald-300' : 'text-red-700 dark:text-red-300'">
                  {{ status.pdfiumReady ? '自动' : '离线' }}
                </span>
              </div>
              <div class="flex items-center justify-between p-3 rounded-lg" :class="status.vlmEnabled ? 'bg-emerald-50 dark:bg-emerald-500/10' : 'bg-slate-50 dark:bg-slate-700/50'">
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full" :class="status.vlmEnabled ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400'" />
                  <span class="text-xs font-medium" :class="status.vlmEnabled ? 'text-emerald-700 dark:text-emerald-300' : 'text-slate-500 dark:text-slate-400'">VLM 热切换</span>
                </div>
                <span class="text-xs font-semibold" :class="status.vlmEnabled ? 'text-emerald-700 dark:text-emerald-300' : 'text-slate-500 dark:text-slate-400'">
                  {{ status.vlmEnabled ? '已启用' : '未配置' }}
                </span>
              </div>
            </div>
          </div>

          <div class="p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-4">{{ t('dashboard.wikiStatus') }}</p>
            <div class="space-y-3">
              <div class="flex items-center justify-between p-3 rounded-lg bg-slate-50 dark:bg-slate-700/50">
                <span class="text-xs text-slate-500 dark:text-slate-400">存储结构</span>
                <span class="text-xs font-semibold text-slate-700 dark:text-slate-300">raw / wiki / scheme</span>
              </div>
              <div class="flex items-center justify-between p-3 rounded-lg bg-slate-50 dark:bg-slate-700/50">
                <span class="text-xs text-slate-500 dark:text-slate-400">自动索引</span>
                <span class="text-xs font-semibold text-emerald-600 dark:text-emerald-400">MAP.md</span>
              </div>
              <div class="flex items-center justify-between p-3 rounded-lg bg-slate-50 dark:bg-slate-700/50">
                <span class="text-xs text-slate-500 dark:text-slate-400">双模态工具</span>
                <span class="text-xs font-semibold text-slate-700 dark:text-slate-300">Server + Agent</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Tools + System -->
        <div class="grid grid-cols-3 gap-6">
          <!-- Tool stats -->
          <div class="col-span-2 p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-4">{{ t('dashboard.toolUsage') }}</p>
            <div class="grid grid-cols-2 gap-3">
              <div
                v-for="tool in metrics.tools"
                :key="tool.name"
                class="p-4 rounded-lg bg-slate-50 dark:bg-slate-700/50 border border-slate-100 dark:border-slate-600/50"
              >
                <div class="flex items-center justify-between mb-2">
                  <span class="text-sm font-medium text-slate-900 dark:text-slate-100">{{ tool.name }}</span>
                  <span class="text-lg font-semibold text-slate-900 dark:text-slate-100">{{ tool.calls.toLocaleString() }}</span>
                </div>
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs text-slate-400 dark:text-slate-500">
                    <span>平均 {{ tool.latency }}ms</span>
                    <span :class="tool.successRate >= 95 ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600'">{{ Math.round(tool.successRate) }}%</span>
                  </div>
                  <div class="h-1 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden">
                    <div
                      class="h-full rounded-full transition-all duration-500"
                      :class="tool.successRate >= 95 ? 'bg-emerald-500' : 'bg-amber-500'"
                      :style="{ width: tool.successRate + '%' }"
                    />
                  </div>
                </div>
              </div>
              <div
                v-if="metrics.tools.length === 0"
                class="col-span-2 flex flex-col items-center justify-center py-10 text-slate-400 dark:text-slate-600"
              >
                <CubeIcon class="w-10 h-10 mb-2" />
                <p class="text-sm">{{ t('dashboard.noActivity') }}</p>
              </div>
            </div>
          </div>

          <!-- System status -->
          <div class="p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-4">{{ t('dashboard.systemStatus') }}</p>
            <div class="space-y-4">
              <div>
                <div class="flex items-center justify-between mb-1.5">
                  <span class="text-xs text-slate-400 dark:text-slate-500">{{ t('dashboard.memoryUsage') }}</span>
                  <span class="text-sm font-semibold" :class="status.memoryPercent > 80 ? 'text-red-600' : 'text-emerald-600 dark:text-emerald-400'">
                    {{ status.memoryPercent.toFixed(1) }}%
                  </span>
                </div>
                <div class="h-1 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden">
                  <div
                    class="h-full rounded-full transition-all duration-500"
                    :class="status.memoryPercent > 80 ? 'bg-red-500' : 'bg-emerald-500'"
                    :style="{ width: Math.min(status.memoryPercent, 100) + '%' }"
                  />
                </div>
              </div>

              <div class="flex items-center justify-between p-3 rounded-lg" :class="status.pdfiumReady ? 'bg-emerald-50 dark:bg-emerald-500/10' : 'bg-red-50 dark:bg-red-500/10'">
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full" :class="status.pdfiumReady ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'" />
                  <span class="text-xs font-medium" :class="status.pdfiumReady ? 'text-emerald-700 dark:text-emerald-300' : 'text-red-700 dark:text-red-300'">
                    {{ t('dashboard.pdfiumStatus') }}
                  </span>
                </div>
                <span class="text-xs font-semibold" :class="status.pdfiumReady ? 'text-emerald-700 dark:text-emerald-300' : 'text-red-700 dark:text-red-300'">
                  {{ status.pdfiumReady ? '就绪' : '离线' }}
                </span>
              </div>

              <div class="p-3 rounded-lg bg-slate-50 dark:bg-slate-700/50">
                <p class="text-xs text-slate-400 dark:text-slate-500 mb-1">{{ t('dashboard.queueLength') }}</p>
                <p class="text-xl font-semibold text-slate-900 dark:text-slate-100">{{ status.queueLength }}</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Activity log -->
        <div class="p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
          <div class="flex items-center justify-between mb-4">
            <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider">{{ t('dashboard.activityLog') }}</p>
            <button class="text-xs text-blue-600 dark:text-blue-400 hover:underline" @click="clearLogs">{{ t('common.clear') }}</button>
          </div>
          <div class="max-h-64 overflow-y-auto divide-y divide-slate-100 dark:divide-slate-700">
            <div
              v-for="(log, index) in logs"
              :key="index"
              class="flex items-start gap-3 py-2"
            >
              <span
                class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium flex-shrink-0 mt-0.5"
                :class="{
                  'bg-emerald-50 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400': log.level === 'info',
                  'bg-amber-50 text-amber-700 dark:bg-amber-500/10 dark:text-amber-400': log.level === 'warn',
                  'bg-red-50 text-red-700 dark:bg-red-500/10 dark:text-red-400': log.level === 'error',
                }"
              >
                {{ log.level.toUpperCase() }}
              </span>
              <span class="text-xs text-slate-400 dark:text-slate-500 font-mono flex-shrink-0">{{ log.time }}</span>
              <span class="flex-1 text-sm text-slate-600 dark:text-slate-300">{{ log.message }}</span>
            </div>
            <div
              v-if="logs.length === 0"
              class="flex flex-col items-center justify-center py-12 text-slate-400 dark:text-slate-600"
            >
              <DocumentIcon class="w-10 h-10 mb-2" />
              <p class="text-sm">{{ t('dashboard.noActivity') }}</p>
            </div>
          </div>
        </div>

        <!-- Quick actions -->
        <div>
          <p class="text-xs text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-3">{{ t('dashboard.quickActions') }}</p>
          <div class="grid grid-cols-5 gap-3">
            <router-link
              v-for="action in quickActions"
              :key="action.to"
              :to="action.to"
              class="flex flex-col items-center gap-2 p-5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 hover:border-blue-300 dark:hover:border-blue-600 transition-colors"
            >
              <component :is="action.icon" class="w-5 h-5 text-slate-400 dark:text-slate-500" />
              <span class="text-sm text-slate-600 dark:text-slate-300">{{ action.label }}</span>
            </router-link>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDashboard } from '@/composables/useDashboard'
import {
  DocumentTextIcon,
  MagnifyingGlassIcon,
  DocumentDuplicateIcon,
  CommandLineIcon,
  CubeIcon,
  DocumentIcon,
  ArrowPathIcon,
  ExclamationTriangleIcon,
  ServerIcon
} from '@heroicons/vue/24/outline'

const AVAILABLE_TOOLS = [
  { name: 'extract_text', label: '文本提取' },
  { name: 'extract_structured', label: '结构化提取' },
  { name: 'get_page_count', label: '页数查询' },
  { name: 'search_keywords', label: '关键词搜索' },
  { name: 'extrude_to_server_wiki', label: '服务端Wiki构建' },
  { name: 'extrude_to_agent_payload', label: '本地Wiki投影' }
] as const

const { t } = useI18n()
const {
  loading,
  error,
  metrics,
  status,
  health,
  logs,
  uptimeStr,
  clearLogs,
  refreshAll
} = useDashboard()

const glmModelNames: Record<string, string> = {
  'glm-4.6v': '高性能版',
  'glm-4.6v-flashx': '轻量高速版',
  'glm-4.6v-flash': '免费版',
  'glm-ocr': '专业OCR',
  'gpt-4o': 'GPT-4o',
  'claude-3.5-sonnet': 'Claude 3.5 Sonnet'
}

const vlmModelName = computed(() => glmModelNames[status.value.vlmModel] || status.value.vlmModel)
const activeToolsCount = computed(() => AVAILABLE_TOOLS.length)

const quickActions = computed(() => [
  { to: '/extract', label: t('nav.extract'), icon: DocumentTextIcon },
  { to: '/search', label: t('nav.search'), icon: MagnifyingGlassIcon },
  { to: '/batch', label: t('nav.batch'), icon: DocumentDuplicateIcon },
  { to: '/mcp-tools', label: t('dashboard.testTools'), icon: CommandLineIcon },
  { to: '/wiki', label: t('nav.wiki'), icon: ServerIcon }
])
</script>
