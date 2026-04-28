import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { OutboxRecord } from '@/types/generated'

export interface OutboxStatusDistribution {
  pending: number
  processing: number
  completed: number
  failed: number
  terminal_failed: number
}

export const useOutboxStore = defineStore('outbox', () => {
  // ── State ─────────────────────────────────────────────────────────────
  const records = ref<OutboxRecord[]>([])
  const compensationHitRate = ref<number>(0)
  const loading = ref<boolean>(false)
  const error = ref<string | null>(null)

  // ── Getters ───────────────────────────────────────────────────────────

  const statusDistribution = computed<OutboxStatusDistribution>(() => {
    const dist: OutboxStatusDistribution = {
      pending: 0,
      processing: 0,
      completed: 0,
      failed: 0,
      terminal_failed: 0,
    }
    for (const record of records.value) {
      dist[record.status]++
    }
    return dist
  })

  const terminalFailed = computed<OutboxRecord[]>(() => {
    return records.value.filter((r) => r.status === 'terminal_failed')
  })

  const isOverThreshold = computed<boolean>(() => {
    return compensationHitRate.value >= 0.8
  })

  // ── Actions ───────────────────────────────────────────────────────────

  async function refresh(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const response = await fetch('/api/v1/outbox')
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const data = (await response.json()) as { records: OutboxRecord[]; compensation_hit_rate: number }
      records.value = data.records
      compensationHitRate.value = data.compensation_hit_rate
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load outbox'
    } finally {
      loading.value = false
    }
  }

  async function retryRecord(id: string): Promise<void> {
    try {
      const response = await fetch(`/api/v1/outbox/${id}/retry`, { method: 'POST' })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      await refresh()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to retry record'
    }
  }

  async function markTerminal(id: string): Promise<void> {
    try {
      const response = await fetch(`/api/v1/outbox/${id}/terminal`, { method: 'POST' })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      await refresh()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to mark terminal'
    }
  }

  return {
    records,
    compensationHitRate,
    loading,
    error,
    statusDistribution,
    terminalFailed,
    isOverThreshold,
    refresh,
    retryRecord,
    markTerminal,
  }
})
