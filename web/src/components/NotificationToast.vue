<template>
  <div class="fixed bottom-xl right-xl z-50 space-y-sm">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="max-w-sm bg-surface border border-border rounded-lg shadow-lg overflow-hidden"
        :class="getToastBorderClass(toast.type)"
      >
        <div class="p-md flex items-start gap-sm">
          <!-- Icon -->
          <component 
            :is="getToastIcon(toast.type)" 
            class="w-5 h-5 flex-shrink-0 mt-0.5"
            :class="getToastIconClass(toast.type)"
          />
          
          <!-- Content -->
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium text-text-primary">{{ toast.title }}</div>
            <div v-if="toast.message" class="text-sm text-text-muted mt-xs">{{ toast.message }}</div>
          </div>
          
          <!-- Close Button -->
          <button 
            @click="removeToast(toast.id)"
            class="flex-shrink-0 w-5 h-5 rounded flex items-center justify-center hover:bg-surface-hover transition-colors"
          >
            <XMarkIcon class="w-3 h-3 text-text-muted" />
          </button>
        </div>

        <!-- Progress Bar (for auto-dismiss) -->
        <div 
          v-if="toast.duration"
          class="h-1 bg-border overflow-hidden"
        >
          <div 
            class="h-full transition-all"
            :class="getToastProgressClass(toast.type)"
            :style="{ width: `${toast.progress}%` }"
          ></div>
        </div>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { 
  XMarkIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  ExclamationTriangleIcon,
  InformationCircleIcon
} from '@heroicons/vue/24/outline'

// Toast store
const toasts = ref([])

// Add toast function
const addToast = (toast) => {
  const id = Date.now()
  const newToast = {
    id,
    progress: 100,
    duration: 5000,
    ...toast
  }
  
  toasts.value.push(newToast)
  
  // Auto dismiss with progress
  if (newToast.duration) {
    const startTime = Date.now()
    const interval = setInterval(() => {
      const elapsed = Date.now() - startTime
      const remaining = Math.max(0, 100 - (elapsed / newToast.duration * 100))
      
      const toastIndex = toasts.value.findIndex(t => t.id === id)
      if (toastIndex !== -1) {
        toasts.value[toastIndex].progress = remaining
      }
      
      if (remaining <= 0) {
        clearInterval(interval)
        removeToast(id)
      }
    }, 50)
  }
  
  return id
}

// Remove toast function
const removeToast = (id) => {
  const index = toasts.value.findIndex(t => t.id === id)
  if (index !== -1) {
    toasts.value.splice(index, 1)
  }
}

// Toast type helpers
const getToastIcon = (type) => {
  const icons = {
    success: CheckCircleIcon,
    error: ExclamationCircleIcon,
    warning: ExclamationTriangleIcon,
    info: InformationCircleIcon
  }
  return icons[type] || InformationCircleIcon
}

const getToastIconClass = (type) => {
  const classes = {
    success: 'text-success',
    error: 'text-error',
    warning: 'text-warning',
    info: 'text-info'
  }
  return classes[type] || 'text-info'
}

const getToastBorderClass = (type) => {
  const classes = {
    success: 'border-l-4 border-l-success',
    error: 'border-l-4 border-l-error',
    warning: 'border-l-4 border-l-warning',
    info: 'border-l-4 border-l-info'
  }
  return classes[type] || ''
}

const getToastProgressClass = (type) => {
  const classes = {
    success: 'bg-success',
    error: 'bg-error',
    warning: 'bg-warning',
    info: 'bg-info'
  }
  return classes[type] || 'bg-info'
}

// Expose methods for global use
defineExpose({
  addToast,
  removeToast,
  success: (title, message) => addToast({ type: 'success', title, message }),
  error: (title, message) => addToast({ type: 'error', title, message }),
  warning: (title, message) => addToast({ type: 'warning', title, message }),
  info: (title, message) => addToast({ type: 'info', title, message })
})
</script>

<style scoped>
/* Toast animations */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(2rem);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(2rem);
}

.toast-move {
  transition: transform 0.3s ease;
}
</style>
