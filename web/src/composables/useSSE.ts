import { ref, onUnmounted, type Ref } from 'vue'

export interface SSEOptions {
  url: string
  withCredentials?: boolean
}

export function useSSE<T = Record<string, unknown>>(
  options: SSEOptions,
  onMessage: (data: T) => void,
  onError?: (error: Event) => void
): { connect: () => void; disconnect: () => void; status: Ref<string> } {
  const status = ref<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected')
  let eventSource: EventSource | null = null
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null

  const connect = () => {
    if (eventSource) {
      eventSource.close()
    }

    status.value = 'connecting'
    eventSource = new EventSource(options.url, {
      withCredentials: options.withCredentials ?? false
    })

    eventSource.onopen = () => {
      status.value = 'connected'
    }

    eventSource.onmessage = (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data) as T
        onMessage(data)
      } catch {
        // Non-JSON message, ignore
      }
    }

    eventSource.onerror = (event: Event) => {
      status.value = 'error'
      onError?.(event)
      // Auto-reconnect after 3s
      if (reconnectTimer) clearTimeout(reconnectTimer)
      reconnectTimer = setTimeout(() => {
        connect()
      }, 3000)
    }
  }

  const disconnect = () => {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
    if (eventSource) {
      eventSource.close()
      eventSource = null
    }
    status.value = 'disconnected'
  }

  onUnmounted(() => {
    disconnect()
  })

  return { connect, disconnect, status }
}
