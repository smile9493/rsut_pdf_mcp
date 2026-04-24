import axios from 'axios'

const api = axios.create({
  baseURL: '/api/v1/x2text',
  timeout: 60000, // 增加到60秒，处理大文件
  headers: {
    'Content-Type': 'application/json'
  }
})

// Request interceptor
api.interceptors.request.use(
  config => {
    // 添加请求时间戳用于性能监控
    config.metadata = { startTime: Date.now() }
    return config
  },
  error => {
    return Promise.reject(error)
  }
)

// Response interceptor
api.interceptors.response.use(
  response => {
    // 计算请求耗时
    const duration = Date.now() - response.config.metadata.startTime
    return {
      data: response.data,
      duration,
      status: response.status
    }
  },
  error => {
    const message = error.response?.data?.message || error.message || '请求失败'
    const status = error.response?.status || 500
    return Promise.reject({ message, status, error })
  }
)

export const pdfApi = {
  // Health check
  async health() {
    try {
      const response = await api.get('/health')
      return { healthy: true, ...response }
    } catch {
      return { healthy: false }
    }
  },

  // List adapters
  async listAdapters() {
    try {
      const response = await api.get('/adapters')
      return response.data?.adapters || []
    } catch {
      return []
    }
  },

  // Extract text from file path
  async extractTextFromPath(filePath, adapter = null) {
    const formData = new FormData()
    formData.append('file_path', filePath)
    if (adapter) {
      formData.append('adapter', adapter)
    }
    const response = await api.post('/extract', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return {
      text: response.data,
      duration: response.duration
    }
  },

  // Extract text from file object
  async extractTextFromFile(file, adapter = null) {
    const formData = new FormData()
    formData.append('file', file)
    if (adapter) {
      formData.append('adapter', adapter)
    }
    const response = await api.post('/extract', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return {
      text: response.data,
      duration: response.duration
    }
  },

  // Extract structured from file path
  async extractStructuredFromPath(filePath, adapter = null, options = {}) {
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
      ...response.data,
      duration: response.duration
    }
  },

  // Extract structured from file object
  async extractStructuredFromFile(file, adapter = null, options = {}) {
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
      ...response.data,
      duration: response.duration
    }
  },

  // Get PDF info
  async getInfo(filePath) {
    const formData = new FormData()
    formData.append('file_path', filePath)
    const response = await api.post('/info', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return response.data
  },

  // Cache stats
  async getCacheStats() {
    try {
      const response = await api.get('/cache/stats')
      return response.data
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

  // Search keywords (MCP tool simulation via structured extraction)
  async searchKeywords(filePath, keywords, options = {}) {
    // 先提取文本
    const result = await this.extractStructuredFromPath(filePath, null, options)
    const text = result.extracted_text || ''
    
    // 在前端进行关键词搜索
    const matches = []
    const caseSensitive = options.caseSensitive || false
    
    keywords.forEach(keyword => {
      const regex = new RegExp(
        keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'),
        caseSensitive ? 'g' : 'gi'
      )
      let match
      while ((match = regex.exec(text)) !== null) {
        const startIndex = match.index
        const endIndex = startIndex + keyword.length
        const contextLength = options.contextLength || 50
        
        // 找到对应的页码
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
      matches,
      totalMatches: matches.length,
      pagesWithMatches: [...new Set(matches.map(m => m.pageNumber))],
      duration: result.duration
    }
  },

  // Extract keywords (frequency analysis)
  async extractKeywords(filePath, topN = 10) {
    const result = await this.extractTextFromPath(filePath)
    const text = result.text
    
    // 简单的词频统计
    const words = text.toLowerCase()
      .replace(/[^\w\s\u4e00-\u9fa5]/g, ' ')
      .split(/\s+/)
      .filter(w => w.length >= 2)
    
    const freq = {}
    words.forEach(word => {
      freq[word] = (freq[word] || 0) + 1
    })
    
    const sorted = Object.entries(freq)
      .sort((a, b) => b[1] - a[1])
      .slice(0, topN)
      .map(([word, count]) => [word, count])
    
    return {
      keywords: sorted,
      duration: result.duration
    }
  }
}

export default api
