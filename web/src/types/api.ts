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

export interface TextExtractionMetadata {
  whisper_hash: string
  line_metadata?: Record<string, unknown>
}

export interface TextExtractionResponse {
  extracted_text: string
  extraction_metadata?: TextExtractionMetadata
}

export interface LineInfo {
  bbox: number[]
  text: string
}

export interface PageMetadata {
  page_number: number
  text: string
  bbox?: [number, number, number, number]
  lines: LineInfo[]
}

export interface FileInfo {
  file_path: string
  file_size: number
  file_size_mb: number
}

export interface StructuredExtractionResponse {
  extracted_text: string
  page_count: number
  pages: PageMetadata[]
  extraction_metadata?: TextExtractionMetadata
  file_info: FileInfo
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

export interface ExtractOptions {
  enable_highlight?: boolean
  adapter?: string
}

export type ExecutionStatus = 'success' | 'failed' | 'timeout' | 'cancelled'

export interface ExecutionMetric {
  tool_name: string
  execution_id: string
  start_time: string
  end_time: string
  status: ExecutionStatus
  error_message?: string
}

export interface ToolContext {
  execution_id: string
  org_id?: string
  workflow_id?: string
  user_id?: string
  request_id?: string
  metadata?: Record<string, string>
}

export interface ToolExecutionOptions {
  enable_streaming?: boolean
  timeout?: number
  enable_cache?: boolean
  enable_metrics?: boolean
  additional?: Record<string, unknown>
}

export type ParameterType = 'string' | 'number' | 'boolean' | 'array' | 'object'

export interface Parameter {
  name: string
  type: ParameterType
  description: string
  required: boolean
  default?: unknown
  enum_values?: string[]
}

export type InputType = 'file' | 'database' | 'index' | 'text'

export type OutputType = 'file' | 'database' | 'index' | 'text' | 'json'

export interface ResourceRequirement {
  input: boolean
  output: boolean
}

export interface ToolRequirements {
  files: ResourceRequirement
  databases: ResourceRequirement
}

export type ExtractionStatus = 'pending' | 'processing' | 'completed' | 'failed'

export type LogLevel = 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'FATAL'

export type StorageType = 'local' | 's3' | 'gcs' | 'azure_blob' | 'minio' | 'http'

export interface FileMetadata {
  path: string
  size: number
  modified: string
  content_type?: string
}

export type Environment = 'development' | 'staging' | 'production'

export interface LocalStorageConfig {
  base_dir: string
}

export interface S3StorageConfig {
  bucket: string
  region: string
  prefix?: string
  access_key?: string
  secret_key?: string
  endpoint?: string
}

export interface GCSStorageConfig {
  bucket: string
  credentials_path: string
}

export interface AzureStorageConfig {
  account: string
  key: string
  container: string
}

export type PluginType = 'local' | 'remote' | 'wasm'

export interface RetryPolicy {
  max_retries: number
  initial_delay_ms: number
  max_delay_ms: number
  multiplier: number
}

export interface RateLimitConfig {
  requests_per_second: number
  burst_size: number
}

export interface PluginConfig {
  plugin_id: string
  plugin_type: PluginType
  enabled?: boolean
  priority?: number
  timeout_ms?: number
  retry_policy?: RetryPolicy
  rate_limit?: RateLimitConfig
}

export interface ToolExecutionResult {
  workflow_id: string
  elapsed_time: number
  output: unknown
  metadata?: ExecutionMetadata
}

export interface ExecutionMetadata {
  file_name: string
  file_size: number
  processing_time: number
  cache_hit: boolean
  adapter_used: string
}
