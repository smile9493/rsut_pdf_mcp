<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

interface PipelineStage {
  name: 'parse' | 'ast_slice' | 'vectorize' | 'llm_extract'
  label: string
  status: 'pending' | 'running' | 'completed' | 'failed' | 'intercepted'
  durationMs: number
  metadata: Record<string, unknown>
}

const documentId = computed(() => (route.params.documentId as string) ?? 'unknown')
const stages = ref<PipelineStage[]>([
  { name: 'parse', label: 'PDF Parse', status: 'completed', durationMs: 23, metadata: {} },
  { name: 'ast_slice', label: 'AST Slice', status: 'completed', durationMs: 45, metadata: {} },
  { name: 'vectorize', label: 'Vectorize', status: 'completed', durationMs: 12, metadata: {} },
])
const confidence = ref(0.92)
const totalDuration = computed(() => stages.value.reduce((sum, s) => sum + s.durationMs, 0))
const intercepted = computed(() => stages.value.some(s => s.status === 'intercepted'))
const blockingQueueDepth = ref({ current: 2, max: 8 })
const circuitState = ref<'closed' | 'open' | 'half-open'>('closed')

const statusIcon = (status: PipelineStage['status']): string => {
  switch (status) {
    case 'completed': return '\u2705'
    case 'failed': return '\u274C'
    case 'running': return '\u23F3'
    case 'intercepted': return '\u26A0\uFE0F'
    default: return '\u25CB'
  }
}

const statusColor = (status: PipelineStage['status']): string => {
  switch (status) {
    case 'completed': return 'text-success'
    case 'failed': return 'text-error'
    case 'running': return 'text-warning'
    case 'intercepted': return 'text-warning'
    default: return 'text-text-muted'
  }
}

const overallStatus = computed(() => {
  if (stages.value.some(s => s.status === 'failed')) return 'FAILED'
  if (stages.value.some(s => s.status === 'intercepted')) return 'INTERCEPTED'
  if (stages.value.every(s => s.status === 'completed')) return 'COMPLETED'
  return 'RUNNING'
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div>
      <h1 class="text-xl font-semibold text-text-primary font-sans">
        EXTRACTION PIPELINE
        <span class="font-mono text-text-muted text-sm ml-2">— {{ documentId }}</span>
      </h1>
    </div>

    <!-- Pipeline Flow -->
    <div class="bg-surface border border-border rounded-lg p-6">
      <div class="flex items-center justify-center gap-4 flex-wrap">
        <div
          v-for="(stage, idx) in stages"
          :key="stage.name"
          class="flex items-center gap-4"
        >
          <!-- Stage Node -->
          <div
            class="bg-surface-hover border border-border rounded-lg p-4 min-w-[140px] text-center transition-colors"
            :class="stage.status === 'completed' ? 'border-success/30' : stage.status === 'failed' ? 'border-error/30' : ''"
          >
            <div class="text-sm font-medium text-text-primary font-sans mb-1">{{ stage.label }}</div>
            <div class="text-lg font-mono font-bold text-text-primary">{{ stage.durationMs }}ms</div>
            <div class="text-xs font-mono mt-1" :class="statusColor(stage.status)">
              {{ statusIcon(stage.status) }} {{ stage.status.toUpperCase() }}
            </div>
          </div>

          <!-- Arrow -->
          <div v-if="idx < stages.length - 1" class="text-text-muted text-lg">\u2192</div>
        </div>
      </div>

      <!-- Confidence Intercept -->
      <div class="mt-6 flex items-center justify-center gap-6">
        <div class="bg-surface-hover border border-border rounded-lg px-4 py-2">
          <span class="text-xs text-text-muted font-sans">Confidence:</span>
          <span
            class="ml-2 font-mono font-bold"
            :class="confidence > 0.85 ? 'text-success' : 'text-error'"
          >
            {{ confidence.toFixed(2) }}
          </span>
          <span class="text-xs text-text-muted ml-1">
            (&gt; 0.85 {{ confidence > 0.85 ? '\u2705' : '\u274C' }})
          </span>
        </div>

        <div class="bg-surface-hover border border-border rounded-lg px-4 py-2">
          <span class="text-xs text-text-muted font-sans">Blocking Queue:</span>
          <span class="ml-2 font-mono font-bold text-text-primary">
            {{ blockingQueueDepth.current }}/{{ blockingQueueDepth.max }}
          </span>
        </div>

        <div class="bg-surface-hover border border-border rounded-lg px-4 py-2">
          <span class="text-xs text-text-muted font-sans">Circuit:</span>
          <span
            class="ml-2 font-mono font-bold"
            :class="circuitState === 'closed' ? 'text-success' : 'text-error'"
          >
            {{ circuitState.toUpperCase() }}
          </span>
        </div>
      </div>

      <!-- Summary -->
      <div class="mt-6 pt-4 border-t border-border flex items-center justify-center gap-6 text-sm">
        <span class="text-text-muted font-sans">Total: <span class="font-mono font-bold text-text-primary">{{ totalDuration }}ms</span></span>
        <span class="text-text-muted font-sans">Confidence: <span class="font-mono font-bold text-text-primary">{{ confidence.toFixed(2) }}</span></span>
        <span class="text-text-muted font-sans">Status: <span class="font-mono font-bold" :class="overallStatus === 'COMPLETED' ? 'text-success' : 'text-error'">{{ overallStatus }}</span></span>
      </div>
    </div>
  </div>
</template>
