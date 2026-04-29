import { ref, type Ref } from 'vue'

export interface AsyncState<T> {
  data: Ref<T | null>
  loading: Ref<boolean>
  error: Ref<string | null>
}

export function useAsyncAction<T>(
  initialData: T | null = null
): AsyncState<T> & {
  execute: <R = T>(
    action: () => Promise<R>,
    onSuccess?: (data: R) => void,
    onError?: (err: Error) => void
  ) => Promise<R | null>
  reset: () => void
} {
  const data = ref<T | null>(initialData) as Ref<T | null>
  const loading = ref(false)
  const error = ref<string | null>(null)

  const execute = async <R = T>(
    action: () => Promise<R>,
    onSuccess?: (data: R) => void,
    onError?: (err: Error) => void
  ): Promise<R | null> => {
    loading.value = true
    error.value = null
    try {
      const result = await action()
      onSuccess?.(result)
      return result
    } catch (err) {
      const message = (err as Error).message
      error.value = message
      onError?.(err as Error)
      return null
    } finally {
      loading.value = false
    }
  }

  const reset = (): void => {
    data.value = initialData
    loading.value = false
    error.value = null
  }

  return { data, loading, error, execute, reset }
}
