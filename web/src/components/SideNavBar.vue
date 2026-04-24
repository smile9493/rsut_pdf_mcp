<template>
  <aside 
    class="bg-surface border-r border-border flex flex-col transition-all duration-300"
    :class="isCollapsed ? 'w-16' : 'w-64'"
  >
    <!-- Logo Section -->
    <div class="p-lg border-b border-border">
      <div class="flex items-center gap-sm">
        <div class="w-8 h-8 rounded bg-primary flex items-center justify-center flex-shrink-0">
          <DocumentTextIcon class="w-5 h-5 text-white" />
        </div>
        <div v-if="!isCollapsed" class="overflow-hidden">
          <div class="text-sm font-semibold text-text-primary">{{ t('common.appName') }}</div>
          <div class="text-micro text-text-muted">{{ t('common.version') }}</div>
        </div>
      </div>
    </div>

    <!-- Collapse Toggle -->
    <button 
      @click="toggleCollapse"
      class="absolute top-lg right-0 transform translate-x-1/2 w-6 h-6 bg-surface border border-border rounded-full flex items-center justify-center hover:bg-surface-hover transition-colors z-10"
    >
      <ChevronLeftIcon 
        class="w-4 h-4 text-text-muted transition-transform duration-300"
        :class="isCollapsed ? 'rotate-180' : ''"
      />
    </button>

    <!-- Navigation Sections -->
    <nav class="flex-1 overflow-y-auto p-md">
      <div v-for="section in navSections" :key="section.id" class="mb-xl">
        <!-- Section Header -->
        <div 
          v-if="!isCollapsed"
          class="text-micro font-semibold text-text-muted uppercase tracking-wider mb-sm px-sm"
        >
          {{ t(section.labelKey) }}
        </div>
        
        <!-- Section Items -->
        <div class="space-y-xs">
          <router-link
            v-for="item in section.items"
            :key="item.path"
            :to="item.path"
            class="flex items-center gap-sm px-md py-sm rounded text-sm transition-all duration-150 group relative"
            :class="[
              $route.path === item.path 
                ? 'bg-primary/20 text-primary-light font-medium' 
                : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover',
              isCollapsed ? 'justify-center' : ''
            ]"
          >
            <component 
              :is="item.icon" 
              class="w-4 h-4 flex-shrink-0"
              :class="$route.path === item.path ? 'text-primary-light' : ''"
            />
            <span v-if="!isCollapsed">{{ t(item.labelKey) }}</span>
            
            <!-- Tooltip for collapsed state -->
            <div 
              v-if="isCollapsed"
              class="absolute left-full ml-sm px-sm py-xs bg-surface border border-border rounded text-sm text-text-primary whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-20"
            >
              {{ t(item.labelKey) }}
            </div>
          </router-link>
        </div>
      </div>
    </nav>

    <!-- Bottom Section -->
    <div class="border-t border-border">
      <!-- Language & Theme Controls -->
      <div v-if="!isCollapsed" class="p-md space-y-md">
        <!-- Language Switcher -->
        <div class="flex items-center justify-between">
          <span class="text-micro text-text-muted">{{ t('common.language') }}</span>
          <select 
            :value="currentLanguage" 
            @change="changeLanguage"
            class="text-sm bg-surface border border-border rounded px-sm py-xs text-text-primary cursor-pointer hover:border-primary transition-colors"
          >
            <option value="zh">中文</option>
            <option value="en">English</option>
          </select>
        </div>
        
        <!-- Theme Switcher -->
        <div class="flex items-center justify-between">
          <span class="text-micro text-text-muted">{{ t('common.theme') }}</span>
          <select 
            :value="currentTheme" 
            @change="changeTheme"
            class="text-sm bg-surface border border-border rounded px-sm py-xs text-text-primary cursor-pointer hover:border-primary transition-colors"
          >
            <option value="light">{{ t('common.light') }}</option>
            <option value="dark">{{ t('common.dark') }}</option>
            <option value="auto">{{ t('common.auto') }}</option>
          </select>
        </div>
      </div>

      <!-- Quick Theme Toggle (collapsed) -->
      <div v-else class="p-md flex justify-center">
        <button 
          @click="toggleTheme"
          class="w-8 h-8 rounded flex items-center justify-center hover:bg-surface-hover transition-colors"
          :title="t('common.toggleTheme')"
        >
          <SunIcon v-if="currentTheme === 'dark'" class="w-4 h-4 text-text-muted" />
          <MoonIcon v-else class="w-4 h-4 text-text-muted" />
        </button>
      </div>

      <!-- API Status -->
      <div class="p-lg border-t border-border">
        <div class="flex items-center gap-sm" :class="isCollapsed ? 'justify-center' : ''">
          <div
            class="w-2 h-2 rounded-full flex-shrink-0 transition-colors duration-300"
            :class="isHealthy ? 'bg-success animate-pulse' : 'bg-error'"
          ></div>
          <span v-if="!isCollapsed" class="text-micro text-text-muted">
            {{ isHealthy ? t('status.apiConnected') : t('status.apiOffline') }}
          </span>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  DocumentTextIcon,
  ChevronLeftIcon,
  SunIcon,
  MoonIcon,
  HomeIcon,
  DocumentMagnifyingGlassIcon,
  Cog6ToothIcon,
  ChartBarIcon,
  CubeIcon,
  WrenchScrewdriverIcon,
  ClipboardDocumentListIcon,
  DocumentDuplicateIcon,
  MagnifyingGlassIcon,
  BoltIcon,
  ServerIcon
} from '@heroicons/vue/24/outline'
import axios from 'axios'
import { setLanguage, getLanguage } from '../i18n'
import { setTheme, getTheme } from '../theme'

const { t } = useI18n()

// Navigation sections inspired by unstract's design
const navSections = [
  {
    id: 'main',
    labelKey: 'nav.sections.main',
    items: [
      { path: '/', labelKey: 'nav.dashboard', icon: HomeIcon }
    ]
  },
  {
    id: 'operations',
    labelKey: 'nav.sections.operations',
    items: [
      { path: '/extract', labelKey: 'nav.extract', icon: DocumentMagnifyingGlassIcon },
      { path: '/search', labelKey: 'nav.search', icon: MagnifyingGlassIcon },
      { path: '/batch', labelKey: 'nav.batch', icon: DocumentDuplicateIcon }
    ]
  },
  {
    id: 'tools',
    labelKey: 'nav.sections.tools',
    items: [
      { path: '/mcp-tools', labelKey: 'nav.mcpTools', icon: WrenchScrewdriverIcon },
      { path: '/engines', labelKey: 'nav.engines', icon: CubeIcon },
      { path: '/plugins', labelKey: 'nav.plugins', icon: BoltIcon }
    ]
  },
  {
    id: 'monitoring',
    labelKey: 'nav.sections.monitoring',
    items: [
      { path: '/stats', labelKey: 'nav.performance', icon: ChartBarIcon },
      { path: '/audit-logs', labelKey: 'nav.auditLogs', icon: ClipboardDocumentListIcon }
    ]
  },
  {
    id: 'settings',
    labelKey: 'nav.sections.settings',
    items: [
      { path: '/settings', labelKey: 'nav.settings', icon: Cog6ToothIcon }
    ]
  }
]

const isHealthy = ref(false)
const isCollapsed = ref(false)
const currentLanguage = ref(getLanguage())
const currentTheme = ref(getTheme())

const checkHealth = async () => {
  try {
    await axios.get('/api/v1/x2text/health', { timeout: 2000 })
    isHealthy.value = true
  } catch {
    isHealthy.value = false
  }
}

const toggleCollapse = () => {
  isCollapsed.value = !isCollapsed.value
  localStorage.setItem('sidebar-collapsed', isCollapsed.value)
}

const changeLanguage = (e) => {
  const lang = e.target.value
  setLanguage(lang)
  currentLanguage.value = lang
}

const changeTheme = (e) => {
  const theme = e.target.value
  setTheme(theme)
  currentTheme.value = theme
}

const toggleTheme = () => {
  const newTheme = currentTheme.value === 'dark' ? 'light' : 'dark'
  setTheme(newTheme)
  currentTheme.value = newTheme
}

onMounted(() => {
  // Restore collapsed state
  const savedCollapsed = localStorage.getItem('sidebar-collapsed')
  if (savedCollapsed !== null) {
    isCollapsed.value = savedCollapsed === 'true'
  }
  
  checkHealth()
  setInterval(checkHealth, 30000)
})
</script>

<style scoped>
/* Smooth transitions for sidebar */
aside {
  position: relative;
}

/* Pulse animation for status indicator */
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style>
