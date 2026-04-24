<template>
  <div class="p-2xl">
    <!-- Header -->
    <header class="mb-3xl">
      <h1 class="text-h1 font-bold text-text-primary mb-sm">
        {{ t('dashboard.title') }}
      </h1>
      <p class="text-text-secondary max-w-2xl">
        {{ t('dashboard.subtitle') }}
      </p>
    </header>

    <!-- Key Metrics -->
    <section class="mb-3xl">
      <div class="grid grid-cols-4 gap-lg">
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="metric-label mb-xs">{{ t('dashboard.extractionSpeed') }}</div>
          <div class="metric">10x</div>
          <div class="text-sm text-text-muted mt-sm">{{ t('dashboard.vsBaseline') }}</div>
        </div>
        
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="metric-label mb-xs">{{ t('dashboard.cacheHitRate') }}</div>
          <div class="metric">95%</div>
          <div class="text-sm text-text-muted mt-sm">{{ t('dashboard.optimized') }}</div>
        </div>
        
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="metric-label mb-xs">{{ t('dashboard.engines') }}</div>
          <div class="metric">3</div>
          <div class="text-sm text-text-muted mt-sm">{{ t('dashboard.active') }}</div>
        </div>
        
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="metric-label mb-xs">{{ t('dashboard.accuracy') }}</div>
          <div class="metric">95%</div>
          <div class="text-sm text-text-muted mt-sm">{{ t('dashboard.agentCalls') }}</div>
        </div>
      </div>
    </section>

    <!-- Main Grid -->
    <div class="grid grid-cols-3 gap-xl">
      <!-- Engines Status -->
      <section class="col-span-2">
        <h2 class="section-title">{{ t('dashboard.engineStatus') }}</h2>
        <div class="bg-surface rounded-lg border border-border overflow-hidden">
          <table class="table">
            <thead>
              <tr>
                <th>{{ t('dashboard.engine') }}</th>
                <th>{{ t('dashboard.status') }}</th>
                <th>{{ t('dashboard.successRate') }}</th>
                <th>{{ t('dashboard.avgTime') }}</th>
                <th>{{ t('dashboard.circuit') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="engine in engines" :key="engine.id">
                <td>
                  <div class="font-medium">{{ engine.name }}</div>
                  <div class="text-micro text-text-muted font-mono">{{ engine.id }}</div>
                </td>
                <td>
                  <span :class="engine.healthy ? 'badge-success' : 'badge-error'">
                    {{ engine.healthy ? t('dashboard.healthy') : t('dashboard.failed') }}
                  </span>
                </td>
                <td>
                  <div class="flex items-center gap-sm">
                    <div class="flex-1 h-1 bg-border rounded overflow-hidden">
                      <div 
                        class="h-full bg-primary"
                        :style="{ width: `${engine.successRate}%` }"
                      ></div>
                    </div>
                    <span class="text-micro font-mono w-12 text-right">{{ engine.successRate }}%</span>
                  </div>
                </td>
                <td class="font-mono">{{ engine.avgTime }}ms</td>
                <td>
                  <span :class="engine.circuitOpen ? 'text-error' : 'text-success'">
                    {{ engine.circuitOpen ? t('dashboard.open') : t('dashboard.closed') }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <!-- Quick Actions -->
      <section>
        <h2 class="section-title">{{ t('dashboard.quickActions') }}</h2>
        <div class="space-y-md">
          <router-link 
            to="/extract" 
            class="block bg-surface rounded-lg p-lg border border-border hover:border-primary transition-colors"
          >
            <div class="flex items-center gap-md">
              <DocumentMagnifyingGlassIcon class="w-6 h-6 text-primary" />
              <div>
                <div class="font-medium text-text-primary">{{ t('dashboard.extractText') }}</div>
                <div class="text-sm text-text-muted">{{ t('dashboard.processPdf') }}</div>
              </div>
            </div>
          </router-link>

          <router-link 
            to="/search" 
            class="block bg-surface rounded-lg p-lg border border-border hover:border-primary transition-colors"
          >
            <div class="flex items-center gap-md">
              <MagnifyingGlassIcon class="w-6 h-6 text-primary" />
              <div>
                <div class="font-medium text-text-primary">{{ t('dashboard.searchKeywords') }}</div>
                <div class="text-sm text-text-muted">{{ t('dashboard.findInDocuments') }}</div>
              </div>
            </div>
          </router-link>

          <router-link 
            to="/stats" 
            class="block bg-surface rounded-lg p-lg border border-border hover:border-primary transition-colors"
          >
            <div class="flex items-center gap-md">
              <ChartBarIcon class="w-6 h-6 text-primary" />
              <div>
                <div class="font-medium text-text-primary">{{ t('dashboard.viewStats') }}</div>
                <div class="text-sm text-text-muted">{{ t('dashboard.performanceMetrics') }}</div>
              </div>
            </div>
          </router-link>
        </div>
      </section>

      <!-- Architecture -->
      <section class="col-span-3">
        <h2 class="section-title">{{ t('dashboard.architecture') }}</h2>
        <div class="bg-surface rounded-lg border border-border p-xl font-mono text-sm text-text-secondary overflow-x-auto">
          <pre>{{ architectureDiagram }}</pre>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  DocumentMagnifyingGlassIcon,
  MagnifyingGlassIcon,
  ChartBarIcon
} from '@heroicons/vue/24/outline'

const { t } = useI18n()

const engines = ref([
  { id: 'lopdf', name: 'Lopdf', healthy: true, successRate: 98, avgTime: 45, circuitOpen: false },
  { id: 'pdf-extract', name: 'PDF Extract', healthy: true, successRate: 99, avgTime: 25, circuitOpen: false },
  { id: 'pdfium', name: 'PDFium', healthy: true, successRate: 97, avgTime: 65, circuitOpen: false }
])

const architectureDiagram = `┌─────────────────────────────────────────────────────────────┐
│                         Clients                              │
│   Cursor / Claude Desktop / Python SDK / REST API           │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    Protocol Layer                            │
│         MCP Server (stdio/SSE)  │  REST Server              │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    Core Service                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  PdfExtractorService                                 │   │
│  │  ├─ SmartRouter    (intelligent engine selection)   │   │
│  │  ├─ CircuitBreaker (fault tolerance)                │   │
│  │  └─ Cache          (Moka concurrent cache)          │   │
│  └─────────────────────────────────────────────────────┘   │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                      Engine Layer                            │
│   lopdf (layout)  │  pdf-extract (fast)  │  pdfium (compat) │
└─────────────────────────────────────────────────────────────┘`
</script>
