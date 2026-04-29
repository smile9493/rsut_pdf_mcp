import type {
  AdapterInfoResponse,
  CacheStatsResponse,
  KeywordSearchResponse,
  KeywordExtractResponse
} from '@/types/api'

export const pdfApi = {
  adapters: [
    { id: 'pdfium', name: 'PDFium', description: 'High compatibility PDF extraction engine' }
  ] as AdapterInfoResponse[],

  async health(): Promise<{ healthy: boolean }> {
    return { healthy: false }
  },

  async listAdapters(): Promise<AdapterInfoResponse[]> {
    return this.adapters
  },

  async extractTextFromPath(_filePath: string, _adapter: string | null = null): Promise<{ text: string; duration: number }> {
    throw new Error('MCP required: Use MCP server for extraction')
  },

  async extractTextFromFile(_file: File, _adapter: string | null = null): Promise<{ text: string; duration: number }> {
    throw new Error('MCP required: Use MCP server for extraction')
  },

  async extractStructuredFromPath(
    _filePath: string,
    _adapter: string | null = null,
    _options: { enableHighlight?: boolean } = {}
  ): Promise<{ extracted_text: string; page_count: number; pages: unknown[]; duration: number }> {
    throw new Error('MCP required: Use MCP server for extraction')
  },

  async extractStructuredFromFile(
    _file: File,
    _adapter: string | null = null,
    _options: { enableHighlight?: boolean } = {}
  ): Promise<{ extracted_text: string; page_count: number; pages: unknown[]; duration: number }> {
    throw new Error('MCP required: Use MCP server for extraction')
  },

  async getInfo(_filePath: string): Promise<Record<string, unknown>> {
    throw new Error('MCP required: Use MCP server for file info')
  },

  async getCacheStats(): Promise<CacheStatsResponse> {
    return { size: 0, max_size: 1000, hits: 0, misses: 0, hit_rate: 0 }
  },

  async searchKeywords(
    text: string,
    keywords: string[],
    options: { caseSensitive?: boolean; contextLength?: number } = {}
  ): Promise<KeywordSearchResponse> {
    const { caseSensitive = false, contextLength = 50 } = options
    const startTime = Date.now()

    const matches = keywords.flatMap((keyword) => {
      const regex = new RegExp(
        keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'),
        caseSensitive ? 'g' : 'gi'
      )
      return Array.from(text.matchAll(regex), (m) => ({
        keyword,
        page_number: 1,
        text: text.slice(Math.max(0, m.index! - contextLength), Math.min(text.length, m.index! + keyword.length + contextLength)),
        start_index: m.index!,
        end_index: m.index! + keyword.length,
        confidence: 1.0
      }))
    })

    return {
      keywords,
      matches,
      total_matches: matches.length,
      pages_with_matches: [...new Set(matches.map((m) => m.page_number))],
      duration: Date.now() - startTime
    }
  },

  async extractKeywords(text: string, topN: number = 10): Promise<KeywordExtractResponse> {
    const startTime = Date.now()
    const keywords = Object.entries(
      text
        .toLowerCase()
        .replace(/[^\w\s\u4e00-\u9fa5]/g, ' ')
        .split(/\s+/)
        .filter((w) => w.length >= 2)
        .reduce<Record<string, number>>((acc, w) => ({ ...acc, [w]: (acc[w] || 0) + 1 }), {})
    )
      .sort((a, b) => b[1] - a[1])
      .slice(0, topN) as Array<[string, number]>

    return { keywords, duration: Date.now() - startTime }
  }
}
