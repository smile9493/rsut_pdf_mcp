<script setup lang="ts">
import type { OutboxStatus } from '@/types/generated'

defineProps<{
  status: OutboxStatus
}>()

const statusConfig: Record<OutboxStatus, { label: string; class: string }> = {
  pending: { label: 'PENDING', class: 'bg-warning/20 text-warning' },
  processing: { label: 'PROCESSING', class: 'bg-info/20 text-info' },
  completed: { label: 'COMPLETED', class: 'bg-success/20 text-success' },
  failed: { label: 'FAILED', class: 'bg-error/20 text-error' },
  terminal_failed: { label: 'TERMINAL', class: 'bg-error/30 text-error font-bold' },
}
</script>

<template>
  <span
    class="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono font-medium"
    :class="statusConfig[status]?.class ?? 'bg-surface-hover text-text-muted'"
  >
    {{ statusConfig[status]?.label ?? status }}
  </span>
</template>
