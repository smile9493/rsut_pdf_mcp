import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface McpClient {
  id: string
  name: string
  platform: 'windows' | 'linux' | 'macos'
  version: string
  connectedAt: string
  lastActivity: string
  status: 'connected' | 'disconnected' | 'error'
  toolCalls: number
  errors: number
}

export interface McpTool {
  name: string
  description: string
  inputSchema: Record<string, unknown>
  callCount: number
  avgDuration: number
  lastCalled: string
}

export interface McpLog {
  id: string
  timestamp: string
  level: 'info' | 'warn' | 'error' | 'debug'
  source: string
  message: string
  details?: Record<string, unknown>
}

export interface McpMetrics {
  totalRequests: number
  requestsPerMinute: number
  avgLatency: number
  p95Latency: number
  errorRate: number
  throughput: number
}

export const useMcpStore = defineStore('mcp', () => {
  const clients = ref<McpClient[]>([])
  const tools = ref<McpTool[]>([])
  const logs = ref<McpLog[]>([])
  const metrics = ref<McpMetrics>({
    totalRequests: 0,
    requestsPerMinute: 0,
    avgLatency: 0,
    p95Latency: 0,
    errorRate: 0,
    throughput: 0
  })

  const config = ref({
    serverCommand: '',
    serverArgs: [] as string[],
    env: {} as Record<string, string>,
    timeout: 30000,
    retryCount: 3
  })

  const connectedClients = computed(() => clients.value.filter(c => c.status === 'connected'))
  const recentLogs = computed(() => logs.value.slice(-100))
  const errorLogs = computed(() => logs.value.filter(l => l.level === 'error'))

  function addClient(client: McpClient) {
    clients.value.push(client)
  }

  function updateClient(id: string, update: Partial<McpClient>) {
    const idx = clients.value.findIndex(c => c.id === id)
    if (idx !== -1) {
      clients.value[idx] = { ...clients.value[idx], ...update }
    }
  }

  function removeClient(id: string) {
    clients.value = clients.value.filter(c => c.id !== id)
  }

  function addLog(log: McpLog) {
    logs.value.push(log)
    if (logs.value.length > 1000) {
      logs.value = logs.value.slice(-1000)
    }
  }

  function updateTools(newTools: McpTool[]) {
    tools.value = newTools
  }

  function updateMetrics(newMetrics: Partial<McpMetrics>) {
    metrics.value = { ...metrics.value, ...newMetrics }
  }

  function updateConfig(newConfig: Partial<typeof config.value>) {
    config.value = { ...config.value, ...newConfig }
  }

  function clearLogs() {
    logs.value = []
  }

  return {
    clients,
    tools,
    logs,
    metrics,
    config,
    connectedClients,
    recentLogs,
    errorLogs,
    addClient,
    updateClient,
    removeClient,
    addLog,
    updateTools,
    updateMetrics,
    updateConfig,
    clearLogs
  }
})
