import { defineStore } from 'pinia'
import { ref, type Ref } from 'vue'
import { pdfApi } from '@/composables/useApi'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { AdapterInfoResponse, CacheStatsResponse } from '@/types/api'

export const usePdfStore = defineStore('pdf', () => {
  const adapters: Ref<AdapterInfoResponse[]> = ref([])
  const cacheStats: Ref<CacheStatsResponse | null> = ref(null)
  const isHealthy: Ref<boolean> = ref(false)

  const { loading, error, execute } = useAsyncAction<void>()

  const checkHealth = (): Promise<void> =>
    execute(
      async () => {
        await pdfApi.health()
        isHealthy.value = true
      },
      undefined,
      () => { isHealthy.value = false }
    )

  const loadAdapters = (): Promise<void> =>
    execute(async () => { adapters.value = await pdfApi.listAdapters() })

  const loadCacheStats = (): Promise<void> =>
    execute(
      async () => { cacheStats.value = await pdfApi.getCacheStats() },
      undefined,
      () => {
        cacheStats.value = { size: 0, max_size: 1000, hits: 0, misses: 0, hit_rate: 0 }
      }
    )

  const extractText = (filePath: string, adapter: string | null = null) =>
    execute(() => pdfApi.extractTextFromPath(filePath, adapter))

  const extractStructured = (
    filePath: string,
    adapter: string | null = null,
    enableHighlight: boolean = false
  ) =>
    execute(() => pdfApi.extractStructuredFromPath(filePath, adapter, { enableHighlight }))

  const extractTextFromFile = (file: File, adapter: string | null = null) =>
    execute(() => pdfApi.extractTextFromFile(file, adapter))

  const extractStructuredFromFile = (
    file: File,
    adapter: string | null = null,
    enableHighlight: boolean = false
  ) =>
    execute(() => pdfApi.extractStructuredFromFile(file, adapter, { enableHighlight }))

  return {
    adapters,
    cacheStats,
    isHealthy,
    loading,
    error,
    checkHealth,
    loadAdapters,
    loadCacheStats,
    extractText,
    extractStructured,
    extractTextFromFile,
    extractStructuredFromFile
  }
})
