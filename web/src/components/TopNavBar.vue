<template>
  <header class="h-16 bg-surface border-b border-border flex items-center justify-between px-xl">
    <!-- Left Section: Breadcrumb -->
    <div class="flex items-center gap-md">
      <div class="flex items-center gap-xs text-sm text-text-muted">
        <component :is="currentRouteIcon" class="w-4 h-4" />
        <span>{{ t(currentRouteLabel) }}</span>
      </div>
    </div>

    <!-- Center Section: Search (optional) -->
    <div class="flex-1 max-w-xl mx-xl">
      <div class="relative">
        <MagnifyingGlassIcon class="absolute left-sm top-1/2 transform -translate-y-1/2 w-4 h-4 text-text-muted" />
        <input
          type="text"
          :placeholder="t('common.search')"
          class="w-full pl-lg pr-md py-sm bg-surface border border-border rounded text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-primary focus:ring-1 transition-all"
        />
      </div>
    </div>

    <!-- Right Section: Actions -->
    <div class="flex items-center gap-md">
      <!-- Notifications -->
      <button 
        class="relative w-8 h-8 rounded flex items-center justify-center hover:bg-surface-hover transition-colors"
        @click="showNotifications = !showNotifications"
      >
        <BellIcon class="w-4 h-4 text-text-muted" />
        <span 
          v-if="notificationCount > 0"
          class="absolute -top-xs -right-xs w-4 h-4 bg-error rounded-full text-white text-micro flex items-center justify-center"
        >
          {{ notificationCount }}
        </span>
      </button>

      <!-- Quick Actions -->
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

      <!-- Help -->
      <button 
        class="w-8 h-8 rounded flex items-center justify-center hover:bg-surface-hover transition-colors"
        :title="t('common.help')"
      >
        <QuestionMarkCircleIcon class="w-4 h-4 text-text-muted" />
      </button>

      <!-- User Menu -->
      <div class="relative">
        <button 
          class="flex items-center gap-sm px-sm py-xs rounded hover:bg-surface-hover transition-colors"
          @click="showUserMenu = !showUserMenu"
        >
          <UserCircleIcon class="w-6 h-6 text-text-muted" />
          <ChevronDownIcon class="w-3 h-3 text-text-muted" />
        </button>

        <!-- Dropdown Menu -->
        <div 
          v-if="showUserMenu"
          class="absolute right-0 top-full mt-xs w-48 bg-surface border border-border rounded-lg shadow-lg py-xs z-50"
        >
          <a 
            href="#"
            class="flex items-center gap-sm px-md py-sm text-sm text-text-secondary hover:bg-surface-hover transition-colors"
          >
            <Cog6ToothIcon class="w-4 h-4" />
            {{ t('nav.settings') }}
          </a>
          <a 
            href="#"
            class="flex items-center gap-sm px-md py-sm text-sm text-text-secondary hover:bg-surface-hover transition-colors"
          >
            <DocumentTextIcon class="w-4 h-4" />
            {{ t('common.documentation') }}
          </a>
          <div class="border-t border-border my-xs"></div>
          <a 
            href="#"
            class="flex items-center gap-sm px-md py-sm text-sm text-error hover:bg-surface-hover transition-colors"
          >
            <ArrowRightOnRectangleIcon class="w-4 h-4" />
            {{ t('common.logout') }}
          </a>
        </div>
      </div>
    </div>

    <!-- Notifications Panel -->
    <div 
      v-if="showNotifications"
      class="absolute right-xl top-full mt-xs w-80 bg-surface border border-border rounded-lg shadow-lg z-50"
    >
      <div class="p-md border-b border-border">
        <h3 class="text-sm font-semibold text-text-primary">{{ t('notifications.title') }}</h3>
      </div>
      <div class="max-h-96 overflow-y-auto">
        <div 
          v-for="notification in notifications"
          :key="notification.id"
          class="p-md border-b border-border hover:bg-surface-hover transition-colors cursor-pointer"
        >
          <div class="flex items-start gap-sm">
            <div 
              class="w-2 h-2 rounded-full mt-1.5 flex-shrink-0"
              :class="notification.type === 'success' ? 'bg-success' : notification.type === 'error' ? 'bg-error' : 'bg-info'"
            ></div>
            <div class="flex-1">
              <div class="text-sm text-text-primary">{{ notification.title }}</div>
              <div class="text-micro text-text-muted mt-xs">{{ notification.time }}</div>
            </div>
          </div>
        </div>
        <div v-if="notifications.length === 0" class="p-lg text-center text-sm text-text-muted">
          {{ t('notifications.empty') }}
        </div>
      </div>
    </div>
  </header>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  MagnifyingGlassIcon,
  BellIcon,
  ArrowPathIcon,
  QuestionMarkCircleIcon,
  UserCircleIcon,
  ChevronDownIcon,
  Cog6ToothIcon,
  DocumentTextIcon,
  ArrowRightOnRectangleIcon,
  HomeIcon,
  DocumentMagnifyingGlassIcon,
  CubeIcon,
  ChartBarIcon,
  WrenchScrewdriverIcon,
  ClipboardDocumentListIcon,
  DocumentDuplicateIcon
} from '@heroicons/vue/24/outline'

const { t } = useI18n()
const route = useRoute()

const showNotifications = ref(false)
const showUserMenu = ref(false)
const isRefreshing = ref(false)

const notificationCount = ref(3)
const notifications = ref([
  { id: 1, title: 'PDF extraction completed successfully', time: '2 minutes ago', type: 'success' },
  { id: 2, title: 'New engine available: pdfium-v2', time: '1 hour ago', type: 'info' },
  { id: 3, title: 'Cache hit rate improved to 95%', time: '3 hours ago', type: 'success' }
])

// Route icon mapping
const routeIcons = {
  '/': HomeIcon,
  '/extract': DocumentMagnifyingGlassIcon,
  '/search': MagnifyingGlassIcon,
  '/batch': DocumentDuplicateIcon,
  '/mcp-tools': WrenchScrewdriverIcon,
  '/engines': CubeIcon,
  '/stats': ChartBarIcon,
  '/audit-logs': ClipboardDocumentListIcon,
  '/settings': Cog6ToothIcon
}

// Route label mapping
const routeLabels = {
  '/': 'nav.dashboard',
  '/extract': 'nav.extract',
  '/search': 'nav.search',
  '/batch': 'nav.batch',
  '/mcp-tools': 'nav.mcpTools',
  '/engines': 'nav.engines',
  '/stats': 'nav.performance',
  '/audit-logs': 'nav.auditLogs',
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
/* Click outside to close dropdowns */
header {
  position: relative;
}

/* Animation for refresh icon */
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
