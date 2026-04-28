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
      d="M11.978 1.18a.75.75 0 00-1.342-.66L6.36 3.82a.75.75 0 00-.348.53l-.836 5.04a.75.75 0 00.934.868l2.248-.612-1.16 3.19a.75.75 0 001.37.504l2.128-5.838a.75.75 0 00-.36-.95l-2.64-1.056.688-4.129 3.178 1.43a.75.75 0 00.968-.454l1.012-3.28a.75.75 0 00-.47-.935zM3.72 13.5a.75.75 0 00-1.44.36l.5 2.5a.75.75 0 00.72.54h3.24a.75.75 0 00.72-.54l.5-2.5a.75.75 0 00-1.44-.36L4.5 15.31l-.78-1.81zm6-6a.75.75 0 00-1.44.36l.5 2.5a.75.75 0 00.72.54h3.24a.75.75 0 00.72-.54l.5-2.5a.75.75 0 00-1.44-.36L10.5 9.31l-.78-1.81z"
      clip-rule="evenodd"
      fill="currentColor"
    />
  </svg>
</template>
