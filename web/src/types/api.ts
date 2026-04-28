export interface ApiResponse<T> {
  data: T
  duration: number
  status: number
}

export interface ApiError {
  message: string
  status: number
  error: unknown
}

export interface HealthResponse {
  healthy: boolean
  data?: Record<string, unknown>
  duration?: number
  status?: number
}

export interface AdapterInfoResponse {
  id: string
  name: string
  description: string
}

export interface CacheStatsResponse {
  size: number
  max_size: number
  hits: number
  misses: number
  hit_rate: number
}

export interface TextExtractionResponse {
  extracted_text: string
  extraction_metadata?: {
    whisper_hash: string
    line_metadata?: Record<string, unknown>
  }
}

export interface PageMetadata {
  page_number: number
  text: string
  bbox?: [number, number, number, number]
  lines: Array<{ bbox: number[]; text: string }>
}

export interface StructuredExtractionResponse {
  extracted_text: string
  page_count: number
  pages: PageMetadata[]
  extraction_metadata?: {
    whisper_hash: string
    line_metadata?: Record<string, unknown>
  }
  file_info: {
    file_path: string
    file_size: number
    file_size_mb: number
  }
}

export interface KeywordMatch {
  keyword: string
  page_number: number
  text: string
  bbox?: [number, number, number, number]
  start_index: number
  end_index: number
  confidence: number
}

export interface KeywordSearchResponse {
  keywords: string[]
  matches: KeywordMatch[]
  total_matches: number
  pages_with_matches: number[]
  duration?: number
}

export interface KeywordExtractResponse {
  keywords: Array<[string, number]>
  duration?: number
}
