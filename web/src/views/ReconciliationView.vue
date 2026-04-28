<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface GhostReference {
  seqId: number
  sourceId: string
  sourceTable: string
  detectedAt: string
  repaired: boolean
}

const diff = ref(2)
const ghosts = ref<GhostReference[]>([
  { seqId: 1843, sourceId: 'doc-abc', sourceTable: 'etl_res', detectedAt: new Date().toISOString(), repaired: false },
  { seqId: 1841, sourceId: 'doc-xyz', sourceTable: 'etl_res', detectedAt: new Date().toISOString(), repaired: false },
])
const lastRun = ref(new Date().toISOString())
const nextRun = ref(new Date(Date.now() + 86400000).toISOString())
const surrealCount = ref(1847)
const lanceCount = ref(1845)
const loading = ref(false)
let refreshTimer: ReturnType<typeof setInterval> | null = null

const hasGhosts = computed(() => diff.value > 0)

const formatDate = (dateStr: string): string => {
  try {
    return new Date(dateStr).toLocaleString()
  } catch {
    return dateStr
  }
}

const repairGhost = async (seqId: number) => {
  const ghost = ghosts.value.find(g => g.seqId === seqId)
  if (ghost) {
    ghost.repaired = true
    diff.value = Math.max(0, diff.value - 1)
    lanceCount.value++
  }
}

const triggerReconciliation = async () => {
  loading.value = true
  try {
    const response = await fetch('/api/v1/x2text/reconciliation/trigger', { method: 'POST' })
    if (response.ok) {
      lastRun.value = new Date().toISOString()
    }
  } catch {
    // Demo: just update the timestamp
    lastRun.value = new Date().toISOString()
  } finally {
    loading.value = false
  }
}

const refresh = async () => {
  try {
    const response = await fetch('/api/v1/x2text/reconciliation/status')
    if (response.ok) {
      const data = await response.json()
      diff.value = data.diff ?? 0
      surrealCount.value = data.surrealCount ?? 0
      lanceCount.value = data.lanceCount ?? 0
    }
  } catch {
    // Use mock data
  }
}

onMounted(() => {
  refresh()
  refreshTimer = setInterval(refresh, 86400000)
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
        <h1 class="text-xl font-semibold text-text-primary font-sans">DUAL-DB RECONCILIATION MONITOR</h1>
        <p class="text-sm text-text-muted mt-1">Cross-reference SurrealDB and LanceDB for consistency</p>
      </div>
      <button
        @click="triggerReconciliation"
        :disabled="loading"
        class="px-4 py-2 text-sm bg-primary text-white rounded hover:bg-primary-light transition-colors disabled:opacity-50"
      >
        {{ loading ? 'Running...' : 'Trigger Reconciliation' }}
      </button>
    </div>

    <!-- Schedule Info -->
    <div class="bg-surface border border-border rounded-lg p-4">
      <div class="grid grid-cols-2 gap-4 text-sm">
        <div>
          <span class="text-text-muted font-sans">Last Reconciliation:</span>
          <span class="ml-2 font-mono text-text-primary">{{ formatDate(lastRun) }}</span>
        </div>
        <div>
          <span class="text-text-muted font-sans">Next Scheduled:</span>
          <span class="ml-2 font-mono text-text-primary">{{ formatDate(nextRun) }}</span>
        </div>
      </div>
    </div>

    <!-- Counts -->
    <div class="grid grid-cols-3 gap-4">
      <div class="bg-surface border border-border rounded-lg p-4">
        <div class="text-xs text-text-muted uppercase tracking-wider mb-2 font-sans">SurrealDB</div>
        <div class="text-2xl font-bold font-mono tabular-nums text-text-primary">
          {{ surrealCount.toLocaleString() }}
        </div>
      </div>
      <div class="bg-surface border border-border rounded-lg p-4">
        <div class="text-xs text-text-muted uppercase tracking-wider mb-2 font-sans">LanceDB</div>
        <div class="text-2xl font-bold font-mono tabular-nums text-text-primary">
          {{ lanceCount.toLocaleString() }}
        </div>
      </div>
      <div
        class="bg-surface border rounded-lg p-4"
        :class="hasGhosts ? 'border-error/50 bg-error/5' : 'border-border'"
      >
        <div class="text-xs text-text-muted uppercase tracking-wider mb-2 font-sans">Diff</div>
        <div
          class="text-2xl font-bold font-mono tabular-nums"
          :class="hasGhosts ? 'text-error' : 'text-success'"
        >
          {{ diff }}
          <span v-if="hasGhosts" class="text-sm font-normal ml-2">GHOST DETECTED</span>
          <span v-else class="text-sm font-normal ml-2 text-success">CLEAN</span>
        </div>
      </div>
    </div>

    <!-- Ghost References -->
    <div class="bg-surface border border-border rounded-lg overflow-hidden">
      <div class="px-4 py-3 border-b border-border">
        <span class="text-sm font-medium text-text-primary font-sans">Ghost References</span>
      </div>
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border">
            <th class="text-left py-2 px-4 text-text-muted font-medium font-mono text-xs">SEQ ID</th>
            <th class="text-left py-2 px-4 text-text-muted font-medium text-xs">SOURCE</th>
            <th class="text-left py-2 px-4 text-text-muted font-medium text-xs">TABLE</th>
            <th class="text-left py-2 px-4 text-text-muted font-medium text-xs">ACTION</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="ghost in ghosts"
            :key="ghost.seqId"
            class="border-b border-border/50 hover:bg-surface-hover transition-colors"
          >
            <td class="py-2 px-4 font-mono text-xs text-text-primary">{{ ghost.seqId }}</td>
            <td class="py-2 px-4 font-mono text-xs text-text-secondary">{{ ghost.sourceId }}</td>
            <td class="py-2 px-4 text-xs text-text-secondary">{{ ghost.sourceTable }}</td>
            <td class="py-2 px-4">
              <button
                v-if="!ghost.repaired"
                @click="repairGhost(ghost.seqId)"
                class="px-2 py-1 text-xs bg-primary/20 text-primary rounded hover:bg-primary/30 transition-colors font-mono"
              >
                REPAIR
              </button>
              <span v-else class="text-xs text-success font-mono">REPAIRED</span>
            </td>
          </tr>
          <tr v-if="ghosts.length === 0">
            <td colspan="4" class="py-8 text-center text-sm text-text-muted">
              No ghost references detected
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
