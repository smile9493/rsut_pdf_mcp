// ─── JSON-RPC 2.0 Protocol Types ────────────────────────────────────────────

export interface JsonRpcRequest {
  jsonrpc: '2.0'
  id: number
  method: string
  params?: Record<string, unknown>
}

export interface JsonRpcResponse {
  jsonrpc: '2.0'
  id: number
  result?: unknown
  error?: { code: number; message: string; data?: unknown }
}

export interface JsonRpcNotification {
  jsonrpc: '2.0'
  method: string
  params?: Record<string, unknown>
}

export type JsonRpcMessage = JsonRpcResponse | JsonRpcNotification

// ─── Circuit Breaker ────────────────────────────────────────────────────────

export type CircuitState = 'closed' | 'open' | 'half-open'

export interface CircuitBreakerConfig {
  threshold: number
  windowMs: number
  cooldownMs: number
}

export class CircuitBreaker {
  private failures = 0
  private lastFailureTime = 0
  private state: CircuitState = 'closed'
  private readonly threshold: number
  private readonly windowMs: number
  private readonly cooldownMs: number

  constructor(config: CircuitBreakerConfig) {
    this.threshold = config.threshold
    this.windowMs = config.windowMs
    this.cooldownMs = config.cooldownMs
  }

  get currentState(): CircuitState {
    if (this.state === 'open') {
      if (Date.now() - this.lastFailureTime >= this.cooldownMs) {
        this.state = 'half-open'
      }
    }
    return this.state
  }

  recordSuccess(): void {
    this.failures = 0
    this.state = 'closed'
  }

  recordFailure(): void {
    const now = Date.now()
    if (now - this.lastFailureTime > this.windowMs) {
      this.failures = 1
    } else {
      this.failures++
    }
    this.lastFailureTime = now

    if (this.failures >= this.threshold) {
      this.state = 'open'
    }
  }

  reset(): void {
    this.failures = 0
    this.lastFailureTime = 0
    this.state = 'closed'
  }
}

// ─── Retry Policy ───────────────────────────────────────────────────────────

export interface RetryPolicyConfig {
  maxRetries: number
  baseDelayMs: number
  maxDelayMs: number
  factor: number
}

export class RetryPolicy {
  private readonly maxRetries: number
  private readonly baseDelayMs: number
  private readonly maxDelayMs: number
  private readonly factor: number

  constructor(config: RetryPolicyConfig) {
    this.maxRetries = config.maxRetries
    this.baseDelayMs = config.baseDelayMs
    this.maxDelayMs = config.maxDelayMs
    this.factor = config.factor
  }

  getDelay(attempt: number): number {
    const delay = this.baseDelayMs * Math.pow(this.factor, attempt)
    return Math.min(delay, this.maxDelayMs)
  }

  shouldRetry(attempt: number): boolean {
    return attempt < this.maxRetries
  }
}

// ─── Provider Config & Types ────────────────────────────────────────────────

export interface McpProviderConfig {
  sseUrl: string
  uplinkUrl: string
  sessionId?: string
  circuitBreaker?: Partial<CircuitBreakerConfig>
  retryPolicy?: Partial<RetryPolicyConfig>
}

interface PendingRequest {
  resolve: (value: unknown) => void
  reject: (reason: Error) => void
  timer: ReturnType<typeof setTimeout>
}

export type NotificationHandler = (notification: JsonRpcNotification) => void

// ─── McpProvider ────────────────────────────────────────────────────────────

export class McpProvider {
  private readonly config: McpProviderConfig
  private eventSource: EventSource | null = null
  private pendingRequests = new Map<number, PendingRequest>()
  private notificationHandlers = new Map<string, Set<NotificationHandler>>()
  private nextId = 1
  private _connected = false
  private _session: string | null = null
  private _circuitState: CircuitState = 'closed'
  private readonly circuitBreaker: CircuitBreaker
  private readonly retryPolicy: RetryPolicy

  constructor(config: McpProviderConfig) {
    this.config = config
    this.circuitBreaker = new CircuitBreaker({
      threshold: config.circuitBreaker?.threshold ?? 5,
      windowMs: config.circuitBreaker?.windowMs ?? 60_000,
      cooldownMs: config.circuitBreaker?.cooldownMs ?? 30_000,
    })
    this.retryPolicy = new RetryPolicy({
      maxRetries: config.retryPolicy?.maxRetries ?? 3,
      baseDelayMs: config.retryPolicy?.baseDelayMs ?? 1_000,
      maxDelayMs: config.retryPolicy?.maxDelayMs ?? 30_000,
      factor: config.retryPolicy?.factor ?? 2,
    })
    this._session = config.sessionId ?? null
  }

  // ── Properties ──────────────────────────────────────────────────────────

  get connected(): boolean {
    return this._connected
  }

  get session(): string | null {
    return this._session
  }

  get circuitState(): CircuitState {
    return this.circuitBreaker.currentState
  }

  // ── Lifecycle ───────────────────────────────────────────────────────────

  async connect(): Promise<void> {
    if (this._connected) return

    return new Promise((resolve, reject) => {
      const es = new EventSource(this.config.sseUrl)

      es.onopen = () => {
        this._connected = true
        this.circuitBreaker.recordSuccess()
        resolve()
      }

      es.onmessage = (event: MessageEvent) => {
        this.handleSseMessage(event.data)
      }

      es.onerror = () => {
        this._connected = false
        this.circuitBreaker.recordFailure()
        reject(new Error('SSE connection failed'))
      }
    })
  }

  disconnect(): void {
    if (this.eventSource) {
      this.eventSource.close()
      this.eventSource = null
    }
    this._connected = false
    this.rejectAllPending('Provider disconnected')
  }

  destroy(): void {
    this.disconnect()
    this.notificationHandlers.clear()
    this.circuitBreaker.reset()
  }

  // ── Request / Notify ────────────────────────────────────────────────────

  async request<T = unknown>(method: string, params?: Record<string, unknown>): Promise<T> {
    if (this.circuitBreaker.currentState === 'open') {
      throw new Error('Circuit breaker is open')
    }

    const id = this.nextId++
    const request: JsonRpcRequest = {
      jsonrpc: '2.0',
      id,
      method,
      params,
    }

    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pendingRequests.delete(id)
        reject(new Error(`Request ${id} timed out`))
      }, 30_000)

      this.pendingRequests.set(id, {
        resolve: resolve as (value: unknown) => void,
        reject,
        timer,
      })

      this.postRequest(request).catch((err) => {
        clearTimeout(timer)
        this.pendingRequests.delete(id)
        reject(err)
      })
    })
  }

  notify(method: string, params?: Record<string, unknown>): void {
    const notification: JsonRpcNotification = {
      jsonrpc: '2.0',
      method,
      params,
    }
    this.postRequest(notification).catch(() => {
      // Notifications are fire-and-forget
    })
  }

  onNotification(method: string, handler: NotificationHandler): () => void {
    if (!this.notificationHandlers.has(method)) {
      this.notificationHandlers.set(method, new Set())
    }
    this.notificationHandlers.get(method)!.add(handler)

    return () => {
      this.notificationHandlers.get(method)?.delete(handler)
    }
  }

  // ── Internal ────────────────────────────────────────────────────────────

  private async postRequest(message: JsonRpcRequest | JsonRpcNotification): Promise<void> {
    const response = await fetch(this.config.uplinkUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(this._session ? { 'X-Mcp-Session': this._session } : {}),
      },
      body: JSON.stringify(message),
    })

    if (!response.ok) {
      throw new Error(`Uplink request failed: ${response.status}`)
    }

    // If it's a request (has id), handle the response
    if ('id' in message) {
      const json = (await response.json()) as JsonRpcResponse
      this.handleResponse(json)
    }
  }

  private handleResponse(response: JsonRpcResponse): void {
    const pending = this.pendingRequests.get(response.id)
    if (!pending) return

    clearTimeout(pending.timer)
    this.pendingRequests.delete(response.id)

    if (response.error) {
      this.circuitBreaker.recordFailure()
      pending.reject(new Error(response.error.message))
    } else {
      this.circuitBreaker.recordSuccess()
      pending.resolve(response.result)
    }
  }

  private handleSseMessage(data: string): void {
    try {
      const message = JSON.parse(data) as JsonRpcMessage

      // Check for session header in SSE handshake
      if ('method' in message && message.method === 'session.init') {
        const params = message.params as Record<string, unknown> | undefined
        if (params?.sessionId) {
          this._session = params.sessionId as string
        }
        return
      }

      // Handle response (has id)
      if ('id' in message) {
        this.handleResponse(message as JsonRpcResponse)
        return
      }

      // Handle notification (no id)
      const notification = message as JsonRpcNotification
      const handlers = this.notificationHandlers.get(notification.method)
      if (handlers) {
        handlers.forEach((handler) => handler(notification))
      }
    } catch {
      // Ignore malformed messages
    }
  }

  private rejectAllPending(reason: string): void {
    this.pendingRequests.forEach((pending) => {
      clearTimeout(pending.timer)
      pending.reject(new Error(reason))
    })
    this.pendingRequests.clear()
  }
}
