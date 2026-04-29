<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import IconDocumentText from '@/components/atoms/IconDocumentText.vue'
import IconChevronLeft from '@/components/atoms/IconChevronLeft.vue'
import IconSun from '@/components/atoms/IconSun.vue'
import IconMoon from '@/components/atoms/IconMoon.vue'
import IconHome from '@/components/atoms/IconHome.vue'
import IconExtract from '@/components/atoms/IconExtract.vue'
import IconSearch from '@/components/atoms/IconSearch.vue'
import IconBatch from '@/components/atoms/IconBatch.vue'
import IconMcp from '@/components/atoms/IconMcp.vue'
import IconEngine from '@/components/atoms/IconEngine.vue'
import IconPlugin from '@/components/atoms/IconPlugin.vue'
import IconStats from '@/components/atoms/IconStats.vue'
import IconSettings from '@/components/atoms/IconSettings.vue'
import { setLanguage, getLanguage } from '@/i18n'
import { setTheme, getTheme } from '@/theme'
import type { Component } from 'vue'

const { t } = useI18n()

interface NavItem {
  path: string
  labelKey: string
  icon: Component
}

interface NavSection {
  id: string
  labelKey: string
  items: NavItem[]
}

const navSections: NavSection[] = [
  {
    id: 'main',
    labelKey: 'nav.sections.main',
    items: [
      { path: '/', labelKey: 'nav.dashboard', icon: IconHome }
    ]
  },
  {
    id: 'operations',
    labelKey: 'nav.sections.operations',
    items: [
      { path: '/extract', labelKey: 'nav.extract', icon: IconExtract },
      { path: '/search', labelKey: 'nav.search', icon: IconSearch },
      { path: '/batch', labelKey: 'nav.batch', icon: IconBatch },
      { path: '/mcp-tools', labelKey: 'nav.mcpTools', icon: IconMcp }
    ]
  },
  {
    id: 'settings',
    labelKey: 'nav.sections.settings',
    items: [
      { path: '/settings', labelKey: 'nav.settings', icon: IconSettings }
    ]
  }
]

const isCollapsed = ref(false)
const currentLanguage = ref(getLanguage())
const currentTheme = ref(getTheme())

const toggleCollapse = () => {
  isCollapsed.value = !isCollapsed.value
  localStorage.setItem('sidebar-collapsed', String(isCollapsed.value))
}

const changeLanguage = (e: Event) => {
  const lang = (e.target as HTMLSelectElement).value
  setLanguage(lang)
  currentLanguage.value = lang
}

const changeTheme = (e: Event) => {
  const theme = (e.target as HTMLSelectElement).value
  setTheme(theme as 'light' | 'dark' | 'auto')
  currentTheme.value = theme
}

const toggleTheme = () => {
  const newTheme = currentTheme.value === 'dark' ? 'light' : 'dark'
  setTheme(newTheme)
  currentTheme.value = newTheme
}

onMounted(() => {
  const savedCollapsed = localStorage.getItem('sidebar-collapsed')
  if (savedCollapsed !== null) {
    isCollapsed.value = savedCollapsed === 'true'
  }
})
</script>

<template>
  <aside
    class="bg-surface border-r border-border flex flex-col transition-all duration-300"
    :class="isCollapsed ? 'w-16' : 'w-64'"
  >
    <div class="p-lg border-b border-border">
      <div class="flex items-center gap-sm">
        <div class="w-8 h-8 rounded bg-primary flex items-center justify-center flex-shrink-0">
          <IconDocumentText class="w-5 h-5 text-white" />
        </div>
        <div v-if="!isCollapsed" class="overflow-hidden">
          <div class="text-sm font-semibold text-text-primary">{{ t('common.appName') }}</div>
          <div class="text-micro text-text-muted">{{ t('common.version') }}</div>
        </div>
      </div>
    </div>

    <button
      @click="toggleCollapse"
      class="absolute top-lg right-0 transform translate-x-1/2 w-6 h-6 bg-surface border border-border rounded-full flex items-center justify-center hover:bg-surface-hover transition-colors z-10"
    >
      <IconChevronLeft
        class="w-4 h-4 text-text-muted transition-transform duration-300"
        :class="isCollapsed ? 'rotate-180' : ''"
      />
    </button>

    <nav class="flex-1 overflow-y-auto p-md">
      <div v-for="section in navSections" :key="section.id" class="mb-xl">
        <div
          v-if="!isCollapsed"
          class="text-micro font-semibold text-text-muted uppercase tracking-wider mb-sm px-sm"
        >
          {{ t(section.labelKey) }}
        </div>

        <div class="space-y-xs">
          <router-link
            v-for="item in section.items"
            :key="item.path"
            :to="item.path"
            class="flex items-center gap-sm px-md py-sm rounded text-sm transition-all duration-150 group relative"
            :class="[
              $route.path === item.path || $route.path.startsWith(item.path + '/')
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

    <div class="border-t border-border">
      <div v-if="!isCollapsed" class="p-md space-y-md">
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

      <div v-else class="p-md flex justify-center">
        <button
          @click="toggleTheme"
          class="w-8 h-8 rounded flex items-center justify-center hover:bg-surface-hover transition-colors"
          :title="t('common.toggleTheme')"
        >
          <IconSun v-if="currentTheme === 'dark'" class="w-4 h-4 text-text-muted" />
          <IconMoon v-else class="w-4 h-4 text-text-muted" />
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
aside {
  position: relative;
}
</style>
