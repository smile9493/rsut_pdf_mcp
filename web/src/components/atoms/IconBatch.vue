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
      d="M2 4.5A2.5 2.5 0 014.5 2h11A2.5 2.5 0 0118 4.5v2A2.5 2.5 0 0115.5 9h-11A2.5 2.5 0 012 6.5v-2zM2 12.5A2.5 2.5 0 014.5 10h11a2.5 2.5 0 012.5 2.5v2a2.5 2.5 0 01-2.5 2.5h-11A2.5 2.5 0 012 14.5v-2z"
      fill="currentColor"
    />
  </svg>
</template>
