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
      d="M7.455 2.004a.75.75 0 01.26.77 7 7 0 009.958.784l.001-.001.001.001a.75.75 0 011.06 1.06l-.001.001.001.001a8.75 8.75 0 01-11.698 11.187.75.75 0 01-.968-.12 7 7 0 01-1.454-2.326.75.75 0 01.77-.26z"
      clip-rule="evenodd"
      fill="currentColor"
    />
  </svg>
</template>
