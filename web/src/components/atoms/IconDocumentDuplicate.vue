<script setup lang="ts">
import IconBase from './IconBase.vue'

interface Props {
  size?: number
  color?: string
}

withDefaults(defineProps<Props>(), {
  size: 24,
  color: 'currentColor'
})
</script>

<template>
  <IconBase :size="size" :color="color">
    <rect x="8" y="8" width="12" height="12" rx="2" ry="2" />
    <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
  </IconBase>
</template>
