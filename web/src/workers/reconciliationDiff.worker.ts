import type { WorkerMessage, WorkerResponse } from './protocol'

interface DiffRequest {
  surrealIds: string[]
  lanceIds: string[]
}

interface DiffResponse {
  missingInLance: string[]
  missingInSurreal: string[]
  totalDiff: number
}

self.onmessage = (e: MessageEvent<WorkerMessage<DiffRequest>>) => {
  const { id, payload } = e.data

  const surrealSet = new Set(payload.surrealIds)
  const lanceSet = new Set(payload.lanceIds)

  const missingInLance = payload.surrealIds.filter(id => !lanceSet.has(id))
  const missingInSurreal = payload.lanceIds.filter(id => !surrealSet.has(id))

  const response: WorkerResponse<DiffResponse> = {
    type: 'response',
    id,
    payload: {
      missingInLance,
      missingInSurreal,
      totalDiff: missingInLance.length + missingInSurreal.length
    }
  }
  self.postMessage(response)
}
