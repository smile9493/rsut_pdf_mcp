<template>
  <header class="h-14 bg-surface border-b border-border flex items-center justify-between px-6">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2 text-sm text-text-secondary">
        <component
          :is="currentRouteIcon"
          class="w-4 h-4"
        />
        <span>{{ t(currentRouteLabel) }}</span>
      </div>
    </div>

    <div class="flex items-center gap-3">
      <button
        class="w-8 h-8 rounded flex items-center justify-center hover:bg-surface-hover transition-colors"
        :title="t('common.refresh')"
        @click="handleRefresh"
      >
        <ArrowPathIcon
          class="w-4 h-4 text-text-muted"
          :class="isRefreshing ? 'animate-spin' : ''"
        />
      </button>
    </div>
  </header>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  ArrowPathIcon,
  HomeIcon,
  DocumentMagnifyingGlassIcon,
  MagnifyingGlassIcon,
  DocumentDuplicateIcon,
  WrenchScrewdriverIcon,
  Cog6ToothIcon
} from '@heroicons/vue/24/outline'

const { t } = useI18n()
const route = useRoute()

const isRefreshing = ref(false)

const routeIcons = {
  '/': HomeIcon,
  '/extract': DocumentMagnifyingGlassIcon,
  '/search': MagnifyingGlassIcon,
  '/batch': DocumentDuplicateIcon,
  '/mcp-tools': WrenchScrewdriverIcon,
  '/settings': Cog6ToothIcon
}

const routeLabels = {
  '/': 'nav.dashboard',
  '/extract': 'nav.extract',
  '/search': 'nav.search',
  '/batch': 'nav.batch',
  '/mcp-tools': 'nav.mcpTools',
  '/settings': 'nav.settings'
}

const currentRouteIcon = computed(() => routeIcons[route.path] || HomeIcon)
const currentRouteLabel = computed(() => routeLabels[route.path] || 'nav.dashboard')

const handleRefresh = () => {
  isRefreshing.value = true
  setTimeout(() => {
    isRefreshing.value = false
  }, 1000)
}
</script>

<style scoped>
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1s linear infinite;
}
</style>
