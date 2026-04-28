import { ref, onMounted, onUnmounted } from 'vue'
import type { UserContext, AccessLevel, Permission } from '@/types/generated'

export interface ResourcePermission {
  resource: string
  permissions: Permission[]
}

export interface RbacData {
  userContext: UserContext | null
  resourcePermissions: ResourcePermission[]
  allPermissions: Permission[]
}

export function useRbac() {
  const data = ref<RbacData>({
    userContext: null,
    resourcePermissions: [],
    allPermissions: ['read', 'write', 'delete', 'admin']
  })
  const loading = ref(true)
  const error = ref<string | null>(null)
  let pollTimer: ReturnType<typeof setInterval> | null = null

  const fetchRbacData = async () => {
    try {
      const [contextRes, permsRes] = await Promise.all([
        fetch('/api/v1/x2text/rbac/context'),
        fetch('/api/v1/x2text/rbac/permissions')
      ])

      if (contextRes.ok) {
        data.value.userContext = await contextRes.json() as UserContext
      }
      if (permsRes.ok) {
        const perms = await permsRes.json() as { resources: ResourcePermission[] }
        data.value.resourcePermissions = perms.resources
      }

      loading.value = false
      error.value = null
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch RBAC data'
      loading.value = false
    }
  }

  const startPolling = (intervalMs = 30000) => {
    fetchRbacData()
    pollTimer = setInterval(fetchRbacData, intervalMs)
  }

  const stopPolling = () => {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  const hasPermission = (resource: string, permission: Permission): boolean => {
    const resourcePerm = data.value.resourcePermissions.find(r => r.resource === resource)
    return resourcePerm?.permissions.includes(permission) ?? false
  }

  const getAccessLevelNumber = (level: AccessLevel): number => {
    const levels: Record<AccessLevel, number> = {
      public: 0,
      internal: 1,
      confidential: 2,
      secret: 3
    }
    return levels[level]
  }

  onMounted(() => {
    startPolling()
  })

  onUnmounted(() => {
    stopPolling()
  })

  return {
    data,
    loading,
    error,
    fetchRbacData,
    startPolling,
    stopPolling,
    hasPermission,
    getAccessLevelNumber
  }
}
