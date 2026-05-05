<template>
  <div class="h-full overflow-auto bg-bg">
    <div class="sticky top-0 z-10 bg-bg/90 backdrop-blur-sm px-2xl py-lg border-b border-border/50">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-lg">
          <h1 class="text-lg font-semibold text-text-primary">
            {{ t('dashboard.title') }}
          </h1>
          <span class="text-sm text-text-muted">{{ t('dashboard.subtitle') }}</span>
          <div class="h-5 w-px bg-border" />
          <div class="flex items-center gap-sm">
            <div
              class="inline-flex items-center gap-1.5 px-sm py-xs rounded-full text-xs font-medium"
              :class="health.mcpHealthy ? 'bg-success/10 text-success' : 'bg-error/10 text-error'"
            >
              <span class="w-1.5 h-1.5 rounded-full" :class="health.mcpHealthy ? 'bg-success animate-pulse' : 'bg-error'" />
              {{ health.mcpHealthy ? 'MCP 健康' : 'MCP 异常' }}
            </div>
            <span class="text-xs text-text-muted">
              客户端 <strong class="text-text-secondary">{{ health.clientConnections }}</strong>
            </span>
            <span class="text-xs text-text-muted">
              运行 <strong class="text-text-secondary">{{ uptimeStr }}</strong>
            </span>
          </div>
        </div>
        <button
          class="inline-flex items-center gap-1.5 px-md py-sm text-sm font-medium text-text-secondary bg-surface border border-border rounded-lg hover:bg-surface-hover transition-colors"
          @click="refreshAll"
          :disabled="loading"
        >
          <ArrowPathIcon class="w-3.5 h-3.5" :class="{ 'animate-spin': loading }" />
          {{ t('common.refresh') }}
        </button>
      </div>
    </div>

    <div class="px-2xl pb-12 max-w-7xl mx-auto space-y-2xl">
      <div
        v-if="error"
        class="flex items-start gap-md p-lg rounded-lg bg-error/10 border border-error/20"
      >
        <ExclamationTriangleIcon class="w-5 h-5 text-error flex-shrink-0 mt-xs" />
        <div class="flex-1">
          <p class="text-sm font-medium text-error">连接错误</p>
          <p class="text-xs text-error mt-xs">{{ error }}</p>
        </div>
        <button class="text-xs font-medium text-error hover:underline" @click="refreshAll">重试</button>
      </div>

      <div v-if="loading && metrics.totalCalls === 0" class="flex items-center justify-center py-24">
        <div class="relative w-12 h-12">
          <div class="absolute inset-0 rounded-full border-2 border-border" />
          <div class="absolute inset-0 rounded-full border-2 border-primary border-t-transparent animate-spin" />
        </div>
        <p class="ml-lg text-sm text-text-muted">{{ t('common.loading') }}</p>
      </div>

      <template v-else>
        <div class="grid grid-cols-5 gap-lg">
          <div class="p-lg rounded-lg bg-surface border border-border">
            <p class="text-xs text-text-muted uppercase tracking-wider">{{ t('dashboard.tools') }}</p>
            <p class="text-2xl font-semibold text-text-primary mt-md">{{ activeToolsCount }}</p>
            <p class="text-xs text-text-muted mt-xs">
              {{ AVAILABLE_TOOLS.map(item => item.label).join(' · ') }}
            </p>
          </div>
          <div class="p-lg rounded-lg bg-surface border border-border">
            <p class="text-xs text-text-muted uppercase tracking-wider">{{ t('dashboard.totalCalls') }}</p>
            <p class="text-2xl font-semibold text-text-primary mt-md">{{ metrics.totalCalls.toLocaleString() }}</p>
            <p class="text-xs text-text-muted mt-xs">累计调用次数</p>
          </div>
          <div class="p-lg rounded-lg bg-surface border border-border">
            <p class="text-xs text-text-muted uppercase tracking-wider">{{ t('dashboard.avgLatency') }}</p>
            <p class="text-2xl font-semibold text-text-primary mt-md">{{ metrics.avgLatency }}<span class="text-sm font-normal text-text-muted">ms</span></p>
            <div class="mt-md flex items-center gap-sm">
              <div class="flex-1 h-1 bg-surface-100 rounded-full overflow-hidden">
                <div
                  class="h-full rounded-full transition-all duration-500"
                  :class="metrics.successRate >= 95 ? 'bg-success' : metrics.successRate >= 80 ? 'bg-warning' : 'bg-error'"
                  :style="{ width: metrics.successRate + '%' }"
                />
              </div>
              <span class="text-xs font-medium text-text-secondary">{{ metrics.successRate.toFixed(1) }}%</span>
            </div>
          </div>
          <div class="p-lg rounded-lg bg-surface border border-border">
            <p class="text-xs text-text-muted uppercase tracking-wider">{{ t('dashboard.tokenUsage') }}</p>
            <p class="text-2xl font-semibold text-text-primary mt-md">~{{ (metrics.totalCalls * 1500).toLocaleString() }}</p>
            <p class="text-xs text-text-muted mt-xs">{{ t('dashboard.tokenEstimate') }}</p>
          </div>
          <div class="p-lg rounded-lg bg-surface border border-border">
            <p class="text-xs text-text-muted uppercase tracking-wider">{{ t('dashboard.filesProcessed') }}</p>
            <p class="text-2xl font-semibold text-text-primary mt-md">{{ metrics.filesProcessed }}</p>
            <p class="text-xs text-text-muted mt-xs">累计处理文件数</p>
          </div>
        </div>

        <div
          v-if="status.vlmEnabled"
          class="p-lg rounded-lg bg-surface-100 border border-border"
        >
          <p class="text-xs text-text-muted uppercase tracking-wider mb-lg">{{ t('dashboard.vlmStatus') }}</p>
          <div class="grid grid-cols-4 gap-xl">
            <div>
              <p class="text-xs text-text-muted mb-xs">{{ t('dashboard.vlmModel') }}</p>
              <p class="text-sm font-semibold text-text-primary">{{ status.vlmModel }}</p>
              <p class="text-xs text-text-muted mt-xs">{{ vlmModelName }}</p>
            </div>
            <div>
              <p class="text-xs text-text-muted mb-xs">{{ t('dashboard.multiModelRouting') }}</p>
              <p class="text-sm font-semibold" :class="status.vlmMultiModelRouting ? 'text-success' : 'text-text-muted'">
                {{ status.vlmMultiModelRouting ? '智能路由' : '固定模型' }}
              </p>
            </div>
            <div>
              <p class="text-xs text-text-muted mb-xs">{{ t('dashboard.thinking') }}</p>
              <p class="text-sm font-semibold" :class="status.vlmThinking ? 'text-success' : 'text-text-muted'">
                {{ status.vlmThinking ? '已开启' : '未开启' }}
              </p>
            </div>
            <div>
              <p class="text-xs text-text-muted mb-xs">{{ t('dashboard.functionCall') }}</p>
              <p class="text-sm font-semibold" :class="status.vlmFunctionCall ? 'text-success' : 'text-text-muted'">
                {{ status.vlmFunctionCall ? '已开启' : '未开启' }}
              </p>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-xl">
          <div class="p-lg rounded-lg bg-surface border border-border">
            <p class="text-xs text-text-muted uppercase tracking-wider mb-lg">{{ t('dashboard.qualityProbe') }}</p>
            <div class="space-y-md">
              <div class="flex items-center justify-between p-md rounded-lg bg-success/10">
                <div class="flex items-center gap-sm">
                  <span class="w-2 h-2 rounded-full bg-success animate-pulse" />
                  <span class="text-xs font-medium text-success">mmap 零拷贝</span>
                </div>
                <span class="text-xs font-semibold text-success">就绪</span>
              </div>
              <div class="flex items-center justify-between p-md rounded-lg" :class="status.pdfiumReady ? 'bg-success/10' : 'bg-error/10'">
                <div class="flex items-center gap-sm">
                  <span class="w-2 h-2 rounded-full" :class="status.pdfiumReady ? 'bg-success animate-pulse' : 'bg-error'" />
                  <span class="text-xs font-medium" :class="status.pdfiumReady ? 'text-success' : 'text-error'">质量探测</span>
                </div>
                <span class="text-xs font-semibold" :class="status.pdfiumReady ? 'text-success' : 'text-error'">
                  {{ status.pdfiumReady ? '自动' : '离线' }}
                </span>
              </div>
              <div class="flex items-center justify-between p-md rounded-lg" :class="status.vlmEnabled ? 'bg-success/10' : 'bg-surface-100'">
                <div class="flex items-center gap-sm">
                  <span class="w-2 h-2 rounded-full" :class="status.vlmEnabled ? 'bg-success animate-pulse' : 'bg-text-muted'" />
                  <span class="text-xs font-medium" :class="status.vlmEnabled ? 'text-success' : 'text-text-muted'">VLM 热切换</span>
                </div>
                <span class="text-xs font-semibold" :class="status.vlmEnabled ? 'text-success' : 'text-text-muted'">
                  {{ status.vlmEnabled ? '已启用' : '未配置' }}
                </span>
              </div>
            </div>
          </div>

          <div class="p-lg rounded-lg bg-surface border border-border">
            <p class="text-xs text-text-muted uppercase tracking-wider mb-lg">{{ t('dashboard.wikiStatus') }}</p>
            <div class="space-y-md">
              <div class="flex items-center justify-between p-md rounded-lg bg-surface-100">
                <span class="text-xs text-text-muted">存储结构</span>
                <span class="text-xs font-semibold text-text-secondary">raw / wiki / scheme</span>
              </div>
              <div class="flex items-center justify-between p-md rounded-lg bg-surface-100">
                <span class="text-xs text-text-muted">自动索引</span>
                <span class="text-xs font-semibold text-success">MAP.md</span>
              </div>
              <div class="flex items-center justify-between p-md rounded-lg bg-surface-100">
                <span class="text-xs text-text-muted">双模态工具</span>
                <span class="text-xs font-semibold text-text-secondary">Server + Agent</span>
              </div>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-xl">
          <div class="col-span-2 p-lg rounded-lg bg-surface border border-border">
            <p class="text-xs text-text-muted uppercase tracking-wider mb-lg">{{ t('dashboard.toolUsage') }}</p>
            <div class="grid grid-cols-2 gap-md">
              <div
                v-for="tool in metrics.tools"
                :key="tool.name"
                class="p-lg rounded-lg bg-surface-100 border border-border"
              >
                <div class="flex items-center justify-between mb-sm">
                  <span class="text-sm font-medium text-text-primary">{{ getToolDisplayName(tool.name) }}</span>
                  <span class="text-lg font-semibold text-text-primary">{{ tool.calls.toLocaleString() }}</span>
                </div>
                <div class="space-y-xs">
                  <div class="flex items-center justify-between text-xs text-text-muted">
                    <span>平均 {{ tool.latency }}ms</span>
                    <span :class="tool.successRate >= 95 ? 'text-success' : 'text-warning'">{{ Math.round(tool.successRate) }}%</span>
                  </div>
                  <div class="h-1 bg-border rounded-full overflow-hidden">
                    <div
                      class="h-full rounded-full transition-all duration-500"
                      :class="tool.successRate >= 95 ? 'bg-success' : 'bg-warning'"
                      :style="{ width: tool.successRate + '%' }"
                    />
                  </div>
                </div>
              </div>
              <div
                v-if="metrics.tools.length === 0"
                class="col-span-2 flex flex-col items-center justify-center py-10 text-text-muted"
              >
                <CubeIcon class="w-10 h-10 mb-sm" />
                <p class="text-sm">{{ t('dashboard.noActivity') }}</p>
              </div>
            </div>
          </div>

          <div class="p-lg rounded-lg bg-surface border border-border">
            <p class="text-xs text-text-muted uppercase tracking-wider mb-lg">{{ t('dashboard.systemStatus') }}</p>
            <div class="space-y-lg">
              <div>
                <div class="flex items-center justify-between mb-xs">
                  <span class="text-xs text-text-muted">{{ t('dashboard.memoryUsage') }}</span>
                  <span class="text-sm font-semibold" :class="status.memoryPercent > 80 ? 'text-error' : 'text-success'">
                    {{ status.memoryPercent.toFixed(1) }}%
                  </span>
                </div>
                <div class="h-1 bg-border rounded-full overflow-hidden">
                  <div
                    class="h-full rounded-full transition-all duration-500"
                    :class="status.memoryPercent > 80 ? 'bg-error' : 'bg-success'"
                    :style="{ width: Math.min(status.memoryPercent, 100) + '%' }"
                  />
                </div>
              </div>

              <div class="flex items-center justify-between p-md rounded-lg" :class="status.pdfiumReady ? 'bg-success/10' : 'bg-error/10'">
                <div class="flex items-center gap-sm">
                  <span class="w-2 h-2 rounded-full" :class="status.pdfiumReady ? 'bg-success animate-pulse' : 'bg-error'" />
                  <span class="text-xs font-medium" :class="status.pdfiumReady ? 'text-success' : 'text-error'">
                    {{ t('dashboard.pdfiumStatus') }}
                  </span>
                </div>
                <span class="text-xs font-semibold" :class="status.pdfiumReady ? 'text-success' : 'text-error'">
                  {{ status.pdfiumReady ? '就绪' : '离线' }}
                </span>
              </div>

              <div class="p-md rounded-lg bg-surface-100">
                <p class="text-xs text-text-muted mb-xs">{{ t('dashboard.queueLength') }}</p>
                <p class="text-xl font-semibold text-text-primary">{{ status.queueLength }}</p>
              </div>
            </div>
          </div>
        </div>

        <div class="p-lg rounded-lg bg-surface border border-border">
          <div class="flex items-center justify-between mb-lg">
            <p class="text-xs text-text-muted uppercase tracking-wider">{{ t('dashboard.activityLog') }}</p>
            <button class="text-xs text-primary hover:underline" @click="clearLogs">{{ t('common.clear') }}</button>
          </div>
          <div class="max-h-64 overflow-y-auto divide-y divide-border">
            <div
              v-for="(log, index) in logs"
              :key="index"
              class="flex items-start gap-md py-sm"
            >
              <span
                class="inline-flex items-center px-xs py-xs rounded text-xs font-medium flex-shrink-0 mt-xs"
                :class="{
                  'bg-success/10 text-success': log.level === 'info',
                  'bg-warning/10 text-warning': log.level === 'warn',
                  'bg-error/10 text-error': log.level === 'error',
                }"
              >
                {{ log.level.toUpperCase() }}
              </span>
              <span class="text-xs text-text-muted font-mono flex-shrink-0">{{ log.time }}</span>
              <span class="flex-1 text-sm text-text-secondary">{{ log.message }}</span>
            </div>
            <div
              v-if="logs.length === 0"
              class="flex flex-col items-center justify-center py-12 text-text-muted"
            >
              <DocumentIcon class="w-10 h-10 mb-sm" />
              <p class="text-sm">{{ t('dashboard.noActivity') }}</p>
            </div>
          </div>
        </div>

        <div>
          <p class="text-xs text-text-muted uppercase tracking-wider mb-md">{{ t('dashboard.quickActions') }}</p>
          <div class="grid grid-cols-5 gap-md">
            <router-link
              v-for="action in quickActions"
              :key="action.to"
              :to="action.to"
              class="flex flex-col items-center gap-sm p-lg rounded-lg bg-surface border border-border hover:border-primary transition-colors"
            >
              <component :is="action.icon" class="w-5 h-5 text-text-muted" />
              <span class="text-sm text-text-secondary">{{ action.label }}</span>
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
  { name: 'extrude_to_agent_payload', label: '本地Wiki投影' },
  { name: 'compile_to_wiki', label: '编译到Wiki' },
  { name: 'incremental_compile', label: '增量编译' },
  { name: 'search_knowledge', label: '搜索知识库' },
  { name: 'rebuild_index', label: '重建索引' },
  { name: 'get_entry_context', label: '获取条目上下文' },
  { name: 'find_orphans', label: '查找孤立条目' },
  { name: 'suggest_links', label: '建议链接' },
  { name: 'export_concept_map', label: '导出概念图' },
  { name: 'check_quality', label: '质量检查' },
  { name: 'micro_compile', label: '微编译' },
  { name: 'aggregate_entries', label: '聚合条目' },
  { name: 'hypothesis_test', label: '假设测试' },
  { name: 'recompile_entry', label: '重编译条目' }
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

const toolNameMap: Record<string, string> = {
  'extract_text': '文本提取',
  'extract_structured': '结构化提取',
  'get_page_count': '页数查询',
  'search_keywords': '关键词搜索',
  'extrude_to_server_wiki': '服务端Wiki构建',
  'extrude_to_agent_payload': '本地Wiki投影',
  'compile_to_wiki': '编译到Wiki',
  'incremental_compile': '增量编译',
  'search_knowledge': '搜索知识库',
  'rebuild_index': '重建索引',
  'get_entry_context': '获取条目上下文',
  'find_orphans': '查找孤立条目',
  'suggest_links': '建议链接',
  'export_concept_map': '导出概念图',
  'check_quality': '质量检查',
  'micro_compile': '微编译',
  'aggregate_entries': '聚合条目',
  'hypothesis_test': '假设测试',
  'recompile_entry': '重编译条目'
}

const getToolDisplayName = (name: string) => {
  const cnName = toolNameMap[name] || name
  return `${cnName}（${name}）`
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
