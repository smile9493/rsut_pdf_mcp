// ─── SSE Subscription Layer ─────────────────────────────────────────────────

export type SubscriptionHandler = (data: unknown) => void

export interface McpSubscriptionConfig {
  sseUrl: string
  maxReconnectAttempts?: number
  baseReconnectDelayMs?: number
  maxReconnectDelayMs?: number
}

export class McpSubscription {
  private eventSource: EventSource | null = null
  private handlers = new Map<string, Set<SubscriptionHandler>>()
  private reconnectAttempts = 0
  private disposed = false
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private readonly config: Required<McpSubscriptionConfig>

  constructor(config: McpSubscriptionConfig) {
    this.config = {
      sseUrl: config.sseUrl,
      maxReconnectAttempts: config.maxReconnectAttempts ?? 10,
      baseReconnectDelayMs: config.baseReconnectDelayMs ?? 1_000,
      maxReconnectDelayMs: config.maxReconnectDelayMs ?? 30_000,
    }
  }

  // ── Lifecycle ───────────────────────────────────────────────────────────

  connect(): void {
    if (this.disposed) return
    if (this.eventSource) return

    const es = new EventSource(this.config.sseUrl)

    es.onopen = () => {
      this.reconnectAttempts = 0
    }

    es.onmessage = (event: MessageEvent) => {
      this.dispatch(event.data)
    }

    es.onerror = () => {
      this.eventSource?.close()
      this.eventSource = null
      this.scheduleReconnect()
    }

    this.eventSource = es
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    if (this.eventSource) {
      this.eventSource.close()
      this.eventSource = null
    }
    this.reconnectAttempts = 0
  }

  dispose(): void {
    this.disposed = true
    this.disconnect()
    this.handlers.clear()
  }

  // ── Handler Registration ────────────────────────────────────────────────

  subscribe(metric: string, handler: SubscriptionHandler): () => void {
    if (!this.handlers.has(metric)) {
      this.handlers.set(metric, new Set())
    }
    this.handlers.get(metric)!.add(handler)

    return () => {
      this.handlers.get(metric)?.delete(handler)
      if (this.handlers.get(metric)?.size === 0) {
        this.handlers.delete(metric)
      }
    }
  }

  // ── Internal ────────────────────────────────────────────────────────────

  private dispatch(rawData: string): void {
    try {
      const parsed = JSON.parse(rawData) as { metric?: string; data?: unknown }
      const metric = parsed.metric
      if (metric && this.handlers.has(metric)) {
        this.handlers.get(metric)!.forEach((handler) => handler(parsed.data))
      }
    } catch {
      // Ignore malformed messages
    }
  }

  private scheduleReconnect(): void {
    if (this.disposed) return
    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) return

    const delay = Math.min(
      this.config.baseReconnectDelayMs * Math.pow(2, this.reconnectAttempts),
      this.config.maxReconnectDelayMs,
    )
    this.reconnectAttempts++

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null
      this.connect()
    }, delay)
  }
}
