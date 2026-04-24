import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '@/views/HomeView.vue'
import ExtractView from '@/views/ExtractView.vue'
import SearchView from '@/views/SearchView.vue'
import EnginesView from '@/views/EnginesView.vue'
import StatsView from '@/views/StatsView.vue'
import SettingsView from '@/views/SettingsView.vue'
import McpToolsView from '@/views/McpToolsView.vue'
import AuditLogsView from '@/views/AuditLogsView.vue'
import BatchProcessView from '@/views/BatchProcessView.vue'
import PluginsView from '@/views/PluginsView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/extract',
      name: 'extract',
      component: ExtractView
    },
    {
      path: '/search',
      name: 'search',
      component: SearchView
    },
    {
      path: '/engines',
      name: 'engines',
      component: EnginesView
    },
    {
      path: '/stats',
      name: 'stats',
      component: StatsView
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView
    },
    {
      path: '/mcp-tools',
      name: 'mcp-tools',
      component: McpToolsView
    },
    {
      path: '/audit-logs',
      name: 'audit-logs',
      component: AuditLogsView
    },
    {
      path: '/batch',
      name: 'batch',
      component: BatchProcessView
    },
    {
      path: '/plugins',
      name: 'plugins',
      component: PluginsView
    }
  ]
})

export default router
