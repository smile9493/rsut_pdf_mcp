<template>
  <div class="h-screen flex flex-col bg-bg overflow-hidden">
    <header v-if="!hideTopNav" class="flex-shrink-0">
      <TopNavBar />
    </header>

    <main class="flex-1 min-w-0 overflow-auto">
      <div
        v-if="loading"
        class="flex items-center justify-center h-full"
      >
        <div class="flex flex-col items-center gap-md">
          <div class="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin" />
          <span class="text-sm text-text-muted">{{ t('common.loading') }}</span>
        </div>
      </div>

      <div
        v-else-if="error"
        class="flex items-center justify-center h-full"
      >
        <div class="flex flex-col items-center gap-md text-center">
          <ExclamationCircleIcon class="w-12 h-12 text-error" />
          <div class="text-lg font-medium text-text-primary mb-xs">
            {{ t('common.error') }}
          </div>
          <div class="text-sm text-text-muted">
            {{ error }}
          </div>
          <button
            class="btn btn-primary mt-md"
            @click="$emit('retry')"
          >
            {{ t('common.retry') }}
          </button>
        </div>
      </div>

      <div v-else class="h-full">
        <slot>
          <router-view v-slot="{ Component }">
            <keep-alive :max="5">
              <component :is="Component" />
            </keep-alive>
          </router-view>
        </slot>
      </div>
    </main>

    <NotificationToast />
  </div>
</template>

<script setup lang="ts">
import { ExclamationCircleIcon } from '@heroicons/vue/24/outline'
import TopNavBar from './TopNavBar.vue'
import NotificationToast from './NotificationToast.vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps({
  hideTopNav: {
    type: Boolean,
    default: false
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
