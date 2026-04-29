import { computed, onUnmounted } from 'vue'
import { usePipelineStore } from '@/stores/pipelineStore'
import type { PipelineStage } from '@/stores/pipelineStore'
import type { CircuitState } from '@/providers/McpProvider'

export function useExtractionPipeline() {
  const store = usePipelineStore()

  const stages = computed<PipelineStage[]>(() => store.stages)
  const totalDuration = computed<number>(() => store.totalDuration)
  const confidence = computed<number>(() => store.confidence)
  const intercepted = computed<boolean>(() => store.intercepted)
  const circuitState = computed<CircuitState>(() => store.circuitState)
  const blockingQueueDepth = computed<number>(() => store.blockingQueueDepth)

  function subscribe(handler: (stage: PipelineStage) => void): () => void {
    return store.subscribe(handler)
  }

  return {
    stages,
    totalDuration,
    confidence,
    intercepted,
    circuitState,
    blockingQueueDepth,
    subscribe,
  }
}
