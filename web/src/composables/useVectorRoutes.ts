import { ref, onMounted, onUnmounted } from 'vue'

export interface VectorRouteVersion {
  version: number
  label: string
  isCurrent: boolean
  created_at: string
  vector_count: number
  status: 'active' | 'deprecated' | 'pending'
}

export interface VectorRouteData {
  versions: VectorRouteVersion[]
  currentVersion: number
  lazyRecomputeProgress: number
  gcStatus: 'idle' | 'running' | 'pending'
  gcPendingChunks: number
}

export function useVectorRoutes() {
  const data = ref<VectorRouteData>({
    versions: [],
    currentVersion: 1,
    lazyRecomputeProgress: 0,
    gcStatus: 'idle',
    gcPendingChunks: 0
  })
  const loading = ref(true)
  const error = ref<string | null>(null)
  let pollTimer: ReturnType<typeof setInterval> | null = null

  const fetchVectorRouteData = async () => {
    try {
      const [versionsRes, gcRes] = await Promise.all([
        fetch('/api/v1/x2text/vector-routes/versions'),
        fetch('/api/v1/x2text/vector-routes/gc-status')
      ])

      if (versionsRes.ok) {
        const versionsData = await versionsRes.json() as { versions: VectorRouteVersion[]; current_version: number }
        data.value.versions = versionsData.versions
        data.value.currentVersion = versionsData.current_version
      }
      if (gcRes.ok) {
        const gcData = await gcRes.json() as { status: string; pending_chunks: number; progress: number }
        data.value.gcStatus = gcData.status as VectorRouteData['gcStatus']
        data.value.gcPendingChunks = gcData.pending_chunks
        data.value.lazyRecomputeProgress = gcData.progress
      }

      loading.value = false
      error.value = null
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch vector route data'
      loading.value = false
    }
  }

  const startPolling = (intervalMs = 5000) => {
    fetchVectorRouteData()
    pollTimer = setInterval(fetchVectorRouteData, intervalMs)
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
    data,
    loading,
    error,
    fetchVectorRouteData,
    startPolling,
    stopPolling
  }
}
