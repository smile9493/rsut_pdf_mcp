import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/HomeView.vue')
    },
    {
      path: '/extract',
      name: 'extract',
      component: () => import('@/views/ExtractView.vue')
    },
    {
      path: '/search',
      name: 'search',
      component: () => import('@/views/SearchView.vue')
    },
    {
      path: '/mcp-tools',
      name: 'mcp-tools',
      component: () => import('@/views/McpToolsView.vue')
    },
    {
      path: '/wiki',
      name: 'wiki',
      component: () => import('@/views/WikiView.vue')
    },
    {
      path: '/batch',
      name: 'batch',
      component: () => import('@/views/BatchProcessView.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue')
    }
  ] satisfies RouteRecordRaw[]
})

export default router
