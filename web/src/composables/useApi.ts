import axios from 'axios'
import type { AxiosRequestConfig, InternalAxiosRequestConfig, AxiosResponse } from 'axios'
import type {
  ApiResponse,
  ApiError,
  HealthResponse,
  AdapterInfoResponse,
  CacheStatsResponse,
  TextExtractionResponse,
  StructuredExtractionResponse,
  KeywordSearchResponse,
  KeywordExtractResponse
} from '@/types/api'

interface RequestMetadata {
  startTime: number
}

interface ExtractOptions {
  enableHighlight?: boolean
}

interface KeywordSearchOptions {
  caseSensitive?: boolean
  contextLength?: number
  enableHighlight?: boolean
}

const api = axios.create({
  baseURL: '/api/v1/x2text',
  timeout: 60000,
  headers: {
    'Content-Type': 'application/json'
  }
})

api.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    ;(config as InternalAxiosRequestConfig & { metadata: RequestMetadata }).metadata = {
      startTime: Date.now()
    }
    return config
  },
  (error: unknown) => {
    return Promise.reject(error)
  }
)

api.interceptors.response.use(
  (response: AxiosResponse) => {
    const metadata = (response.config as InternalAxiosRequestConfig & { metadata: RequestMetadata }).metadata
    const duration = Date.now() - metadata.startTime
    return {
      data: response.data,
      duration,
      status: response.status
    } as ApiResponse<typeof response.data>
  },
  (error: unknown) => {
    const axiosError = error as {
      response?: { data?: { message?: string }; status?: number }
      message?: string
    }
    const message: string = axiosError.response?.data?.message || axiosError.message || '请求失败'
    const status: number = axiosError.response?.status || 500
    return Promise.reject({ message, status, error } as ApiError)
  }
)

export const pdfApi = {
  async health(): Promise<HealthResponse> {
    try {
      const response = await api.get('/health')
      return { healthy: true, ...response } as HealthResponse
    } catch {
      return { healthy: false }
    }
  },

  async listAdapters(): Promise<AdapterInfoResponse[]> {
    try {
      const response = await api.get('/adapters')
      return (response.data as { adapters?: AdapterInfoResponse[] })?.adapters || []
    } catch {
      return []
    }
  },

  async extractTextFromPath(filePath: string, adapter: string | null = null): Promise<{ text: string; duration: number }> {
    const formData = new FormData()
    formData.append('file_path', filePath)
    if (adapter) {
      formData.append('adapter', adapter)
    }
    const response = await api.post('/extract', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return {
      text: response.data as string,
      duration: (response as ApiResponse<string>).duration
    }
  },

  async extractTextFromFile(file: File, adapter: string | null = null): Promise<{ text: string; duration: number }> {
    const formData = new FormData()
    formData.append('file', file)
    if (adapter) {
      formData.append('adapter', adapter)
    }
    const response = await api.post('/extract', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return {
      text: response.data as string,
      duration: (response as ApiResponse<string>).duration
    }
  },

  async extractStructuredFromPath(
    filePath: string,
    adapter: string | null = null,
    options: ExtractOptions = {}
  ): Promise<StructuredExtractionResponse & { duration: number }> {
    const formData = new FormData()
    formData.append('file_path', filePath)
    if (adapter) {
      formData.append('adapter', adapter)
    }
    if (options.enableHighlight) {
      formData.append('enable_highlight', 'true')
    }
    const response = await api.post('/extract-json', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return {
      ...(response.data as StructuredExtractionResponse),
      duration: (response as ApiResponse<StructuredExtractionResponse>).duration
    }
  },

  async extractStructuredFromFile(
    file: File,
    adapter: string | null = null,
    options: ExtractOptions = {}
  ): Promise<StructuredExtractionResponse & { duration: number }> {
    const formData = new FormData()
    formData.append('file', file)
    if (adapter) {
      formData.append('adapter', adapter)
    }
    if (options.enableHighlight) {
      formData.append('enable_highlight', 'true')
    }
    const response = await api.post('/extract-json', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return {
      ...(response.data as StructuredExtractionResponse),
      duration: (response as ApiResponse<StructuredExtractionResponse>).duration
    }
  },

  async getInfo(filePath: string): Promise<Record<string, unknown>> {
    const formData = new FormData()
    formData.append('file_path', filePath)
    const response = await api.post('/info', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return response.data as Record<string, unknown>
  },

  async getCacheStats(): Promise<CacheStatsResponse> {
    try {
      const response = await api.get('/cache/stats')
      return response.data as CacheStatsResponse
    } catch {
      return {
        size: 0,
        max_size: 1000,
        hits: 0,
        misses: 0,
        hit_rate: 0
      }
    }
  },

  async searchKeywords(
    filePath: string,
    keywords: string[],
    options: KeywordSearchOptions = {}
  ): Promise<KeywordSearchResponse> {
    const result = await this.extractStructuredFromPath(filePath, null, options)
    const text = result.extracted_text || ''

    const matches: Array<{
      keyword: string
      pageNumber: number
      text: string
      startIndex: number
      endIndex: number
      confidence: number
    }> = []
    const caseSensitive = options.caseSensitive || false

    keywords.forEach((keyword: string) => {
      const regex = new RegExp(
        keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'),
        caseSensitive ? 'g' : 'gi'
      )
      let match: RegExpExecArray | null
      while ((match = regex.exec(text)) !== null) {
        const startIndex = match.index
        const endIndex = startIndex + keyword.length
        const contextLength = options.contextLength || 50

        let pageNumber = 1
        if (result.pages) {
          let charCount = 0
          for (const page of result.pages) {
            charCount += page.text.length
            if (startIndex < charCount) {
              pageNumber = page.page_number
              break
            }
          }
        }

        matches.push({
          keyword,
          pageNumber,
          text: text.substring(
            Math.max(0, startIndex - contextLength),
            Math.min(text.length, endIndex + contextLength)
          ),
          startIndex,
          endIndex,
          confidence: 1.0
        })
      }
    })

    return {
      keywords,
      matches: matches.map((m) => ({
        keyword: m.keyword,
        page_number: m.pageNumber,
        text: m.text,
        start_index: m.startIndex,
        end_index: m.endIndex,
        confidence: m.confidence
      })),
      total_matches: matches.length,
      pages_with_matches: [...new Set(matches.map((m) => m.pageNumber))],
      duration: result.duration
    }
  },

  async extractKeywords(filePath: string, topN: number = 10): Promise<KeywordExtractResponse> {
    const result = await this.extractTextFromPath(filePath)
    const text = result.text

    const words = text
      .toLowerCase()
      .replace(/[^\w\s\u4e00-\u9fa5]/g, ' ')
      .split(/\s+/)
      .filter((w: string) => w.length >= 2)

    const freq: Record<string, number> = {}
    words.forEach((word: string) => {
      freq[word] = (freq[word] || 0) + 1
    })

    const sorted: Array<[string, number]> = Object.entries(freq)
      .sort((a: [string, number], b: [string, number]) => b[1] - a[1])
      .slice(0, topN)
      .map(([word, count]: [string, number]) => [word, count])

    return {
      keywords: sorted,
      duration: result.duration
    }
  }
}

export default api
