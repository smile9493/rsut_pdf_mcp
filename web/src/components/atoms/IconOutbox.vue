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
      d="M1 12.75A2.25 2.25 0 013.25 10.5h13.5A2.25 2.25 0 0119 12.75v3a2.25 2.25 0 01-2.25 2.25h-13.5A2.25 2.25 0 011 15.75v-3zm13.5-7.5a.75.75 0 00-.75.75v3.25l-1.72-1.72a.75.75 0 00-1.06 0l-2.5 2.5a.75.75 0 101.06 1.06L11 9.31v3.19a.75.75 0 001.5 0V9.31l1.22 1.22a.75.75 0 001.06-1.06l-2.5-2.5a.75.75 0 00-1.06 0L8.25 8.81V6a.75.75 0 00-.75-.75h-4.5a.75.75 0 000 1.5h3.75v2.25a.75.75 0 001.5 0V6.75a.75.75 0 00-.75-.75h-3.75a.75.75 0 000 1.5H4.5v2.25c0 .414.336.75.75.75h8.25z"
      clip-rule="evenodd"
      fill="currentColor"
    />
  </svg>
</template>
