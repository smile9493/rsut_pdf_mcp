// ─── Backpressure Awareness ─────────────────────────────────────────────────

export interface BackpressureConfig {
  normalInterval: number
  backoffInterval: number
  threshold: number
  maxBackoff: number
}

export class BackpressureAwareness {
  private hitRate = 0
  private _isBackpressured = false
  private readonly config: BackpressureConfig

  constructor(config?: Partial<BackpressureConfig>) {
    this.config = {
      normalInterval: config?.normalInterval ?? 1_000,
      backoffInterval: config?.backoffInterval ?? 5_000,
      threshold: config?.threshold ?? 0.8,
      maxBackoff: config?.maxBackoff ?? 30_000,
    }
  }

  get isBackpressured(): boolean {
    return this._isBackpressured
  }

  get currentHitRate(): number {
    return this.hitRate
  }

  /**
   * Update the hit rate from an SSE subscription event.
   */
  updateHitRate(rate: number): void {
    this.hitRate = Math.max(0, Math.min(1, rate))
    this._isBackpressured = this.hitRate >= this.config.threshold
  }

  /**
   * Compute the recommended request interval based on current backpressure.
   */
  getRequestInterval(): number {
    if (!this._isBackpressured) {
      return this.config.normalInterval
    }

    const overshoot = (this.hitRate - this.config.threshold) / (1 - this.config.threshold)
    const clampedOvershoot = Math.min(1, Math.max(0, overshoot))
    const interval =
      this.config.backoffInterval + clampedOvershoot * (this.config.maxBackoff - this.config.backoffInterval)

    return Math.min(interval, this.config.maxBackoff)
  }

  /**
   * Reset all state.
   */
  reset(): void {
    this.hitRate = 0
    this._isBackpressured = false
  }
}
