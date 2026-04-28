<script setup lang="ts">
withDefaults(defineProps<{
  size?: 16 | 20 | 24
  class?: string
}>(), {
  size: 20,
  class: '',
})
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 20 20"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    :class="$props.class"
    aria-hidden="true"
  >
    <path
      d="M12.232 4.232a3.75 3.75 0 015.304 5.304l-7.024 7.025a3.75 3.75 0 01-4.953.618L3.34 14.556a.75.75 0 11-1.06-1.06l2.375-2.375a3.75 3.75 0 01.618-4.953l7.025-7.024zm4.306 1.955a2.25 2.25 0 00-3.182 0l-2.99 2.99a.75.75 0 001.06 1.06l2.99-2.99a.75.75 0 011.06 0l.001.001a.75.75 0 010 1.06l-2.99 2.99a.75.75 0 001.06 1.06l2.99-2.99a2.25 2.25 0 000-3.182z"
      fill="currentColor"
    />
  </svg>
</template>
