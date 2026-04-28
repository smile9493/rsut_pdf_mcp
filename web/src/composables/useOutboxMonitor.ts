import { ref, computed, onMounted, onUnmounted } from 'vue'
import type { OutboxRecord, OutboxStatus } from '@/types/generated'

export interface OutboxStatusCounts {
  pending: number
  processing: number
  completed: number
  failed: number
  terminal_failed: number
}

export interface OutboxMonitorData {
  statusCounts: OutboxStatusCounts
  compensationHitRate: number
  recentRecords: OutboxRecord[]
  isLive: boolean
}

export function useOutboxMonitor() {
  const statusCounts = ref<OutboxStatusCounts>({
    pending: 0,
    processing: 0,
    completed: 0,
    failed: 0,
    terminal_failed: 0
  })
  const compensationHitRate = ref(0)
  const recentRecords = ref<OutboxRecord[]>([])
  const isLive = ref(false)
  const loading = ref(true)
  const error = ref<string | null>(null)
  let pollTimer: ReturnType<typeof setInterval> | null = null

  const totalRecords = computed(() =>
    statusCounts.value.pending +
    statusCounts.value.processing +
    statusCounts.value.completed +
    statusCounts.value.failed +
    statusCounts.value.terminal_failed
  )

  const fetchOutboxData = async () => {
    try {
      const [countsRes, rateRes, recordsRes] = await Promise.all([
        fetch('/api/v1/x2text/outbox/status-counts'),
        fetch('/api/v1/x2text/outbox/compensation-hit-rate'),
        fetch('/api/v1/x2text/outbox/recent?limit=50')
      ])

      if (countsRes.ok) {
        statusCounts.value = await countsRes.json() as OutboxStatusCounts
      }
      if (rateRes.ok) {
        const rateData = await rateRes.json() as { rate: number }
        compensationHitRate.value = rateData.rate
      }
      if (recordsRes.ok) {
        const recordsData = await recordsRes.json() as { records: OutboxRecord[] }
        recentRecords.value = recordsData.records
      }

      loading.value = false
      error.value = null
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch outbox data'
      loading.value = false
    }
  }

  const startPolling = (intervalMs = 5000) => {
    isLive.value = true
    fetchOutboxData()
    pollTimer = setInterval(fetchOutboxData, intervalMs)
  }

  const stopPolling = () => {
    isLive.value = false
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  const getStatusLabel = (status: OutboxStatus): string => {
    const labels: Record<OutboxStatus, string> = {
      pending: 'Pending',
      processing: 'Processing',
      completed: 'Completed',
      failed: 'Failed',
      terminal_failed: 'Terminal Failed'
    }
    return labels[status]
  }

  const getStatusColor = (status: OutboxStatus): string => {
    const colors: Record<OutboxStatus, string> = {
      pending: 'warning',
      processing: 'info',
      completed: 'success',
      failed: 'error',
      terminal_failed: 'error'
    }
    return colors[status]
  }

  onMounted(() => {
    startPolling()
  })

  onUnmounted(() => {
    stopPolling()
  })

  return {
    statusCounts,
    compensationHitRate,
    recentRecords,
    isLive,
    loading,
    error,
    totalRecords,
    fetchOutboxData,
    startPolling,
    stopPolling,
    getStatusLabel,
    getStatusColor
  }
}
