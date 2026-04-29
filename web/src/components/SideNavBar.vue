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
import IconPipeline from '@/components/atoms/IconPipeline.vue'
import { setLanguage, getLanguage } from '@/i18n'
import { setTheme, getTheme } from '@/theme'
import type { Component } from 'vue'
import type { SupportedLocale, ThemeValue } from '@/types/generated'

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
      { path: '/mcp-tools', labelKey: 'nav.mcpTools', icon: IconMcp },
      { path: '/wiki', labelKey: 'nav.wiki', icon: IconPipeline }
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
  const lang = (e.target as HTMLSelectElement).value as SupportedLocale
  setLanguage(lang)
  currentLanguage.value = lang
}

const changeTheme = (e: Event) => {
  const theme = (e.target as HTMLSelectElement).value as ThemeValue
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
    class="h-full bg-surface border-r border-border flex flex-col transition-all duration-300 relative"
    :class="isCollapsed ? 'w-16' : 'w-60'"
  >
    <div class="p-4 border-b border-border">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-7 h-7 rounded-md bg-primary flex items-center justify-center flex-shrink-0">
            <IconDocumentText class="w-4 h-4 text-white" />
          </div>
          <div
            v-if="!isCollapsed"
            class="overflow-hidden"
          >
            <div class="text-xs font-semibold text-text-primary">
              {{ t('common.appName') }}
            </div>
          </div>
        </div>
        <button
          class="w-6 h-6 flex items-center justify-center rounded hover:bg-surface-hover transition-colors flex-shrink-0"
          @click="toggleCollapse"
        >
          <IconChevronLeft
            class="w-3.5 h-3.5 text-text-muted transition-transform duration-300"
            :class="isCollapsed ? 'rotate-180' : ''"
          />
        </button>
      </div>
    </div>

    <nav class="flex-1 overflow-y-auto p-2.5">
      <div
        v-for="section in navSections"
        :key="section.id"
        class="mb-4"
      >
        <div
          v-if="!isCollapsed"
          class="text-[10px] font-semibold text-text-muted uppercase tracking-wider mb-1.5 px-2"
        >
          {{ t(section.labelKey) }}
        </div>

        <div class="space-y-0.5">
          <router-link
            v-for="item in section.items"
            :key="item.path"
            :to="item.path"
            class="flex items-center gap-2.5 px-3 py-2 rounded-md text-xs transition-all duration-150 group relative"
            :class="[
              $route.path === item.path || $route.path.startsWith(item.path + '/')
                ? 'bg-primary/15 text-primary-light font-medium'
                : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover',
              isCollapsed ? 'justify-center px-0' : ''
            ]"
          >
            <component
              :is="item.icon"
              class="w-4 h-4 flex-shrink-0"
              :class="$route.path === item.path ? 'text-primary-light' : ''"
            />
            <span v-if="!isCollapsed" class="font-medium">{{ t(item.labelKey) }}</span>

            <div
              v-if="isCollapsed"
              class="absolute left-full ml-2 px-2 py-1 bg-surface border border-border rounded text-xs text-text-primary whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-20"
            >
              {{ t(item.labelKey) }}
            </div>
          </router-link>
        </div>
      </div>
    </nav>

    <div class="border-t border-border p-2.5">
      <div
        v-if="!isCollapsed"
        class="space-y-2"
      >
        <div class="flex items-center justify-between">
          <span class="text-[10px] text-text-muted">{{ t('common.language') }}</span>
          <select
            :value="currentLanguage"
            class="text-xs bg-transparent border border-border rounded px-1.5 py-0.5 text-text-primary cursor-pointer hover:border-primary transition-colors"
            @change="changeLanguage"
          >
            <option value="zh">
              中文
            </option>
            <option value="en">
              English
            </option>
          </select>
        </div>

        <div class="flex items-center justify-between">
          <span class="text-[10px] text-text-muted">{{ t('common.theme') }}</span>
          <select
            :value="currentTheme"
            class="text-xs bg-transparent border border-border rounded px-1.5 py-0.5 text-text-primary cursor-pointer hover:border-primary transition-colors"
            @change="changeTheme"
          >
            <option value="light">
              {{ t('common.light') }}
            </option>
            <option value="dark">
              {{ t('common.dark') }}
            </option>
            <option value="auto">
              {{ t('common.auto') }}
            </option>
          </select>
        </div>
      </div>

      <div
        v-else
        class="flex justify-center"
      >
        <button
          class="w-7 h-7 rounded flex items-center justify-center hover:bg-surface-hover transition-colors"
          :title="t('common.toggleTheme')"
          @click="toggleTheme"
        >
          <IconSun
            v-if="currentTheme === 'dark'"
            class="w-3.5 h-3.5 text-text-muted"
          />
          <IconMoon
            v-else
            class="w-3.5 h-3.5 text-text-muted"
          />
        </button>
      </div>
    </div>
  </aside>
</template>
