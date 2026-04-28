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
      fill-rule="evenodd"
      d="M4.5 2A1.5 1.5 0 003 3.5v13A1.5 1.5 0 004.5 18h11a1.5 1.5 0 001.5-1.5V7.621a1.5 1.5 0 00-.44-1.06l-4.12-4.122A1.5 1.5 0 0011.378 2H4.5zM8 4a.75.75 0 000 1.5h4.5a.75.75 0 000-1.5H8zM6.75 6a.75.75 0 01.75-.75h5.5a.75.75 0 010 1.5h-5.5A.75.75 0 016.75 6zm.75 3.75a.75.75 0 000 1.5h5.5a.75.75 0 000-1.5h-5.5zM6 13.25a.75.75 0 01.75-.75h3.5a.75.75 0 010 1.5h-3.5a.75.75 0 01-.75-.75z"
      fill="currentColor"
    />
  </svg>
</template>
