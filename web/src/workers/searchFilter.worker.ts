import type { WorkerMessage, WorkerResponse } from './protocol'

interface FilterRequest {
  data: Array<Record<string, unknown>>
  sortBy?: string
  sortDir?: 'asc' | 'desc'
  filterField?: string
  filterValue?: string
}

self.onmessage = (e: MessageEvent<WorkerMessage<FilterRequest>>) => {
  const { id, payload } = e.data
  let result = [...payload.data]

  // Filter
  if (payload.filterField && payload.filterValue) {
    const field = payload.filterField
    const value = payload.filterValue.toLowerCase()
    result = result.filter(item => {
      const fieldValue = String(item[field] ?? '').toLowerCase()
      return fieldValue.includes(value)
    })
  }

  // Sort
  if (payload.sortBy) {
    const dir = payload.sortDir === 'desc' ? -1 : 1
    const field = payload.sortBy
    result.sort((a, b) => {
      const aVal = a[field]
      const bVal = b[field]
      if (typeof aVal === 'number' && typeof bVal === 'number') {
        return (aVal - bVal) * dir
      }
      return String(aVal).localeCompare(String(bVal)) * dir
    })
  }

  const response: WorkerResponse<typeof result> = {
    type: 'response',
    id,
    payload: result
  }
  self.postMessage(response)
}
