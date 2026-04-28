import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import type { AccessLevel } from '@/types/generated'

declare module 'vue-router' {
  interface RouteMeta {
    title?: string
    requiresAuth?: boolean
    accessLevel?: AccessLevel
  }
}

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
      path: '/engines',
      name: 'engines',
      component: () => import('@/views/EnginesView.vue')
    },
    {
      path: '/stats',
      name: 'stats',
      component: () => import('@/views/StatsView.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue')
    },
    {
      path: '/mcp-tools',
      name: 'mcp-tools',
      component: () => import('@/views/McpToolsView.vue')
    },
    {
      path: '/audit-logs',
      name: 'audit-logs',
      component: () => import('@/views/AuditLogsView.vue')
    },
    {
      path: '/batch',
      name: 'batch',
      component: () => import('@/views/BatchProcessView.vue')
    },
    {
      path: '/plugins',
      name: 'plugins',
      component: () => import('@/views/PluginsView.vue')
    },
    {
      path: '/outbox',
      name: 'outbox',
      component: () => import('@/views/OutboxView.vue'),
      meta: { title: 'Outbox Monitor' }
    },
    {
      path: '/observability',
      name: 'observability',
      component: () => import('@/views/ObservabilityView.vue'),
      meta: { title: 'Observability' }
    },
    {
      path: '/reconciliation',
      name: 'reconciliation',
      component: () => import('@/views/ReconciliationView.vue'),
      meta: { title: 'Reconciliation' }
    },
    {
      path: '/pipeline/:documentId',
      name: 'pipeline',
      component: () => import('@/views/PipelineView.vue'),
      meta: { title: 'Pipeline' }
    },
    {
      path: '/rbac',
      name: 'rbac',
      component: () => import('@/views/RbacView.vue'),
      meta: { title: 'RBAC', requiresAuth: true }
    },
    {
      path: '/vector-routes',
      name: 'vector-routes',
      component: () => import('@/views/VectorRouteView.vue'),
      meta: { title: 'Vector Routes' }
    }
  ] satisfies RouteRecordRaw[]
})

export default router
