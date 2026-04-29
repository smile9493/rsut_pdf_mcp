import { computed, onUnmounted } from 'vue'
import { useMetricsStore } from '@/stores/metricsStore'
import type { MetricSnapshot, MetricAlert } from '@/stores/metricsStore'

export function useMetrics() {
  const store = useMetricsStore()

  const metrics = computed<MetricSnapshot[]>(() => store.metrics)
  const history = computed<MetricSnapshot[]>(() => store.history)
  const alerts = computed<MetricAlert[]>(() => store.alerts)
  const hasAlerts = computed<boolean>(() => store.hasAlerts)

  function subscribe(handler: (snapshot: MetricSnapshot) => void): () => void {
    store.subscribe(handler)
    return () => {}
  }

  return {
    metrics,
    history,
    alerts,
    hasAlerts,
    subscribe,
  }
}
