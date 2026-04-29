<script setup lang="ts">
import type { OutboxRecord } from '@/types/generated'
import OutboxStatusBadge from './OutboxStatusBadge.vue'

defineProps<{
  records: OutboxRecord[]
}>()

const formatDate = (dateStr: string): string => {
  try {
    return new Date(dateStr).toLocaleTimeString()
  } catch {
    return dateStr
  }
}
</script>

<template>
  <div class="overflow-x-auto">
    <table class="w-full text-sm">
      <thead>
        <tr class="border-b border-border">
          <th class="text-left py-2 px-3 text-text-muted font-medium font-mono text-xs">
            ID
          </th>
          <th class="text-left py-2 px-3 text-text-muted font-medium font-mono text-xs">
            SEQ ID
          </th>
          <th class="text-left py-2 px-3 text-text-muted font-medium text-xs">
            TABLE
          </th>
          <th class="text-left py-2 px-3 text-text-muted font-medium text-xs">
            STATUS
          </th>
          <th class="text-left py-2 px-3 text-text-muted font-medium font-mono text-xs">
            UPDATED
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="record in records"
          :key="record.id"
          class="border-b border-border/50 hover:bg-surface-hover transition-colors"
          :class="record.status === 'terminal_failed' ? 'bg-error/5' : ''"
        >
          <td class="py-2 px-3 font-mono text-xs text-text-secondary truncate max-w-[120px]">
            {{ record.id }}
          </td>
          <td class="py-2 px-3 font-mono text-xs text-text-primary">
            {{ record.global_seq_id }}
          </td>
          <td class="py-2 px-3 text-xs text-text-secondary">
            {{ record.source_table }}
          </td>
          <td class="py-2 px-3">
            <OutboxStatusBadge :status="record.status" />
          </td>
          <td class="py-2 px-3 font-mono text-xs text-text-muted">
            {{ formatDate(record.updated_at) }}
          </td>
        </tr>
        <tr v-if="records.length === 0">
          <td
            colspan="5"
            class="py-8 text-center text-sm text-text-muted"
          >
            No records
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
