import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { BackpressureSnapshot } from '@/types/generated'

export const useBackpressureStore = defineStore('backpressure', () => {
  // ── State ─────────────────────────────────────────────────────────────
  const snapshot = ref<BackpressureSnapshot>({
    compensation_hit_rate: 0,
    current_concurrency: 1,
    suggested_concurrency: 1,
    backpressure_triggered: false,
    alert_triggered: false,
    window_samples: 0,
  })
  const loading = ref<boolean>(false)

  // ── Getters ───────────────────────────────────────────────────────────

  const hitRate = computed<number>(() => snapshot.value.compensation_hit_rate)

  const currentConcurrency = computed<number>(() => snapshot.value.current_concurrency)

  const suggestedConcurrency = computed<number>(() => snapshot.value.suggested_concurrency)

  const isBackpressured = computed<boolean>(() => snapshot.value.backpressure_triggered)

  const isAlert = computed<boolean>(() => snapshot.value.alert_triggered)

  const requestInterval = computed<number>(() => {
    const rate = snapshot.value.compensation_hit_rate
    if (rate < 0.8) return 1_000
    const overshoot = (rate - 0.8) / 0.2
    return Math.min(1_000 + overshoot * 29_000, 30_000)
  })

  // ── Actions ───────────────────────────────────────────────────────────

  function updateSnapshot(newSnapshot: BackpressureSnapshot): void {
    snapshot.value = newSnapshot
  }

  return {
    snapshot,
    loading,
    hitRate,
    currentConcurrency,
    suggestedConcurrency,
    isBackpressured,
    isAlert,
    requestInterval,
    updateSnapshot,
  }
})
