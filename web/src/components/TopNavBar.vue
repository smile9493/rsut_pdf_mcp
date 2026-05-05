<template>
  <header class="h-14 bg-surface border-b border-border flex items-center justify-between px-6">
    <div class="flex items-center gap-6">
      <div class="flex items-center gap-2.5">
        <div class="w-7 h-7 rounded-md bg-primary flex items-center justify-center flex-shrink-0">
          <IconDocumentText class="w-4 h-4 text-white" />
        </div>
        <span class="text-sm font-semibold text-text-primary">{{ t('common.appName') }}</span>
      </div>
      
      <nav class="flex items-center gap-1">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs transition-all duration-150"
          :class="$route.path === item.path || $route.path.startsWith(item.path + '/')
            ? 'bg-primary/15 text-primary-light font-medium'
            : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover'"
        >
          <component :is="item.icon" class="w-4 h-4" />
          <span>{{ t(item.labelKey) }}</span>
        </router-link>
      </nav>
    </div>

    <div class="flex items-center gap-3">
      <select
        :value="currentLanguage"
        class="text-xs bg-transparent border border-border rounded px-2 py-1 text-text-primary cursor-pointer hover:border-primary transition-colors"
        @change="changeLanguage"
      >
        <option value="zh">中文</option>
        <option value="en">English</option>
      </select>

      <select
        :value="currentTheme"
        class="text-xs bg-transparent border border-border rounded px-2 py-1 text-text-primary cursor-pointer hover:border-primary transition-colors"
        @change="changeTheme"
      >
        <option value="light">{{ t('common.light') }}</option>
        <option value="dark">{{ t('common.dark') }}</option>
        <option value="auto">{{ t('common.auto') }}</option>
      </select>

      <button
        class="w-8 h-8 rounded flex items-center justify-center hover:bg-surface-hover transition-colors"
        :title="t('common.refresh')"
        @click="handleRefresh"
      >
        <ArrowPathIcon class="w-4 h-4 text-text-muted" :class="isRefreshing ? 'animate-spin' : ''" />
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { setLanguage, getLanguage } from '@/i18n'
import { setTheme, getTheme } from '@/theme'
import ArrowPathIcon from '@heroicons/vue/24/outline/esm/ArrowPathIcon'
import IconDocumentText from '@/components/atoms/IconDocumentText.vue'
import IconHome from '@/components/atoms/IconHome.vue'
import IconExtract from '@/components/atoms/IconExtract.vue'
import IconSearch from '@/components/atoms/IconSearch.vue'
import IconBatch from '@/components/atoms/IconBatch.vue'
import IconMcp from '@/components/atoms/IconMcp.vue'
import IconPipeline from '@/components/atoms/IconPipeline.vue'
import IconSettings from '@/components/atoms/IconSettings.vue'
import type { Component } from 'vue'
import type { SupportedLocale, ThemeValue } from '@/types/generated'

const { t } = useI18n()

interface NavItem {
  path: string
  labelKey: string
  icon: Component
}

const navItems: NavItem[] = [
  { path: '/', labelKey: 'nav.dashboard', icon: IconHome },
  { path: '/extract', labelKey: 'nav.extract', icon: IconExtract },
  { path: '/search', labelKey: 'nav.search', icon: IconSearch },
  { path: '/batch', labelKey: 'nav.batch', icon: IconBatch },
  { path: '/mcp-tools', labelKey: 'nav.mcpTools', icon: IconMcp },
  { path: '/wiki', labelKey: 'nav.wiki', icon: IconPipeline },
  { path: '/settings', labelKey: 'nav.settings', icon: IconSettings }
]

const isRefreshing = ref(false)
const currentLanguage = ref(getLanguage())
const currentTheme = ref(getTheme())

const handleRefresh = () => {
  isRefreshing.value = true
  setTimeout(() => {
    isRefreshing.value = false
  }, 1000)
}

const changeLanguage = (e: Event) => {
  const lang = (e.target as HTMLSelectElement).value as SupportedLocale
  setLanguage(lang)
  currentLanguage.value = lang
}

const changeTheme = (e: Event) => {
  const theme = (e.target as HTMLSelectElement).value as ThemeValue
  setTheme(theme as 'light' | 'dark' | 'auto')
  currentTheme.value = theme
}

onMounted(() => {
  currentLanguage.value = getLanguage()
  currentTheme.value = getTheme()
})
</script>

<style scoped>
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
.animate-spin {
  animation: spin 1s linear infinite;
}
</style>
