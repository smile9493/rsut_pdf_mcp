<template>
  <div class="p-xl">
    <div class="max-w-6xl mx-auto">
      <!-- Header -->
      <div class="mb-xl">
        <h1 class="text-2xl font-bold text-text-primary mb-sm">{{ t('audit.title') }}</h1>
        <p class="text-text-secondary">{{ t('audit.description') }}</p>
      </div>

      <!-- Filters -->
      <div class="bg-surface border border-border rounded-lg p-lg mb-lg">
        <h2 class="text-lg font-semibold text-text-primary mb-md">{{ t('audit.filters') }}</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-md">
          <!-- Action Filter -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-xs">
              {{ t('audit.action') }}
            </label>
            <select
              v-model="filters.action"
              class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option :value="null">{{ t('audit.allActions') }}</option>
              <option v-for="action in actions" :key="action" :value="action">
                {{ action }}
              </option>
            </select>
          </div>

          <!-- Date Range -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-xs">
              {{ t('audit.startDate') }}
            </label>
            <input
              v-model="filters.startDate"
              type="date"
              class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-text-primary mb-xs">
              {{ t('audit.endDate') }}
            </label>
            <input
              v-model="filters.endDate"
              type="date"
              class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          <!-- Search -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-xs">
              {{ t('audit.search') }}
            </label>
            <input
              v-model="filters.search"
              type="text"
              :placeholder="t('audit.searchPlaceholder')"
              class="w-full px-md py-sm border border-border rounded text-text-primary bg-surface focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>
        </div>

        <div class="mt-md flex gap-sm">
          <button
            @click="loadLogs"
            class="px-lg py-sm bg-primary text-white rounded hover:bg-primary-dark transition-colors"
          >
            {{ t('audit.apply') }}
          </button>
          <button
            @click="resetFilters"
            class="px-lg py-sm border border-border text-text-primary rounded hover:bg-surface-hover transition-colors"
          >
            {{ t('audit.reset') }}
          </button>
        </div>
      </div>

      <!-- Stats -->
      <div class="grid grid-cols-1 md:grid-cols-4 gap-md mb-lg">
        <div class="bg-surface border border-border rounded-lg p-md">
          <div class="text-sm text-text-muted mb-xs">{{ t('audit.totalLogs') }}</div>
          <div class="text-2xl font-bold text-text-primary">{{ stats.total }}</div>
        </div>
        <div class="bg-surface border border-border rounded-lg p-md">
          <div class="text-sm text-text-muted mb-xs">{{ t('audit.today') }}</div>
          <div class="text-2xl font-bold text-text-primary">{{ stats.today }}</div>
        </div>
        <div class="bg-surface border border-border rounded-lg p-md">
          <div class="text-sm text-text-muted mb-xs">{{ t('audit.successRate') }}</div>
          <div class="text-2xl font-bold text-success">{{ stats.successRate }}%</div>
        </div>
        <div class="bg-surface border border-border rounded-lg p-md">
          <div class="text-sm text-text-muted mb-xs">{{ t('audit.avgDuration') }}</div>
          <div class="text-2xl font-bold text-text-primary">{{ stats.avgDuration }}ms</div>
        </div>
      </div>

      <!-- Logs Table -->
      <div class="bg-surface border border-border rounded-lg overflow-hidden">
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead class="bg-surface-hover border-b border-border">
              <tr>
                <th class="px-md py-sm text-left text-sm font-medium text-text-primary">
                  {{ t('audit.timestamp') }}
                </th>
                <th class="px-md py-sm text-left text-sm font-medium text-text-primary">
                  {{ t('audit.action') }}
                </th>
                <th class="px-md py-sm text-left text-sm font-medium text-text-primary">
                  {{ t('audit.resource') }}
                </th>
                <th class="px-md py-sm text-left text-sm font-medium text-text-primary">
                  {{ t('audit.status') }}
                </th>
                <th class="px-md py-sm text-left text-sm font-medium text-text-primary">
                  {{ t('audit.duration') }}
                </th>
                <th class="px-md py-sm text-left text-sm font-medium text-text-primary">
                  {{ t('audit.details') }}
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border">
              <tr
                v-for="log in logs"
                :key="log.id"
                class="hover:bg-surface-hover transition-colors"
              >
                <td class="px-md py-sm text-sm text-text-secondary">
                  {{ formatDate(log.timestamp) }}
                </td>
                <td class="px-md py-sm">
                  <span
                    :class="[
                      'px-sm py-xs rounded text-xs font-medium',
                      getActionColor(log.action)
                    ]"
                  >
                    {{ log.action }}
                  </span>
                </td>
                <td class="px-md py-sm text-sm text-text-primary">
                  {{ log.resource || '-' }}
                </td>
                <td class="px-md py-sm">
                  <span
                    :class="[
                      'px-sm py-xs rounded text-xs font-medium',
                      log.success ? 'bg-success/20 text-success' : 'bg-error/20 text-error'
                    ]"
                  >
                    {{ log.success ? t('audit.success') : t('audit.failed') }}
                  </span>
                </td>
                <td class="px-md py-sm text-sm text-text-secondary">
                  {{ log.duration ? `${log.duration}ms` : '-' }}
                </td>
                <td class="px-md py-sm">
                  <button
                    @click="showDetails(log)"
                    class="text-primary hover:text-primary-dark text-sm"
                  >
                    {{ t('audit.viewDetails') }}
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Empty State -->
        <div v-if="logs.length === 0 && !loading" class="p-xl text-center">
          <DocumentTextIcon class="w-12 h-12 text-text-muted mx-auto mb-md" />
          <p class="text-text-secondary">{{ t('audit.noLogs') }}</p>
        </div>

        <!-- Loading -->
        <div v-if="loading" class="p-xl text-center">
          <svg class="animate-spin h-8 w-8 text-primary mx-auto" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
        </div>

        <!-- Pagination -->
        <div v-if="totalPages > 1" class="px-md py-md border-t border-border flex items-center justify-between">
          <div class="text-sm text-text-secondary">
            {{ t('audit.showing') }} {{ (currentPage - 1) * pageSize + 1 }}-{{ Math.min(currentPage * pageSize, totalLogs) }} {{ t('audit.of') }} {{ totalLogs }}
          </div>
          <div class="flex gap-sm">
            <button
              @click="currentPage--"
              :disabled="currentPage === 1"
              :class="[
                'px-md py-xs rounded text-sm',
                currentPage === 1
                  ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                  : 'bg-surface-hover text-text-primary hover:bg-surface'
              ]"
            >
              {{ t('audit.previous') }}
            </button>
            <button
              @click="currentPage++"
              :disabled="currentPage === totalPages"
              :class="[
                'px-md py-xs rounded text-sm',
                currentPage === totalPages
                  ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                  : 'bg-surface-hover text-text-primary hover:bg-surface'
              ]"
            >
              {{ t('audit.next') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Details Modal -->
    <div v-if="selectedLog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="selectedLog = null">
      <div class="bg-surface rounded-lg max-w-2xl w-full max-h-[80vh] overflow-auto m-lg" @click.stop>
        <div class="p-lg border-b border-border flex items-center justify-between">
          <h3 class="text-lg font-semibold text-text-primary">{{ t('audit.logDetails') }}</h3>
          <button @click="selectedLog = null" class="text-text-muted hover:text-text-primary">
            <XMarkIcon class="w-5 h-5" />
          </button>
        </div>
        <div class="p-lg">
          <pre class="text-sm text-text-primary whitespace-pre-wrap">{{ JSON.stringify(selectedLog, null, 2) }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { DocumentTextIcon, XMarkIcon } from '@heroicons/vue/24/outline'
import axios from 'axios'

const { t } = useI18n()

const actions = ['extract_text', 'extract_structured', 'search_keywords', 'extract_keywords', 'get_info']
const filters = ref({
  action: null,
  startDate: null,
  endDate: null,
  search: ''
})

const logs = ref([])
const loading = ref(false)
const currentPage = ref(1)
const pageSize = ref(20)
const totalLogs = ref(0)
const totalPages = ref(0)
const selectedLog = ref(null)

const stats = ref({
  total: 0,
  today: 0,
  successRate: 0,
  avgDuration: 0
})

const formatDate = (date) => {
  return new Date(date).toLocaleString()
}

const getActionColor = (action) => {
  const colors = {
    extract_text: 'bg-blue-100 text-blue-800',
    extract_structured: 'bg-purple-100 text-purple-800',
    search_keywords: 'bg-green-100 text-green-800',
    extract_keywords: 'bg-yellow-100 text-yellow-800',
    get_info: 'bg-gray-100 text-gray-800'
  }
  return colors[action] || 'bg-gray-100 text-gray-800'
}

const loadLogs = async () => {
  loading.value = true
  try {
    const params = {
      page: currentPage.value,
      page_size: pageSize.value,
      ...filters.value
    }

    const response = await axios.get('/api/v1/audit/logs', { params })
    logs.value = response.data.logs || []
    totalLogs.value = response.data.total || 0
    totalPages.value = Math.ceil(totalLogs.value / pageSize.value)

    // Update stats
    stats.value = {
      total: totalLogs.value,
      today: response.data.today_count || 0,
      successRate: response.data.success_rate || 0,
      avgDuration: response.data.avg_duration || 0
    }
  } catch (error) {
    // Mock data for demo
    logs.value = [
      {
        id: 1,
        timestamp: new Date().toISOString(),
        action: 'extract_text',
        resource: 'document.pdf',
        success: true,
        duration: 150
      },
      {
        id: 2,
        timestamp: new Date(Date.now() - 3600000).toISOString(),
        action: 'search_keywords',
        resource: 'report.pdf',
        success: true,
        duration: 200
      }
    ]
    stats.value = {
      total: 2,
      today: 2,
      successRate: 100,
      avgDuration: 175
    }
  } finally {
    loading.value = false
  }
}

const resetFilters = () => {
  filters.value = {
    action: null,
    startDate: null,
    endDate: null,
    search: ''
  }
  currentPage.value = 1
  loadLogs()
}

const showDetails = (log) => {
  selectedLog.value = log
}

watch(currentPage, () => {
  loadLogs()
})

onMounted(() => {
  loadLogs()
})
</script>
