import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { CircuitState } from '@/providers/McpProvider'

export interface PipelineStage {
  name: string
  status: 'pending' | 'active' | 'completed' | 'failed'
  durationMs: number | null
  confidence: number | null
}

export const usePipelineStore = defineStore('pipeline', () => {
  // ── State ─────────────────────────────────────────────────────────────
  const stages = ref<PipelineStage[]>([])
  const confidence = ref<number>(0)
  const circuitState = ref<CircuitState>('closed')
  const blockingQueueDepth = ref<number>(0)
  const loading = ref<boolean>(false)

  // ── Getters ───────────────────────────────────────────────────────────

  const totalDuration = computed<number>(() => {
    return stages.value.reduce((sum, s) => sum + (s.durationMs ?? 0), 0)
  })

  const intercepted = computed<boolean>(() => {
    return circuitState.value === 'open' || blockingQueueDepth.value > 0
  })

  // ── Actions ───────────────────────────────────────────────────────────

  function subscribe(handler: (stage: PipelineStage) => void): () => void {
    const eventSource = new EventSource('/api/v1/pipeline/stream')

    eventSource.onmessage = (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data) as {
          stage: PipelineStage
          confidence: number
          circuit_state: CircuitState
          blocking_queue_depth: number
        }
        updateStage(data.stage)
        confidence.value = data.confidence
        circuitState.value = data.circuit_state
        blockingQueueDepth.value = data.blocking_queue_depth
        handler(data.stage)
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

  function updateStage(stage: PipelineStage): void {
    const idx = stages.value.findIndex((s) => s.name === stage.name)
    if (idx >= 0) {
      stages.value[idx] = stage
    } else {
      stages.value.push(stage)
    }
  }

  return {
    stages,
    confidence,
    circuitState,
    blockingQueueDepth,
    loading,
    totalDuration,
    intercepted,
    subscribe,
    updateStage,
  }
})
