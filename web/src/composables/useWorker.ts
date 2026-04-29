import { ref, onUnmounted, type Ref } from 'vue'

interface UseWorkerReturn<TRequest, TResponse> {
  data: Ref<TResponse | null>
  loading: Ref<boolean>
  error: Ref<string | null>
  execute: (payload: TRequest, transferables?: ArrayBuffer[]) => Promise<TResponse>
  terminate: () => void
}

export function useWorker<TRequest, TResponse>(
  workerFactory: () => Worker
): UseWorkerReturn<TRequest, TResponse> {
  const data = ref<TResponse | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  let worker: Worker | null = null
  let requestId = 0
  const pending = new Map<number, { resolve: (v: TResponse) => void; reject: (e: Error) => void }>()

  const getWorker = (): Worker => {
    if (!worker) {
      worker = workerFactory()
      worker.onmessage = (e) => {
        const { id, payload } = e.data
        const p = pending.get(id)
        if (p) {
          pending.delete(id)
          p.resolve(payload)
        }
      }
      worker.onerror = (e) => {
        error.value = e.message
        loading.value = false
      }
    }
    return worker
  }

  const execute = async (payload: TRequest, transferables?: ArrayBuffer[]): Promise<TResponse> => {
    loading.value = true
    error.value = null

    return new Promise<TResponse>((resolve, reject) => {
      const id = ++requestId
      pending.set(id, { resolve, reject })
      
      const w = getWorker()
      w.postMessage({ type: 'request', id, payload }, transferables ?? [])
    }).finally(() => {
      loading.value = false
    })
  }

  const terminate = () => {
    if (worker) {
      worker.terminate()
      worker = null
    }
    pending.forEach(p => p.reject(new Error('Worker terminated')))
    pending.clear()
  }

  onUnmounted(() => {
    terminate()
  })

  return { 
    data: data as Ref<TResponse | null>, 
    loading, 
    error, 
    execute, 
    terminate 
  }
}
