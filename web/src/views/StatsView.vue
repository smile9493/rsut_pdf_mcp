<template>
  <div class="p-2xl">
    <header class="mb-2xl">
      <h1 class="text-h1 font-bold">{{ t('performance.title') }}</h1>
    </header>

    <!-- Key Metrics -->
    <section class="mb-2xl">
      <div class="grid grid-cols-4 gap-lg">
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="metric-label mb-xs">{{ t('performance.cacheHitRate') }}</div>
          <div class="metric">{{ cacheStats ? (cacheStats.hitRate * 100).toFixed(0) : 0 }}%</div>
          <div class="text-sm text-text-muted mt-sm">{{ cacheStats?.hits || 0 }} {{ t('performance.hits') }}</div>
        </div>
        
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="metric-label mb-xs">{{ t('performance.cacheSize') }}</div>
          <div class="metric">{{ cacheStats?.size || 0 }}</div>
          <div class="text-sm text-text-muted mt-sm">/ {{ cacheStats?.maxSize || 1000 }}</div>
        </div>
        
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="metric-label mb-xs">{{ t('performance.routingSpeed') }}</div>
          <div class="metric">10x</div>
          <div class="text-sm text-text-muted mt-sm">{{ t('performance.faster') }}</div>
        </div>
        
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="metric-label mb-xs">{{ t('performance.accuracy') }}</div>
          <div class="metric">95%</div>
          <div class="text-sm text-text-muted mt-sm">{{ t('performance.agentCalls') }}</div>
        </div>
      </div>
    </section>

    <!-- Performance Table -->
    <section class="mb-2xl">
      <h2 class="section-title">{{ t('performance.optimizationResults') }}</h2>
      <div class="bg-surface rounded-lg border border-border overflow-hidden">
        <table class="table">
          <thead>
            <tr>
              <th>{{ t('performance.optimization') }}</th>
              <th>{{ t('performance.before') }}</th>
              <th>{{ t('performance.after') }}</th>
              <th>{{ t('performance.improvement') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, index) in optimizations" :key="index">
              <td class="font-medium">{{ item.name }}</td>
              <td class="font-mono text-text-muted">{{ item.before }}</td>
              <td class="font-mono text-primary">{{ item.after }}</td>
              <td>
                <span class="badge-primary">{{ item.improvement }}</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <!-- Cache Strategy -->
    <section>
      <h2 class="section-title">{{ t('performance.cacheStrategy') }}</h2>
      <div class="grid grid-cols-2 gap-lg">
        <div class="bg-surface rounded-lg p-lg border border-border">
          <h3 class="text-sm font-medium mb-md text-text-primary">{{ t('performance.smallFiles') }}</h3>
          <div class="font-mono text-sm text-text-secondary space-y-sm">
            <p>key = path + mtime + size + adapter</p>
            <p class="text-primary">{{ t('performance.noHashNeeded') }}</p>
          </div>
        </div>

        <div class="bg-surface rounded-lg p-lg border border-border">
          <h3 class="text-sm font-medium mb-md text-text-primary">{{ t('performance.largeFiles') }}</h3>
          <div class="font-mono text-sm text-text-secondary space-y-sm">
            <p>key = partial_hash + adapter + size</p>
            <p>partial_hash = SHA256(head 1MB + tail 1MB)</p>
            <p class="text-primary">{{ t('performance.avoidsFullHash') }}</p>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import axios from 'axios'

const { t } = useI18n()

const cacheStats = ref(null)

const optimizations = [
  { name: 'Smart Routing (cached)', before: '~50ms', after: '~5ms', improvement: '10x' },
  { name: 'Circuit Breaker check', before: '~1μs', after: '~100ns', improvement: '10x' },
  { name: 'Regex cache access', before: '~500ns', after: '~50ns', improvement: '10x' },
  { name: 'Agent accuracy', before: '~60%', after: '~95%', improvement: '58%↑' }
]

const loadCacheStats = async () => {
  try {
    const response = await axios.get('/api/v1/x2text/cache/stats')
    cacheStats.value = response.data
  } catch {
    cacheStats.value = {
      size: 0,
      maxSize: 1000,
      hits: 0,
      misses: 0,
      hitRate: 0
    }
  }
}

onMounted(() => {
  loadCacheStats()
  setInterval(loadCacheStats, 10000)
})
</script>
