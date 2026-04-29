import { ref, computed, onMounted, onUnmounted } from 'vue'

const API_BASE = '/api'

export interface DashboardMetrics {
  totalCalls: number
  avgLatency: number
  successRate: number
  filesProcessed: number
  tools: ToolStat[]
  uptimeSecs: number
}

export interface ToolStat {
  name: string
  calls: number
  latency: number
  successRate: number
}

export interface SystemStatus {
  memoryPercent: number
  pdfiumReady: boolean
  pdfiumVersion: string
  queueLength: number
  vlmEnabled: boolean
  vlmModel: string
  vlmThinking: boolean
  vlmFunctionCall: boolean
  vlmMultiModelRouting: boolean
}

export interface HealthCheck {
  status: string
  mcpHealthy: boolean
  clientConnections: number
  uptimeSecs: number
  version: string
}

export interface LogEntry {
  level: string
  time: string
  message: string
}

export function useDashboard() {
  const loading = ref(false)
  const error = ref<string | null>(null)
  const metrics = ref<DashboardMetrics>({
    totalCalls: 0,
    avgLatency: 0,
    successRate: 100,
    filesProcessed: 0,
    tools: [],
    uptimeSecs: 0
  })

  const status = ref<SystemStatus>({
    memoryPercent: 0,
    pdfiumReady: false,
    pdfiumVersion: 'unknown',
    queueLength: 0,
    vlmEnabled: false,
    vlmModel: 'none',
    vlmThinking: false,
    vlmFunctionCall: false,
    vlmMultiModelRouting: false
  })

  const health = ref<HealthCheck>({
    status: 'unknown',
    mcpHealthy: false,
    clientConnections: 0,
    uptimeSecs: 0,
    version: 'unknown'
  })

  const logs = ref<LogEntry[]>([])

  const uptimeStr = computed(() => {
    const secs = health.value.uptimeSecs || metrics.value.uptimeSecs
    const h = Math.floor(secs / 3600)
    const m = Math.floor((secs % 3600) / 60)
    const s = secs % 60
    if (h > 0) return `${h}h ${m}m`
    if (m > 0) return `${m}m ${s}s`
    return `${s}s`
  })

  async function fetchMetrics() {
    try {
      const res = await fetch(`${API_BASE}/metrics`)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await res.json()
      metrics.value = {
        totalCalls: data.total_calls || 0,
        avgLatency: data.avg_latency_ms || 0,
        successRate: data.success_rate ?? 100.0,
        filesProcessed: data.files_processed || 0,
        tools: (data.tools || []).map((t: any) => ({
          name: t.name,
          calls: t.calls,
          latency: t.latency,
          successRate: t.success_rate
        })),
        uptimeSecs: data.uptime_secs || 0
      }
    } catch (e: any) {
      error.value = `Metrics fetch failed: ${e.message}`
    }
  }

  async function fetchStatus() {
    try {
      const res = await fetch(`${API_BASE}/status`)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await res.json()
      status.value = {
        memoryPercent: data.memory_percent || 0,
        pdfiumReady: data.pdfium_ready ?? false,
        pdfiumVersion: data.pdfium_version || 'unknown',
        queueLength: data.queue_length ?? 0,
        vlmEnabled: data.vlm_enabled ?? false,
        vlmModel: data.vlm_model || 'none',
        vlmThinking: data.vlm_thinking ?? false,
        vlmFunctionCall: data.vlm_function_call ?? false,
        vlmMultiModelRouting: data.vlm_multi_model_routing ?? false
      }
    } catch (e: any) {
      error.value = `Status fetch failed: ${e.message}`
    }
  }

  async function fetchHealth() {
    try {
      const res = await fetch(`${API_BASE}/health`)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await res.json()
      health.value = {
        status: data.status || 'unknown',
        mcpHealthy: data.mcp_healthy ?? false,
        clientConnections: data.client_connections ?? 0,
        uptimeSecs: data.uptime_secs || 0,
        version: data.version || 'unknown'
      }
    } catch (e: any) {
      error.value = `Health fetch failed: ${e.message}`
    }
  }

  async function fetchLogs() {
    try {
      const res = await fetch(`${API_BASE}/logs`)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data: LogEntry[] = await res.json()
      logs.value = data.slice(-50)
    } catch (e: any) {
      error.value = `Logs fetch failed: ${e.message}`
    }
  }

  async function clearLogs() {
    try {
      await fetch(`${API_BASE}/logs/clear`, { method: 'POST' })
      logs.value = []
    } catch (e: any) {
      error.value = `Clear logs failed: ${e.message}`
    }
  }

  async function recordToolCall(tool: string, latencyMs: number, success: boolean) {
    try {
      await fetch(`${API_BASE}/record`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tool, latency_ms: latencyMs, success })
      })
    } catch (e: any) {
      error.value = `Record tool call failed: ${e.message}`
    }
  }

  async function refreshAll() {
    loading.value = true
    try {
      await Promise.all([
        fetchHealth(),
        fetchMetrics(),
        fetchStatus(),
        fetchLogs()
      ])
    } finally {
      loading.value = false
    }
  }

  let pollInterval: number | null = null

  function startPolling(intervalMs: number = 5000) {
    stopPolling()
    pollInterval = window.setInterval(() => {
      fetchMetrics()
      fetchLogs()
    }, intervalMs)
  }

  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval)
      pollInterval = null
    }
  }

  onMounted(() => {
    refreshAll()
    startPolling()
  })

  onUnmounted(() => {
    stopPolling()
  })

  return {
    loading,
    error,
    metrics,
    status,
    health,
    logs,
    uptimeStr,
    fetchMetrics,
    fetchStatus,
    fetchHealth,
    fetchLogs,
    clearLogs,
    recordToolCall,
    refreshAll,
    startPolling,
    stopPolling
  }
}
