import { ref, onMounted, onUnmounted } from 'vue'

export interface PipelineStage {
  name: string
  status: 'pending' | 'running' | 'completed' | 'failed'
  duration_ms: number | null
  started_at: string | null
  completed_at: string | null
  error: string | null
}

export interface PipelineData {
  documentId: string
  documentName: string
  stages: PipelineStage[]
  totalDurationMs: number
  confidence: number
  status: 'pending' | 'running' | 'completed' | 'failed'
}

export function usePipeline(documentId: string) {
  const pipeline = ref<PipelineData | null>(null)
  const loading = ref(true)
  const error = ref<string | null>(null)
  let pollTimer: ReturnType<typeof setInterval> | null = null

  const fetchPipelineData = async () => {
    try {
      const response = await fetch(`/api/v1/x2text/pipeline/${documentId}`)
      if (response.ok) {
        pipeline.value = await response.json() as PipelineData
      } else {
        // Mock pipeline data for demo
        pipeline.value = {
          documentId,
          documentName: `${documentId}.pdf`,
          stages: [
            { name: 'PDF Parse', status: 'completed', duration_ms: 23, started_at: '2026-04-25T03:00:00Z', completed_at: '2026-04-25T03:00:00.023Z', error: null },
            { name: 'AST Slice', status: 'completed', duration_ms: 45, started_at: '2026-04-25T03:00:00.023Z', completed_at: '2026-04-25T03:00:00.068Z', error: null },
            { name: 'Vectorize', status: 'completed', duration_ms: 12, started_at: '2026-04-25T03:00:00.068Z', completed_at: '2026-04-25T03:00:00.080Z', error: null }
          ],
          totalDurationMs: 80,
          confidence: 0.92,
          status: 'completed'
        }
      }
      loading.value = false
      error.value = null
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch pipeline data'
      loading.value = false
    }
  }

  const startPolling = (intervalMs = 5000) => {
    fetchPipelineData()
    pollTimer = setInterval(fetchPipelineData, intervalMs)
  }

  const stopPolling = () => {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  onMounted(() => {
    startPolling()
  })

  onUnmounted(() => {
    stopPolling()
  })

  return {
    pipeline,
    loading,
    error,
    fetchPipelineData,
    startPolling,
    stopPolling
  }
}
