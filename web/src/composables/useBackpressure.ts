import { computed } from 'vue'
import { useBackpressureStore } from '@/stores/backpressureStore'
import type { BackpressureSnapshot } from '@/types/generated'

export function useBackpressure() {
  const store = useBackpressureStore()

  const snapshot = computed<BackpressureSnapshot>(() => store.snapshot)
  const hitRate = computed<number>(() => store.hitRate)
  const currentConcurrency = computed<number>(() => store.currentConcurrency)
  const suggestedConcurrency = computed<number>(() => store.suggestedConcurrency)
  const isBackpressured = computed<boolean>(() => store.isBackpressured)
  const isAlert = computed<boolean>(() => store.isAlert)
  const requestInterval = computed<number>(() => store.requestInterval)

  return {
    snapshot,
    hitRate,
    currentConcurrency,
    suggestedConcurrency,
    isBackpressured,
    isAlert,
    requestInterval,
  }
}
