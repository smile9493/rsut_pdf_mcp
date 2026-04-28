<script setup lang="ts">
import { ref, computed } from 'vue'
import type { AccessLevel, Permission } from '@/types/generated'

interface ResourcePermission {
  resource: string
  read: boolean
  write: boolean
  delete: boolean
  admin: boolean
}

const accessLevel = ref<AccessLevel>('internal')
const tokenSource = ref('header')
const roles = ref(['analyst', 'viewer'])

const permissions = ref<ResourcePermission[]>([
  { resource: 'etl_res', read: true, write: true, delete: false, admin: false },
  { resource: 'vectors', read: true, write: false, delete: false, admin: false },
  { resource: 'audit', read: true, write: false, delete: false, admin: false },
  { resource: 'outbox', read: true, write: true, delete: false, admin: false },
  { resource: 'config', read: true, write: false, delete: false, admin: false },
])

const accessLevelLabel = computed(() => {
  const labels: Record<AccessLevel, string> = {
    public: 'Public (0)',
    internal: 'Internal (1)',
    confidential: 'Confidential (2)',
    secret: 'Secret (3)',
  }
  return labels[accessLevel.value]
})

const accessLevelColor = computed(() => {
  const colors: Record<AccessLevel, string> = {
    public: 'text-success',
    internal: 'text-info',
    confidential: 'text-warning',
    secret: 'text-error',
  }
  return colors[accessLevel.value]
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div>
      <h1 class="text-xl font-semibold text-text-primary font-sans">RBAC PERMISSION MATRIX</h1>
      <p class="text-sm text-text-muted mt-1">Role-based access control and permission management</p>
    </div>

    <!-- Permission Matrix -->
    <div class="bg-surface border border-border rounded-lg overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border">
            <th class="text-left py-3 px-4 text-text-muted font-medium text-xs">RESOURCE</th>
            <th class="text-center py-3 px-4 text-text-muted font-medium text-xs">READ</th>
            <th class="text-center py-3 px-4 text-text-muted font-medium text-xs">WRITE</th>
            <th class="text-center py-3 px-4 text-text-muted font-medium text-xs">DELETE</th>
            <th class="text-center py-3 px-4 text-text-muted font-medium text-xs">ADMIN</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="perm in permissions"
            :key="perm.resource"
            class="border-b border-border/50 hover:bg-surface-hover transition-colors"
          >
            <td class="py-3 px-4 font-mono text-xs text-text-primary">{{ perm.resource }}</td>
            <td class="py-3 px-4 text-center">
              <span v-if="perm.read" class="text-success text-lg">\u2705</span>
              <span v-else class="text-error text-lg">\u274C</span>
            </td>
            <td class="py-3 px-4 text-center">
              <span v-if="perm.write" class="text-success text-lg">\u2705</span>
              <span v-else class="text-error text-lg">\u274C</span>
            </td>
            <td class="py-3 px-4 text-center">
              <span v-if="perm.delete" class="text-success text-lg">\u2705</span>
              <span v-else class="text-error text-lg">\u274C</span>
            </td>
            <td class="py-3 px-4 text-center">
              <span v-if="perm.admin" class="text-success text-lg">\u2705</span>
              <span v-else class="text-error text-lg">\u274C</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- User Context Info -->
    <div class="bg-surface border border-border rounded-lg p-4">
      <div class="grid grid-cols-3 gap-4 text-sm">
        <div>
          <span class="text-text-muted font-sans">Access Level:</span>
          <span class="ml-2 font-mono font-bold" :class="accessLevelColor">{{ accessLevelLabel }}</span>
        </div>
        <div>
          <span class="text-text-muted font-sans">Token Source:</span>
          <span class="ml-2 font-mono text-text-primary">{{ tokenSource }}</span>
        </div>
        <div>
          <span class="text-text-muted font-sans">Roles:</span>
          <span class="ml-2 font-mono text-text-primary">{{ roles.join(', ') }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
