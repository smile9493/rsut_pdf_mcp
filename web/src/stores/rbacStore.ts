import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UserContext, Permission, AccessLevel } from '@/types/generated'

export const useRbacStore = defineStore('rbac', () => {
  // ── State ─────────────────────────────────────────────────────────────
  const userContext = ref<UserContext | null>(null)
  const isAuthenticated = ref<boolean>(false)
  const permissions = ref<Permission[]>([])

  // ── Getters ───────────────────────────────────────────────────────────

  const maxAccessLevel = computed<AccessLevel | null>(() => {
    return userContext.value?.max_access_level ?? null
  })

  const roles = computed<string[]>(() => {
    return userContext.value?.roles ?? []
  })

  // ── Actions ───────────────────────────────────────────────────────────

  async function login(token: string): Promise<void> {
    try {
      const response = await fetch('/api/v1/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const data = (await response.json()) as { user: UserContext; permissions: Permission[] }
      userContext.value = data.user
      permissions.value = data.permissions
      isAuthenticated.value = true
    } catch {
      userContext.value = null
      permissions.value = []
      isAuthenticated.value = false
    }
  }

  function logout(): void {
    userContext.value = null
    permissions.value = []
    isAuthenticated.value = false
  }

  function checkPermission(permission: Permission): boolean {
    return permissions.value.includes(permission)
  }

  return {
    userContext,
    isAuthenticated,
    permissions,
    maxAccessLevel,
    roles,
    login,
    logout,
    checkPermission,
  }
})
