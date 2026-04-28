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
    <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
  </IconBase>
</template>
