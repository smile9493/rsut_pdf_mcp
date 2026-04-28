import { ref, computed } from 'vue'
import type { ReconciliationStats, ReconciliationCursor } from '@/types/generated'

export interface GhostReference {
  id: string
  sourceTable: string
  sourceId: string
  lanceRowId: string | null
  status: 'missing' | 'orphaned' | 'stale'
}

export function useReconciliation() {
  const diff = ref<ReconciliationStats | null>(null)
  const ghosts = ref<GhostReference[]>([])
  const lastRun = ref<string | null>(null)
  const nextRun = ref<string | null>(null)

  const lastRunFormatted = computed<string | null>(() => {
    if (!lastRun.value) return null
    return new Date(lastRun.value).toLocaleString()
  })

  async function triggerReconciliation(): Promise<ReconciliationStats> {
    const response = await fetch('/api/v1/reconciliation/run', { method: 'POST' })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const result = (await response.json()) as {
      stats: ReconciliationStats
      ghosts: GhostReference[]
      cursor: ReconciliationCursor
    }
    diff.value = result.stats
    ghosts.value = result.ghosts
    lastRun.value = result.cursor.last_run
    return result.stats
  }

  async function repairGhost(ghostId: string): Promise<void> {
    const response = await fetch(`/api/v1/reconciliation/ghosts/${ghostId}/repair`, {
      method: 'POST',
    })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    // Remove repaired ghost from local state
    ghosts.value = ghosts.value.filter((g) => g.id !== ghostId)
  }

  return {
    diff,
    ghosts,
    lastRun,
    nextRun,
    lastRunFormatted,
    triggerReconciliation,
    repairGhost,
  }
}
