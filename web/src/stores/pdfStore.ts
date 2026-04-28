import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Ref } from 'vue'
import { pdfApi } from '@/composables/useApi'
import type { AdapterInfoResponse, CacheStatsResponse } from '@/types/api'

export const usePdfStore = defineStore('pdf', () => {
  const adapters: Ref<AdapterInfoResponse[]> = ref([])
  const cacheStats: Ref<CacheStatsResponse | null> = ref(null)
  const isHealthy: Ref<boolean> = ref(false)
  const loading: Ref<boolean> = ref(false)
  const error: Ref<string | null> = ref(null)

  const checkHealth = async (): Promise<void> => {
    try {
      await pdfApi.health()
      isHealthy.value = true
    } catch {
      isHealthy.value = false
    }
  }

  const loadAdapters = async (): Promise<void> => {
    try {
      loading.value = true
      error.value = null
      adapters.value = await pdfApi.listAdapters()
    } catch (err) {
      error.value = (err as Error).message
    } finally {
      loading.value = false
    }
  }

  const loadCacheStats = async (): Promise<void> => {
    try {
      cacheStats.value = await pdfApi.getCacheStats()
    } catch {
      cacheStats.value = {
        size: 0,
        max_size: 1000,
        hits: 0,
        misses: 0,
        hit_rate: 0
      }
    }
  }

  const extractText = async (filePath: string, adapter: string | null = null): Promise<{ text: string; duration: number }> => {
    try {
      loading.value = true
      error.value = null
      return await pdfApi.extractTextFromPath(filePath, adapter)
    } catch (err) {
      error.value = (err as Error).message
      throw err
    } finally {
      loading.value = false
    }
  }

  const extractStructured = async (
    filePath: string,
    adapter: string | null = null,
    enableHighlight: boolean = false
  ) => {
    try {
      loading.value = true
      error.value = null
      return await pdfApi.extractStructuredFromPath(filePath, adapter, { enableHighlight })
    } catch (err) {
      error.value = (err as Error).message
      throw err
    } finally {
      loading.value = false
    }
  }

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
    extractStructured
  }
})
