import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

// ─── Metric Types ───────────────────────────────────────────────────────────

export interface MetricSnapshot {
  timestamp: string
  extractionLatencyMs: number
  throughputPerMin: number
  errorRate: number
  cacheHitRate: number
  queueDepth: number
  activeWorkers: number
  compensationHitRate: number
  backpressureScore: number
}

export interface MetricAlert {
  id: string
  metric: string
  severity: 'info' | 'warning' | 'critical'
  message: string
  timestamp: string
}

// ─── Store ──────────────────────────────────────────────────────────────────

export const useMetricsStore = defineStore('metrics', () => {
  // ── State ─────────────────────────────────────────────────────────────
  const metrics = ref<MetricSnapshot>({
    timestamp: new Date().toISOString(),
    extractionLatencyMs: 0,
    throughputPerMin: 0,
    errorRate: 0,
    cacheHitRate: 0,
    queueDepth: 0,
    activeWorkers: 0,
    compensationHitRate: 0,
    backpressureScore: 0,
  })
  const history = ref<MetricSnapshot[]>([])
  const alerts = ref<MetricAlert[]>([])
  const loading = ref<boolean>(false)

  // ── Getters ───────────────────────────────────────────────────────────

  const hasAlerts = computed<boolean>(() => alerts.value.length > 0)

  // ── Actions ───────────────────────────────────────────────────────────

  function subscribe(handler: (snapshot: MetricSnapshot) => void): () => void {
    const eventSource = new EventSource('/api/v1/metrics/stream')

    eventSource.onmessage = (event: MessageEvent) => {
      try {
        const snapshot = JSON.parse(event.data) as MetricSnapshot
        updateMetric(snapshot)
        handler(snapshot)
      } catch {
        // Ignore malformed messages
      }
    }

    eventSource.onerror = () => {
      eventSource.close()
    }

    return () => {
      eventSource.close()
    }
  }

  function updateMetric(snapshot: MetricSnapshot): void {
    metrics.value = snapshot
    history.value.push(snapshot)
    // Keep last 100 snapshots
    if (history.value.length > 100) {
      history.value = history.value.slice(-100)
    }
  }

  return {
    metrics,
    history,
    alerts,
    loading,
    hasAlerts,
    subscribe,
    updateMetric,
  }
})
