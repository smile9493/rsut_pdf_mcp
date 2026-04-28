<script setup lang="ts" generic="T">
import { DynamicScroller, DynamicScrollerItem } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'

const props = withDefaults(defineProps<{
  items: T[]
  itemKey: string | ((item: T) => string | number)
  minHeight?: number
}>(), {
  minHeight: 48
})
</script>

<template>
  <DynamicScroller
    :items="items"
    :min-item-size="minHeight"
    :key-field="typeof itemKey === 'string' ? itemKey : undefined"
    class="virtual-list"
  >
    <template #default="{ item, index, active }">
      <DynamicScrollerItem
        :item="item"
        :active="active"
        :data-index="index"
      >
        <slot name="item" :item="item" :index="index" />
      </DynamicScrollerItem>
    </template>
  </DynamicScroller>
</template>
