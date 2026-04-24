import { defineStore } from 'pinia'
import { ref } from 'vue'
import { pdfApi } from '@/composables/useApi'

export const usePdfStore = defineStore('pdf', () => {
  // State
  const adapters = ref([])
  const cacheStats = ref(null)
  const isHealthy = ref(false)
  const loading = ref(false)
  const error = ref(null)

  // Actions
  const checkHealth = async () => {
    try {
      await pdfApi.health()
      isHealthy.value = true
    } catch {
      isHealthy.value = false
    }
  }

  const loadAdapters = async () => {
    try {
      loading.value = true
      error.value = null
      adapters.value = await pdfApi.listAdapters()
    } catch (err) {
      error.value = err.message
    } finally {
      loading.value = false
    }
  }

  const loadCacheStats = async () => {
    try {
      cacheStats.value = await pdfApi.getCacheStats()
    } catch {
      // Use default values
      cacheStats.value = {
        size: 0,
        maxSize: 1000,
        hits: 0,
        misses: 0,
        hitRate: 0
      }
    }
  }

  const extractText = async (filePath, adapter = null) => {
    try {
      loading.value = true
      error.value = null
      return await pdfApi.extractText(filePath, adapter)
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }

  const extractStructured = async (filePath, adapter = null, enableHighlight = false) => {
    try {
      loading.value = true
      error.value = null
      return await pdfApi.extractStructured(filePath, adapter, enableHighlight)
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    // State
    adapters,
    cacheStats,
    isHealthy,
    loading,
    error,
    // Actions
    checkHealth,
    loadAdapters,
    loadCacheStats,
    extractText,
    extractStructured
  }
})
