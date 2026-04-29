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
    <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
    <rect
      x="8"
      y="2"
      width="8"
      height="4"
      rx="1"
      ry="1"
    />
    <line
      x1="8"
      y1="11"
      x2="16"
      y2="11"
    />
    <line
      x1="8"
      y1="15"
      x2="16"
      y2="15"
    />
  </IconBase>
</template>
