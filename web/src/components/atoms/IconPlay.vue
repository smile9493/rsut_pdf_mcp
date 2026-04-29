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
  <IconBase
    :size="size"
    :color="color"
  >
    <polygon points="5 3 19 12 5 21 5 3" />
  </IconBase>
</template>
