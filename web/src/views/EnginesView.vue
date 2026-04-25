<template>
  <div class="p-2xl">
    <header class="mb-2xl">
      <h1 class="text-h1 font-bold">{{ t('engines.title') }}</h1>
    </header>

    <!-- Engine Status -->
    <section class="mb-2xl">
      <div class="grid grid-cols-3 gap-lg">
        <div
          v-for="engine in engines"
          :key="engine.id"
          class="bg-surface rounded-lg p-lg border border-border"
        >
          <div class="flex items-start justify-between mb-md">
            <div>
              <h3 class="text-h3 font-semibold">{{ engine.name }}</h3>
              <div class="text-micro text-text-muted font-mono">{{ engine.id }}</div>
            </div>
            <span :class="engine.healthy ? 'badge-success' : 'badge-error'">
              {{ engine.healthy ? t('engines.ok') : t('engines.fail') }}
            </span>
          </div>

          <p class="text-sm text-text-secondary mb-lg">{{ engine.description }}</p>

          <div class="space-y-sm text-sm">
            <div class="flex justify-between">
              <span class="text-text-muted">{{ t('dashboard.successRate') }}</span>
              <span class="font-mono">{{ engine.successRate }}%</span>
            </div>
            <div class="flex justify-between">
              <span class="text-text-muted">{{ t('dashboard.avgTime') }}</span>
              <span class="font-mono">{{ engine.avgTime }}ms</span>
            </div>
            <div class="flex justify-between">
              <span class="text-text-muted">{{ t('dashboard.circuit') }}</span>
              <span :class="engine.circuitOpen ? 'text-error' : 'text-success'">
                {{ engine.circuitOpen ? t('dashboard.open') : t('dashboard.closed') }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Routing Strategy -->
    <section class="mb-2xl">
      <h2 class="section-title">{{ t('engines.smartRouting') }}</h2>
      <div class="bg-surface rounded-lg border border-border p-xl">
        <div class="grid grid-cols-2 gap-xl">
          <div>
            <h3 class="text-sm font-medium mb-md text-text-primary">{{ t('engines.rules') }}</h3>
            <div class="space-y-sm text-sm">
              <div class="flex gap-sm">
                <span class="text-primary font-bold">1.</span>
                <span>Small files (&lt;1MB) → pdf-extract</span>
              </div>
              <div class="flex gap-sm">
                <span class="text-primary font-bold">2.</span>
                <span>Pages ≤ 5, simple layout → pdf-extract</span>
              </div>
              <div class="flex gap-sm">
                <span class="text-primary font-bold">3.</span>
                <span>Special encoding (CIDFont) → pdfium</span>
              </div>
              <div class="flex gap-sm">
                <span class="text-primary font-bold">4.</span>
                <span>Default → lopdf</span>
              </div>
            </div>
          </div>

          <div>
            <h3 class="text-sm font-medium mb-md text-text-primary">{{ t('engines.circuitBreaker') }}</h3>
            <div class="space-y-sm text-sm">
              <div class="flex justify-between">
                <span class="text-text-muted">{{ t('engines.failureThreshold') }}</span>
                <span class="font-mono">5</span>
              </div>
              <div class="flex justify-between">
                <span class="text-text-muted">{{ t('engines.cooldown') }}</span>
                <span class="font-mono">60s</span>
              </div>
              <div class="flex justify-between">
                <span class="text-text-muted">{{ t('engines.fallback') }}</span>
                <span class="font-mono">pdfium</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Comparison Table -->
    <section>
      <h2 class="section-title">{{ t('engines.engineComparison') }}</h2>
      <div class="bg-surface rounded-lg border border-border overflow-hidden">
        <table class="table">
          <thead>
            <tr>
              <th>{{ t('engines.feature') }}</th>
              <th>Lopdf</th>
              <th>PDF Extract</th>
              <th>PDFium</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(row, index) in comparisonTable" :key="index">
              <td class="font-medium">{{ row.feature }}</td>
              <td>{{ row.lopdf }}</td>
              <td>{{ row.pdfExtract }}</td>
              <td>{{ row.pdfium }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import axios from 'axios'

const { t } = useI18n()

const engines = ref([
  {
    id: 'lopdf',
    name: 'Lopdf',
    description: 'Layout-aware PDF parsing with MediaBox support',
    healthy: true,
    successRate: 98,
    avgTime: 45,
    circuitOpen: false
  },
  {
    id: 'pdf-extract',
    name: 'PDF Extract',
    description: 'Fast text extraction for simple documents',
    healthy: true,
    successRate: 99,
    avgTime: 25,
    circuitOpen: false
  },
  {
    id: 'pdfium',
    name: 'PDFium',
    description: 'Chrome PDF engine, high compatibility',
    healthy: true,
    successRate: 97,
    avgTime: 65,
    circuitOpen: false
  }
])

const comparisonTable = [
  { feature: 'Speed', lopdf: 'Medium', pdfExtract: 'Fast', pdfium: 'Slow' },
  { feature: 'Layout Aware', lopdf: '✓', pdfExtract: '✗', pdfium: '✓' },
  { feature: 'Special Encoding', lopdf: '✗', pdfExtract: '✗', pdfium: '✓' },
  { feature: 'Complex PDFs', lopdf: '✓', pdfExtract: '✗', pdfium: '✓' },
  { feature: 'Memory', lopdf: 'Medium', pdfExtract: 'Low', pdfium: 'High' },
  { feature: 'Use Case', lopdf: 'General', pdfExtract: 'Simple', pdfium: 'Complex' }
]

const loadEngines = async () => {
  try {
    const response = await axios.get('/api/v1/x2text/adapters')
    if (response.data?.adapters) {
      response.data.adapters.forEach(adapter => {
        const engine = engines.value.find(e => e.id === adapter.id)
        if (engine) {
          engine.name = adapter.name
          engine.description = adapter.description
        }
      })
    }
  } catch {
    // Use defaults
  }
}

onMounted(() => {
  loadEngines()
})
</script>
