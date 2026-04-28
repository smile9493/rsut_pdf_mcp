<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface MetricSnapshot {
  tokio_blocking_queue_depth: number
  outbox_compensation_hit_rate: number
  ast_chunker_fallback_rate: number
  db_seq_reconciliation_diff: number
  vector_gc_pending_chunks: number
  vector_search_latency_p99: number
  pdf_extraction_latency_p99: number
  outbox_terminal_failed_total: number
}

const metrics = ref<MetricSnapshot>({
  tokio_blocking_queue_depth: 0,
  outbox_compensation_hit_rate: 0,
  ast_chunker_fallback_rate: 0,
  db_seq_reconciliation_diff: 0,
  vector_gc_pending_chunks: 0,
  vector_search_latency_p99: 0,
  pdf_extraction_latency_p99: 0,
  outbox_terminal_failed_total: 0,
})

const history = ref<MetricSnapshot[]>([])
const isLive = ref(true)
let refreshTimer: ReturnType<typeof setInterval> | null = null

const metricCards = computed(() => [
  {
    key: 'tokio_blocking_queue_depth' as const,
    label: 'Tokio Blocking Queue Depth',
    value: metrics.value.tokio_blocking_queue_depth,
    threshold: 6,
    unit: '',
    format: 'integer',
  },
  {
    key: 'outbox_compensation_hit_rate' as const,
    label: 'Outbox Compensation Hit Rate',
    value: metrics.value.outbox_compensation_hit_rate,
    threshold: 0.05,
    unit: '%',
    format: 'percent',
  },
  {
    key: 'ast_chunker_fallback_rate' as const,
    label: 'AST Chunker Fallback Rate',
    value: metrics.value.ast_chunker_fallback_rate,
    threshold: 0.10,
    unit: '%',
    format: 'percent',
  },
  {
    key: 'db_seq_reconciliation_diff' as const,
    label: 'DB Seq Reconciliation Diff',
    value: metrics.value.db_seq_reconciliation_diff,
    threshold: 0,
    unit: '',
    format: 'integer',
  },
  {
    key: 'vector_gc_pending_chunks' as const,
    label: 'Vector GC Pending Chunks',
    value: metrics.value.vector_gc_pending_chunks,
    threshold: 100,
    unit: '',
    format: 'integer',
  },
  {
    key: 'vector_search_latency_p99' as const,
    label: 'Vector Search Latency P99',
    value: metrics.value.vector_search_latency_p99,
    threshold: 10,
    unit: 'ms',
    format: 'decimal',
  },
  {
    key: 'pdf_extraction_latency_p99' as const,
    label: 'PDF Extraction Latency P99',
    value: metrics.value.pdf_extraction_latency_p99,
    threshold: 100,
    unit: 'ms',
    format: 'decimal',
  },
  {
    key: 'outbox_terminal_failed_total' as const,
    label: 'Outbox Terminal Failed Total',
    value: metrics.value.outbox_terminal_failed_total,
    threshold: 0,
    unit: '',
    format: 'integer',
  },
])

const formatValue = (value: number, format: string): string => {
  if (format === 'percent') return (value * 100).toFixed(1)
  if (format === 'decimal') return value.toFixed(1)
  return value.toLocaleString()
}

const isOverThreshold = (value: number, threshold: number): boolean => {
  return value > threshold
}

const refresh = async () => {
  try {
    const response = await fetch('/api/v1/x2text/metrics')
    if (response.ok) {
      const data = await response.json()
      metrics.value = data
      history.value.push({ ...data })
      if (history.value.length > 60) history.value.shift()
    }
  } catch {
    // Mock data for demo
    metrics.value = {
      tokio_blocking_queue_depth: Math.floor(Math.random() * 8),
      outbox_compensation_hit_rate: Math.random() * 0.1,
      ast_chunker_fallback_rate: Math.random() * 0.05,
      db_seq_reconciliation_diff: Math.floor(Math.random() * 3),
      vector_gc_pending_chunks: Math.floor(Math.random() * 50),
      vector_search_latency_p99: Math.random() * 15,
      pdf_extraction_latency_p99: Math.random() * 120,
      outbox_terminal_failed_total: Math.floor(Math.random() * 2),
    }
    history.value.push({ ...metrics.value })
    if (history.value.length > 60) history.value.shift()
  }
}

onMounted(() => {
  refresh()
  if (isLive.value) {
    refreshTimer = setInterval(refresh, 10000)
  }
})

onUnmounted(() => {
  if (refreshTimer) clearInterval(refreshTimer)
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-xl font-semibold text-text-primary font-sans">OBSERVABILITY DASHBOARD</h1>
        <p class="text-sm text-text-muted mt-1">8-core system metrics with real-time monitoring</p>
      </div>
      <div class="flex items-center gap-3">
        <span
          v-if="isLive"
          class="inline-flex items-center gap-1.5 px-2 py-1 rounded text-xs font-mono bg-success/20 text-success"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-success animate-pulse"></span>
          LIVE
        </span>
        <button
          @click="refresh"
          class="px-3 py-1.5 text-sm bg-surface border border-border rounded hover:bg-surface-hover transition-colors"
        >
          Refresh
        </button>
      </div>
    </div>

    <!-- Metric Cards Grid -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <div
        v-for="card in metricCards"
        :key="card.key"
        class="bg-surface border border-border rounded-lg p-4 transition-colors"
        :class="isOverThreshold(card.value, card.threshold) ? 'border-error/50' : ''"
      >
        <div class="flex items-center justify-between mb-3">
          <span class="text-xs text-text-muted uppercase tracking-wider font-sans leading-tight">
            {{ card.label }}
          </span>
          <span
            v-if="isOverThreshold(card.value, card.threshold)"
            class="w-2 h-2 rounded-full bg-error animate-pulse flex-shrink-0"
          ></span>
          <span v-else class="w-2 h-2 rounded-full bg-success flex-shrink-0"></span>
        </div>
        <div
          class="text-2xl font-bold font-mono tabular-nums"
          :class="isOverThreshold(card.value, card.threshold) ? 'text-error' : 'text-text-primary'"
        >
          {{ formatValue(card.value, card.format) }}<span class="text-sm font-normal text-text-muted">{{ card.unit }}</span>
        </div>
        <!-- Threshold line indicator -->
        <div class="mt-3 w-full h-1.5 bg-surface-hover rounded-full overflow-hidden">
          <div
            class="h-full rounded-full transition-all duration-500"
            :class="isOverThreshold(card.value, card.threshold) ? 'bg-error' : 'bg-primary'"
            :style="{ width: `${Math.min((card.value / (card.threshold * 2 || 1)) * 100, 100)}%` }"
          ></div>
        </div>
        <div class="mt-1 text-xs text-text-muted font-mono">
          threshold: {{ formatValue(card.threshold, card.format) }}{{ card.unit }}
        </div>
      </div>
    </div>

    <!-- History Sparklines -->
    <div class="bg-surface border border-border rounded-lg p-4">
      <h3 class="text-sm font-medium text-text-primary mb-4 font-sans">Metric History (last 60 samples)</h3>
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div v-for="card in metricCards.slice(0, 4)" :key="card.key">
          <div class="text-xs text-text-muted mb-1 font-sans">{{ card.label }}</div>
          <div class="h-16 flex items-end gap-px">
            <div
              v-for="(sample, idx) in history.slice(-30)"
              :key="idx"
              class="flex-1 rounded-t transition-all duration-300"
              :class="isOverThreshold(sample[card.key], card.threshold) ? 'bg-error/60' : 'bg-primary/60'"
              :style="{
                height: `${Math.min((sample[card.key] / (card.threshold * 2 || 1)) * 100, 100)}%`,
                minHeight: '2px'
              }"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
