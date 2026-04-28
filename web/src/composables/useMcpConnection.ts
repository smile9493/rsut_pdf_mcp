import { ref, onUnmounted } from 'vue'
import { McpProvider } from '@/providers/McpProvider'
import type { CircuitState, McpProviderConfig } from '@/providers/McpProvider'

export function useMcpConnection(config: McpProviderConfig) {
  const provider = new McpProvider(config)
  const connected = ref<boolean>(false)
  const circuitState = ref<CircuitState>('closed')
  const backpressured = ref<boolean>(false)

  // Poll circuit state from provider
  const circuitInterval = setInterval(() => {
    circuitState.value = provider.circuitState
  }, 1_000)

  async function reconnect(): Promise<void> {
    provider.disconnect()
    try {
      await provider.connect()
      connected.value = true
      circuitState.value = provider.circuitState
    } catch {
      connected.value = false
    }
  }

  function destroy(): void {
    clearInterval(circuitInterval)
    provider.destroy()
    connected.value = false
  }

  // Auto-connect
  provider
    .connect()
    .then(() => {
      connected.value = true
      circuitState.value = provider.circuitState
    })
    .catch(() => {
      connected.value = false
    })

  onUnmounted(() => {
    destroy()
  })

  return {
    provider,
    connected,
    circuitState,
    backpressured,
    reconnect,
    destroy,
  }
}
