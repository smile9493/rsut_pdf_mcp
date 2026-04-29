import { ref, computed } from 'vue'

export interface AlertAggregatorConfig {
  windowMs: number
  maxPerWindow: number
  cooldownMs: number
}

export interface AggregatedAlert {
  id: string
  metric: string
  severity: 'info' | 'warning' | 'critical'
  message: string
  timestamp: string
  count: number
}

export function useAlertAggregator(config?: Partial<AlertAggregatorConfig>) {
  const resolvedConfig: AlertAggregatorConfig = {
    windowMs: config?.windowMs ?? 60_000,
    maxPerWindow: config?.maxPerWindow ?? 10,
    cooldownMs: config?.cooldownMs ?? 5_000,
  }

  const alerts = ref<AggregatedAlert[]>([])
  const lastPushTime = ref<number>(0)

  const hasAlerts = computed<boolean>(() => alerts.value.length > 0)

  function push(alert: Omit<AggregatedAlert, 'id' | 'count'>): boolean {
    const now = Date.now()

    // Cooldown check
    if (now - lastPushTime.value < resolvedConfig.cooldownMs) {
      return false
    }

    // Window check
    const windowStart = now - resolvedConfig.windowMs
    const recentAlerts = alerts.value.filter((a) => new Date(a.timestamp).getTime() > windowStart)
    if (recentAlerts.length >= resolvedConfig.maxPerWindow) {
      return false
    }

    // Check for duplicate (same metric + severity within window)
    const existing = alerts.value.find(
      (a) => a.metric === alert.metric && a.severity === alert.severity && new Date(a.timestamp).getTime() > windowStart,
    )

    if (existing) {
      existing.count++
      lastPushTime.value = now
      return true
    }

    alerts.value.push({
      ...alert,
      id: `alert-${now}-${Math.random().toString(36).slice(2, 8)}`,
      count: 1,
    })

    lastPushTime.value = now
    return true
  }

  function dismiss(alertId: string): void {
    alerts.value = alerts.value.filter((a) => a.id !== alertId)
  }

  return {
    push,
    alerts,
    dismiss,
    hasAlerts,
  }
}
