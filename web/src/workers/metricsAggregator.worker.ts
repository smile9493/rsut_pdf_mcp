import type { WorkerMessage, WorkerResponse } from './protocol'

interface MetricData {
  timestamp: number
  value: number
}

interface AggregateRequest {
  data: MetricData[]
  windowSize: number
}

interface AggregateResponse {
  avg: number
  min: number
  max: number
  p99: number
  count: number
}

self.onmessage = (e: MessageEvent<WorkerMessage<AggregateRequest>>) => {
  const { id, payload } = e.data
  const { data, windowSize } = payload

  // Slice to window
  const windowData = data.slice(-windowSize)
  
  if (windowData.length === 0) {
    const response: WorkerResponse<AggregateResponse> = {
      type: 'response',
      id,
      payload: { avg: 0, min: 0, max: 0, p99: 0, count: 0 }
    }
    self.postMessage(response)
    return
  }

  const values = windowData.map(d => d.value).sort((a, b) => a - b)
  const avg = values.reduce((a, b) => a + b, 0) / values.length
  const min = values[0]
  const max = values[values.length - 1]
  const p99Index = Math.floor(values.length * 0.99)
  const p99 = values[Math.min(p99Index, values.length - 1)]

  const response: WorkerResponse<AggregateResponse> = {
    type: 'response',
    id,
    payload: { avg, min, max, p99, count: values.length }
  }
  self.postMessage(response)
}
