<!--
  PageLayout — Facade 层（破水架构）

  职责：
  - 布局隔离：控制 flex 布局、滚动、侧边栏/顶部导航
  - P0 安全：错误边界、加载状态、空值防护
  - 性能优化：keep-alive 缓存、v-once 静态内容

  截拳道原则：每个组件只做一件事。Layout 只负责布局，不负责业务逻辑。
-->
<template>
  <div class="h-screen flex flex-col bg-bg overflow-hidden">
    <!-- Top Navigation Bar -->
    <header v-if="!hideTopNav" class="flex-shrink-0">
      <TopNavBar />
    </header>

    <div class="flex-1 flex overflow-hidden">
      <!-- Side Navigation Bar -->
      <aside v-if="!hideSideNav" class="flex-shrink-0">
        <SideNavBar />
      </aside>

      <!-- Main Content Area - 严格隔离 -->
      <main class="flex-1 min-w-0 overflow-auto">
        <!-- Loading State -->
        <div
          v-if="loading"
          class="flex items-center justify-center h-full"
        >
          <div class="flex flex-col items-center gap-md">
            <div class="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin" />
            <span class="text-sm text-text-muted">{{ t('common.loading') }}</span>
          </div>
        </div>

        <!-- Error State -->
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

        <!-- Main Content - 路由视图 -->
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
    </div>

    <!-- Global Notifications Toast -->
    <NotificationToast />
  </div>
</template>

<script setup lang="ts">
import { ExclamationCircleIcon } from '@heroicons/vue/24/outline'
import SideNavBar from './SideNavBar.vue'
import TopNavBar from './TopNavBar.vue'
import NotificationToast from './NotificationToast.vue'
import { useI18n } from 'vue-i18n'

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
