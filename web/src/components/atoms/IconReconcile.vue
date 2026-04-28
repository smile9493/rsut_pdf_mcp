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
      d="M15.312 11.424a5.5 5.5 0 01-9.201 2.466l-.312-.311h2.433a.75.75 0 000-1.5H3.989a.75.75 0 00-.75.75v3.634a.75.75 0 001.5 0v-2.033l.312.311a7 7 0 0011.712-3.138.75.75 0 00-1.449-.39zM11.23 3.047a.75.75 0 01.564-.89 7 7 0 0111.712 3.138.75.75 0 01-1.449.39 5.5 5.5 0 00-9.201-2.466l-.312.311V5.75a.75.75 0 00-1.5 0v3.634a.75.75 0 00.75.75h3.633a.75.75 0 000-1.5H6.189l.312-.311a.75.75 0 00-.288-1.187z"
      clip-rule="evenodd"
      fill="currentColor"
    />
  </svg>
</template>
