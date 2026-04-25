<template>
  <div class="min-h-screen flex flex-col bg-bg">
    <!-- Top Navigation Bar -->
    <TopNavBar v-if="!hideTopNav" />

    <div class="flex-1 flex">
      <!-- Side Navigation Bar -->
      <SideNavBar v-if="!hideSideNav" />

      <!-- Main Content Area -->
      <main class="flex-1 overflow-auto bg-bg">
        <div 
          class="h-full"
          :class="contentPadding ? 'p-2xl' : ''"
        >
          <!-- Page Header (optional) -->
          <div v-if="$slots.header" class="mb-xl">
            <slot name="header" />
          </div>

          <!-- Loading State -->
          <div v-if="loading" class="flex items-center justify-center h-64">
            <div class="flex flex-col items-center gap-md">
              <div class="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
              <span class="text-sm text-text-muted">{{ t('common.loading') }}</span>
            </div>
          </div>

          <!-- Error State -->
          <div v-else-if="error" class="flex items-center justify-center h-64">
            <div class="flex flex-col items-center gap-md">
              <ExclamationCircleIcon class="w-12 h-12 text-error" />
              <div class="text-center">
                <div class="text-lg font-medium text-text-primary mb-xs">{{ t('common.error') }}</div>
                <div class="text-sm text-text-muted">{{ error }}</div>
              </div>
              <button 
                @click="$emit('retry')"
                class="btn btn-primary"
              >
                {{ t('common.retry') }}
              </button>
            </div>
          </div>

          <!-- Main Content -->
          <div v-else>
            <slot />
          </div>

          <!-- Footer (optional) -->
          <div v-if="$slots.footer" class="mt-xl">
            <slot name="footer" />
          </div>
        </div>
      </main>
    </div>

    <!-- Global Notifications Toast -->
    <NotificationToast />
  </div>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue'
import { useI18n } from 'vue-i18n'
import { ExclamationCircleIcon } from '@heroicons/vue/24/outline'
import SideNavBar from './SideNavBar.vue'
import TopNavBar from './TopNavBar.vue'
import NotificationToast from './NotificationToast.vue'

const { t } = useI18n()

defineProps({
  hideTopNav: {
    type: Boolean,
    default: false
  },
  hideSideNav: {
    type: Boolean,
    default: false
  },
  contentPadding: {
    type: Boolean,
    default: true
  },
  loading: {
    type: Boolean,
    default: false
  },
  error: {
    type: String,
    default: null
  }
})

defineEmits(['retry'])
</script>

<style scoped>
/* Loading spinner animation */
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1s linear infinite;
}
</style>
