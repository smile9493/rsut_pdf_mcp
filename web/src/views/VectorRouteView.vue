<script setup lang="ts">
import { ref } from 'vue'

interface VectorVersion {
  id: string
  version: number
  tableName: string
  vectorCount: number
  dimension: number
  isCurrent: boolean
  createdAt: string
  recomputeProgress?: number
  gcStatus: 'idle' | 'running' | 'completed'
}

const versions = ref<VectorVersion[]>([
  { id: 'v-3', version: 3, tableName: 'vectors_v3', vectorCount: 1847, dimension: 1536, isCurrent: true, createdAt: new Date().toISOString(), gcStatus: 'idle' },
  { id: 'v-2', version: 2, tableName: 'vectors_v2', vectorCount: 1200, dimension: 1536, isCurrent: false, createdAt: new Date(Date.now() - 86400000).toISOString(), recomputeProgress: 100, gcStatus: 'completed' },
  { id: 'v-1', version: 1, tableName: 'vectors_v1', vectorCount: 500, dimension: 1536, isCurrent: false, createdAt: new Date(Date.now() - 172800000).toISOString(), recomputeProgress: 100, gcStatus: 'completed' },
])

const formatDate = (dateStr: string): string => {
  try {
    return new Date(dateStr).toLocaleString()
  } catch {
    return dateStr
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div>
      <h1 class="text-xl font-semibold text-text-primary font-sans">PROGRESSIVE VECTOR ROUTES</h1>
      <p class="text-sm text-text-muted mt-1">Manage vector table versions and lazy recompute progress</p>
    </div>

    <!-- Version Table -->
    <div class="bg-surface border border-border rounded-lg overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border">
            <th class="text-left py-3 px-4 text-text-muted font-medium text-xs">VERSION</th>
            <th class="text-left py-3 px-4 text-text-muted font-medium text-xs">TABLE</th>
            <th class="text-right py-3 px-4 text-text-muted font-medium text-xs">VECTORS</th>
            <th class="text-right py-3 px-4 text-text-muted font-medium text-xs">DIMENSION</th>
            <th class="text-left py-3 px-4 text-text-muted font-medium text-xs">STATUS</th>
            <th class="text-left py-3 px-4 text-text-muted font-medium text-xs">GC</th>
            <th class="text-left py-3 px-4 text-text-muted font-medium font-mono text-xs">CREATED</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="ver in versions"
            :key="ver.id"
            class="border-b border-border/50 hover:bg-surface-hover transition-colors"
            :class="ver.isCurrent ? 'bg-primary/5' : ''"
          >
            <td class="py-3 px-4">
              <span class="font-mono text-xs font-bold text-text-primary">v{{ ver.version }}</span>
              <span
                v-if="ver.isCurrent"
                class="ml-2 px-1.5 py-0.5 bg-primary/20 text-primary text-xs rounded font-mono"
              >
                CURRENT
              </span>
            </td>
            <td class="py-3 px-4 font-mono text-xs text-text-secondary">{{ ver.tableName }}</td>
            <td class="py-3 px-4 text-right font-mono text-xs text-text-primary tabular-nums">{{ ver.vectorCount.toLocaleString() }}</td>
            <td class="py-3 px-4 text-right font-mono text-xs text-text-muted">{{ ver.dimension }}</td>
            <td class="py-3 px-4">
              <span
                v-if="ver.recomputeProgress !== undefined && ver.recomputeProgress < 100"
                class="text-xs font-mono text-warning"
              >
                RECOMPUTING {{ ver.recomputeProgress }}%
              </span>
              <span v-else class="text-xs font-mono text-success">READY</span>
            </td>
            <td class="py-3 px-4">
              <span
                class="text-xs font-mono"
                :class="{
                  'text-text-muted': ver.gcStatus === 'idle',
                  'text-warning': ver.gcStatus === 'running',
                  'text-success': ver.gcStatus === 'completed'
                }"
              >
                {{ ver.gcStatus.toUpperCase() }}
              </span>
            </td>
            <td class="py-3 px-4 font-mono text-xs text-text-muted">{{ formatDate(ver.createdAt) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
