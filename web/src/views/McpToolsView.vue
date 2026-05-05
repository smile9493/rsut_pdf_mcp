<template>
  <div class="h-full overflow-auto bg-slate-50 dark:bg-slate-900">
    <div class="sticky top-0 z-10 bg-slate-50/90 dark:bg-slate-900/90 backdrop-blur-sm px-8 py-4 border-b border-slate-200/50 dark:border-slate-700/50">
      <div class="flex items-center justify-between">
        <h1 class="text-sm font-semibold text-slate-900 dark:text-slate-100">MCP 工具调用</h1>
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[11px] font-medium" :class="mcpReady ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400' : 'bg-red-100 text-red-700 dark:bg-red-500/10 dark:text-red-400'">
            <span class="w-1.5 h-1.5 rounded-full" :class="mcpReady ? 'bg-emerald-500' : 'bg-red-500'" />
            {{ mcpReady ? '已连接' : '未连接' }}
          </div>
          <span class="text-xs text-slate-400">{{ tools.length }} 个工具</span>
        </div>
      </div>
    </div>

    <div class="flex h-[calc(100vh-53px)]">
      <div class="w-80 flex-shrink-0 border-r border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800/50 overflow-y-auto">
        <div class="p-4 space-y-4">
          <div>
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">基础工具</p>
            <div class="space-y-1">
              <button
                v-for="tool in basicTools"
                :key="tool.name"
                class="w-full flex items-center gap-2.5 px-3 py-2 rounded-md text-left text-sm transition-colors"
                :class="selectedTool?.name === tool.name ? 'bg-blue-50 text-blue-700 dark:bg-blue-500/10 dark:text-blue-400' : 'text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700/50'"
                @click="selectedTool = tool"
              >
                <component :is="tool.icon" class="w-4 h-4 flex-shrink-0" />
                <span class="font-medium">{{ tool.label }}</span>
              </button>
            </div>
          </div>
          <div>
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
              Wiki 工具
            </p>
            <div class="space-y-1">
              <button
                v-for="tool in wikiTools"
                :key="tool.name"
                class="w-full flex items-center gap-2.5 px-3 py-2 rounded-md text-left text-sm transition-colors"
                :class="selectedTool?.name === tool.name ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400' : 'text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700/50'"
                @click="selectedTool = tool"
              >
                <component :is="tool.icon" class="w-4 h-4 flex-shrink-0" />
                <span class="font-medium">{{ tool.label }}</span>
              </button>
            </div>
          </div>
          <div>
            <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
              知识引擎
              <span class="ml-1 px-1 py-px rounded text-[9px] font-bold bg-purple-100 text-purple-700 dark:bg-purple-500/20 dark:text-purple-400">AI</span>
            </p>
            <div class="space-y-1">
              <button
                v-for="tool in knowledgeTools"
                :key="tool.name"
                class="w-full flex items-center gap-2.5 px-3 py-2 rounded-md text-left text-sm transition-colors"
                :class="selectedTool?.name === tool.name ? 'bg-purple-50 text-purple-700 dark:bg-purple-500/10 dark:text-purple-400' : 'text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700/50'"
                @click="selectedTool = tool"
              >
                <component :is="tool.icon" class="w-4 h-4 flex-shrink-0" />
                <span class="font-medium">{{ tool.label }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-w-0">
        <template v-if="selectedTool">
          <div class="flex-shrink-0 border-b border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800/30 px-6 py-4">
            <div class="flex items-center gap-2 mb-2">
              <component :is="selectedTool.icon" class="w-4 h-4" :class="{
                'text-blue-600 dark:text-blue-400': selectedTool.category === 'basic',
                'text-emerald-600 dark:text-emerald-400': selectedTool.category === 'wiki',
                'text-purple-600 dark:text-purple-400': selectedTool.category === 'knowledge'
              }" />
              <h2 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{{ selectedTool.label }}</h2>
              <span class="text-xs text-slate-400 font-mono">{{ selectedTool.name }}</span>
            </div>
            <p class="text-xs text-slate-500 dark:text-slate-400 leading-relaxed max-w-2xl">{{ toolInstructions[selectedTool.name] }}</p>
          </div>

          <div class="flex-shrink-0 px-6 py-4 border-b border-slate-100 dark:border-slate-700/50 space-y-3">
            <div v-if="needsFilePath">
              <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">文件路径 <span class="text-red-500">*</span></label>
              <div class="flex gap-2">
                <input v-model="filePath" type="text" placeholder="/path/to/document.pdf" class="flex-1 px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
                <button class="px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700" @click="triggerFileUpload">浏览</button>
                <input ref="fileInput" type="file" accept=".pdf" class="hidden" @change="handleFileSelect">
              </div>
            </div>

            <div v-if="needsKnowledgeBase">
              <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">知识库路径 <span class="text-red-500">*</span></label>
              <input v-model="knowledgeBase" type="text" placeholder="/path/to/knowledge_base" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
            </div>

            <div v-if="selectedTool.name === 'search_keywords'" class="flex gap-3">
              <div class="flex-1">
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">关键词 <span class="text-red-500">*</span></label>
                <input v-model="keywords" type="text" placeholder="多个关键词用逗号分隔" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
              </div>
              <label class="flex items-end gap-1.5 pb-1.5 cursor-pointer">
                <input v-model="caseSensitive" type="checkbox" class="rounded text-blue-600 w-3.5 h-3.5">
                <span class="text-xs text-slate-500">区分大小写</span>
              </label>
            </div>

            <div v-if="selectedTool.name === 'extract_structured'" class="flex items-center gap-1.5">
              <input v-model="enableHighlight" type="checkbox" class="rounded text-blue-600 w-3.5 h-3.5">
              <span class="text-xs text-slate-500">启用高亮标注</span>
            </div>

            <div v-if="selectedTool.name === 'extrude_to_server_wiki'" class="flex gap-3">
              <div class="flex-1">
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">Wiki 存储路径</label>
                <input v-model="wikiBasePath" type="text" placeholder="./wiki" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500">
              </div>
            </div>

            <div v-if="selectedTool.name === 'extrude_to_agent_payload'" class="p-2.5 rounded-md bg-emerald-50 dark:bg-emerald-500/10 border border-emerald-200 dark:border-emerald-500/20">
              <p class="text-xs text-emerald-700 dark:text-emerald-400">返回带 YAML 元数据的 Markdown 报文，供 Agent 本地构建 Wiki，无需服务器端存储。</p>
            </div>

            <div v-if="selectedTool.name === 'compile_to_wiki'" class="space-y-3">
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">领域分类</label>
                <input v-model="domain" type="text" placeholder="IT, Math, 未分类..." class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
              </div>
            </div>

            <div v-if="selectedTool.name === 'search_knowledge'" class="space-y-3">
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">搜索查询 <span class="text-red-500">*</span></label>
                <input v-model="searchQuery" type="text" placeholder="输入关键词或短语..." class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
              </div>
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">结果数量</label>
                <input v-model.number="limit" type="number" min="1" max="100" class="w-24 px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
              </div>
            </div>

            <div v-if="['get_entry_context', 'suggest_links', 'export_concept_map', 'recompile_entry'].includes(selectedTool.name)" class="space-y-3">
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">条目路径 <span class="text-red-500">*</span></label>
                <input v-model="entryPath" type="text" placeholder="it/concept.md" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm font-mono text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
              </div>
              <div v-if="selectedTool.name === 'get_entry_context'" class="flex gap-3">
                <div>
                  <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">跳数</label>
                  <input v-model.number="hops" type="number" min="1" max="5" class="w-20 px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
                </div>
              </div>
              <div v-if="selectedTool.name === 'suggest_links'" class="flex gap-3">
                <div>
                  <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">Top K</label>
                  <input v-model.number="topK" type="number" min="1" max="50" class="w-20 px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
                </div>
              </div>
              <div v-if="selectedTool.name === 'export_concept_map'" class="flex gap-3">
                <div>
                  <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">深度</label>
                  <input v-model.number="depth" type="number" min="1" max="5" class="w-20 px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
                </div>
              </div>
            </div>

            <div v-if="selectedTool.name === 'micro_compile'" class="space-y-3">
              <div>
                <label class="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1.5">页面范围</label>
                <input v-model="pageRange" type="text" placeholder="1-5 或 3,7,12" class="w-full px-3 py-1.5 rounded-md border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
              </div>
            </div>

            <div v-if="['incremental_compile', 'rebuild_index', 'find_orphans', 'check_quality', 'aggregate_entries', 'hypothesis_test'].includes(selectedTool.name)" class="p-2.5 rounded-md bg-purple-50 dark:bg-purple-500/10 border border-purple-200 dark:border-purple-500/20">
              <p class="text-xs text-purple-700 dark:text-purple-400">{{ toolInstructions[selectedTool.name] }}</p>
            </div>

            <div class="flex items-center gap-3 pt-1">
              <button
                :disabled="loading || !canExecute"
                class="px-4 py-1.5 rounded-md text-sm font-medium text-white transition-colors disabled:cursor-not-allowed disabled:opacity-40"
                :class="{
                  'bg-blue-600 hover:bg-blue-700': selectedTool?.category === 'basic',
                  'bg-emerald-600 hover:bg-emerald-700': selectedTool?.category === 'wiki',
                  'bg-purple-600 hover:bg-purple-700': selectedTool?.category === 'knowledge'
                }"
                @click="executeTool"
              >
                <template v-if="loading">
                  <svg class="animate-spin h-3.5 w-3.5 inline mr-1.5" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" /><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" /></svg>
                  执行中
                </template>
                <template v-else>执行</template>
              </button>
              <div v-if="executionLog.length > 0" class="flex items-center gap-3 text-xs text-slate-400">
                <span>{{ executionLog.length }} 次调用</span>
                <span>成功率 {{ successRate.toFixed(0) }}%</span>
                <span>均耗 {{ avgLatency }}ms</span>
              </div>
            </div>
          </div>

          <div class="flex-1 overflow-auto px-6 py-4">
            <div v-if="error" class="mb-4 p-3 rounded-md bg-red-50 dark:bg-red-500/10 border border-red-200 dark:border-red-500/20 flex items-start gap-2">
              <span class="text-xs text-red-700 dark:text-red-300">{{ error }}</span>
              <button class="ml-auto text-red-400 hover:text-red-600 text-xs" @click="error = null">×</button>
            </div>

            <template v-if="result">
              <div class="rounded-md bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 overflow-hidden">
                <div class="flex items-center justify-between px-4 py-2 border-b border-slate-100 dark:border-slate-700/50 bg-slate-50 dark:bg-slate-700/30">
                  <span class="text-[11px] font-semibold text-slate-500 uppercase tracking-wider">执行结果</span>
                  <span class="text-[11px] text-slate-400">{{ result.duration }}ms</span>
                </div>
                <div class="p-4">
                  <template v-if="selectedTool?.name === 'search_keywords' && result.data?.matches">
                    <div class="flex items-center gap-4 text-xs text-slate-500 mb-3">
                      <span>总匹配 <strong class="text-slate-900 dark:text-slate-100">{{ result.data.total_matches }}</strong></span>
                      <span>匹配页数 <strong class="text-slate-900 dark:text-slate-100">{{ result.data.pages_with_matches?.length || 0 }}</strong></span>
                    </div>
                    <div class="space-y-1.5 max-h-[50vh] overflow-auto">
                      <div v-for="(m, idx) in result.data.matches.slice(0, 50)" :key="idx" class="p-2.5 rounded-md bg-slate-50 dark:bg-slate-700/50">
                        <div class="flex items-center gap-2 mb-1">
                          <span class="px-1.5 py-0.5 rounded text-[11px] font-medium bg-blue-100 text-blue-700 dark:bg-blue-500/10 dark:text-blue-400">{{ m.keyword }}</span>
                          <span class="text-[11px] text-slate-400">第 {{ m.page_number }} 页</span>
                        </div>
                        <p class="text-xs text-slate-700 dark:text-slate-300 font-mono leading-relaxed">{{ m.text }}</p>
                      </div>
                    </div>
                  </template>
                  <pre v-else class="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap font-mono bg-slate-50 dark:bg-slate-700/50 rounded-md p-3 max-h-[50vh] overflow-auto leading-relaxed">{{ formatResult(result.data) }}</pre>
                </div>
              </div>
            </template>

            <div v-if="executionLog.length > 0 && !result" class="space-y-1">
              <div class="flex items-center justify-between mb-2">
                <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">调用日志</span>
                <button class="text-[11px] text-slate-400 hover:text-red-500" @click="clearLog">清空</button>
              </div>
              <div v-for="(log, idx) in recentLogs" :key="idx" class="flex items-center justify-between px-3 py-1.5 rounded-md text-xs" :class="log.success ? 'bg-emerald-50 dark:bg-emerald-500/5' : 'bg-red-50 dark:bg-red-500/5'">
                <div class="flex items-center gap-2">
                  <span :class="log.success ? 'text-emerald-600' : 'text-red-600'">{{ log.success ? '✓' : '✗' }}</span>
                  <span class="text-slate-700 dark:text-slate-300">{{ toolNameMap[log.tool] || log.tool }}</span>
                </div>
                <div class="flex items-center gap-2 text-slate-400">
                  <span>{{ log.duration }}ms</span>
                  <span>{{ formatTime(log.timestamp) }}</span>
                </div>
              </div>
            </div>

            <div v-if="!result && executionLog.length === 0" class="flex items-center justify-center h-full text-slate-400 dark:text-slate-600">
              <span class="text-sm">选择工具，填写参数后执行</span>
            </div>
          </div>
        </template>

        <div v-else class="flex-1 flex items-center justify-center text-slate-400 dark:text-slate-600">
          <span class="text-sm">从左侧选择一个工具开始</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  DocumentTextIcon, CubeIcon, DocumentIcon, MagnifyingGlassIcon,
  ServerIcon, CloudIcon, BookOpenIcon, SparklesIcon,
  ArrowPathIcon, BeakerIcon, LightBulbIcon, ChartBarIcon,
  CloudArrowUpIcon, QueueListIcon, PuzzlePieceIcon, CircleStackIcon,
  AcademicCapIcon, DocumentMagnifyingGlassIcon
} from '@heroicons/vue/24/outline'

interface Tool { name: string; label: string; description: string; icon: any; category: string }
interface LogEntry { tool: string; success: boolean; duration: number; timestamp: number; error?: string }

const tools: Tool[] = [
  { name: 'extract_text', label: '文本提取', description: '提取 PDF 纯文本内容', icon: DocumentTextIcon, category: 'basic' },
  { name: 'extract_structured', label: '结构化提取', description: '提取带页面信息的结构化数据', icon: CubeIcon, category: 'basic' },
  { name: 'get_page_count', label: '页数查询', description: '获取 PDF 文件页数', icon: DocumentIcon, category: 'basic' },
  { name: 'search_keywords', label: '关键词搜索', description: '在 PDF 中搜索关键词', icon: MagnifyingGlassIcon, category: 'basic' },
  { name: 'extrude_to_server_wiki', label: '服务端Wiki构建', description: '服务器端闭环提取并保存到Wiki目录', icon: ServerIcon, category: 'wiki' },
  { name: 'extrude_to_agent_payload', label: '本地Wiki投影', description: '返回带YAML元数据的Markdown报文', icon: CloudIcon, category: 'wiki' },
  { name: 'compile_to_wiki', label: '编译到Wiki', description: '将PDF编译到知识库，生成编译提示', icon: BookOpenIcon, category: 'knowledge' },
  { name: 'incremental_compile', label: '增量编译', description: '扫描raw/目录，仅编译新增或变更的PDF', icon: SparklesIcon, category: 'knowledge' },
  { name: 'search_knowledge', label: '搜索知识库', description: '全文搜索Wiki条目，返回排序结果', icon: MagnifyingGlassIcon, category: 'knowledge' },
  { name: 'rebuild_index', label: '重建索引', description: '重建Tantivy全文索引和petgraph链接图', icon: ArrowPathIcon, category: 'knowledge' },
  { name: 'get_entry_context', label: '获取条目上下文', description: '获取知识条目的N跳邻居', icon: BeakerIcon, category: 'knowledge' },
  { name: 'find_orphans', label: '查找孤立条目', description: '查找无链接的知识条目', icon: LightBulbIcon, category: 'knowledge' },
  { name: 'suggest_links', label: '建议链接', description: '基于标签相似度建议潜在链接', icon: ChartBarIcon, category: 'knowledge' },
  { name: 'export_concept_map', label: '导出概念图', description: '导出Mermaid.js格式的概念图', icon: CloudArrowUpIcon, category: 'knowledge' },
  { name: 'check_quality', label: '质量检查', description: '分析Wiki质量，检测问题', icon: QueueListIcon, category: 'knowledge' },
  { name: 'micro_compile', label: '微编译', description: '按需提取PDF内容到当前对话', icon: PuzzlePieceIcon, category: 'knowledge' },
  { name: 'aggregate_entries', label: '聚合条目', description: '识别可聚合的L1条目簇', icon: CircleStackIcon, category: 'knowledge' },
  { name: 'hypothesis_test', label: '假设测试', description: '发现矛盾条目对并生成辩论框架', icon: AcademicCapIcon, category: 'knowledge' },
  { name: 'recompile_entry', label: '重编译条目', description: '重编译单个Wiki条目', icon: DocumentMagnifyingGlassIcon, category: 'knowledge' }
]

const toolNameMap: Record<string, string> = {
  extract_text: '文本提取',
  extract_structured: '结构化提取',
  get_page_count: '页数查询',
  search_keywords: '关键词搜索',
  extrude_to_server_wiki: '服务端Wiki构建',
  extrude_to_agent_payload: '本地Wiki投影',
  compile_to_wiki: '编译到Wiki',
  incremental_compile: '增量编译',
  search_knowledge: '搜索知识库',
  rebuild_index: '重建索引',
  get_entry_context: '获取条目上下文',
  find_orphans: '查找孤立条目',
  suggest_links: '建议链接',
  export_concept_map: '导出概念图',
  check_quality: '质量检查',
  micro_compile: '微编译',
  aggregate_entries: '聚合条目',
  hypothesis_test: '假设测试',
  recompile_entry: '重编译条目'
}

const toolInstructions: Record<string, string> = {
  extract_text: '从 PDF 文件中提取纯文本内容。系统自动质量探测，数字文档用 Pdfium 本地提取，扫描件自动切换 VLM 多模态增强。',
  extract_structured: '提取包含页面信息、位置坐标的结构化数据。返回每页文本及 Bounding Box，适用于精确定位场景。',
  get_page_count: '快速获取 PDF 页数。mmap 零拷贝加载，响应极快，适合批量处理前预判文件规模。',
  search_keywords: '在 PDF 中搜索关键词，返回匹配位置、页码和上下文。支持多关键词（逗号分隔），可选区分大小写。',
  extrude_to_server_wiki: '服务器端闭环完成提取、落盘与索引更新。结果保存到 Wiki 三级存储（raw/wiki/scheme），自动更新 MAP.md。',
  extrude_to_agent_payload: '服务器仅计算，将带 YAML 元数据的 Markdown 报文发还给 Agent。适合本地 Wiki 构建。',
  compile_to_wiki: '将PDF编译到知识库：提取文本、保存到raw/、生成编译提示。这是Karpathy编译器模式的主入口。',
  incremental_compile: '扫描raw/目录，检测新增或变更的PDF（SHA-256哈希比较），仅编译需要的文件。',
  search_knowledge: '使用Tantivy全文搜索所有Wiki条目。支持关键词、短语、布尔查询，返回排序结果和片段。',
  rebuild_index: '从Wiki Markdown文件重建所有索引（Tantivy全文 + petgraph链接图）。批量变更后使用。',
  get_entry_context: '获取知识条目的N跳邻居（通过链接关系、标签共现）。返回关联条目用于上下文扩展。',
  find_orphans: '查找无入站或出站链接的知识条目。这些是需要整合的候选。',
  suggest_links: '基于标签相似度（Jaccard指数）为条目建议潜在链接。帮助发现隐藏关联。',
  export_concept_map: '导出条目周围的概念图为Mermaid.js文本。显示N跳内的关系，用于可视化。',
  check_quality: '分析Wiki质量：检测缺失标签、孤立条目、断链、样式问题。返回综合报告。',
  micro_compile: '按需从PDF提取内容到当前对话上下文。结果不保存到wiki，直接注入AI会话。',
  aggregate_entries: '识别可聚合为L2摘要条目的L1条目簇。返回共享标签的簇供AI综合。',
  hypothesis_test: '发现显式矛盾的条目对，生成辩论框架供AI解决矛盾。',
  recompile_entry: '重编译单个Wiki条目：升级版本、创建备份、检查源PDF变更、生成重编译提示。'
}

const mcpReady = ref(false)
const selectedTool = ref<Tool | null>(null)
const filePath = ref('')
const keywords = ref('')
const caseSensitive = ref(false)
const enableHighlight = ref(true)
const wikiBasePath = ref('./wiki')
const knowledgeBase = ref('')
const searchQuery = ref('')
const entryPath = ref('')
const domain = ref('')
const pageRange = ref('')
const hops = ref(2)
const depth = ref(2)
const topK = ref(10)
const limit = ref(10)
const loading = ref(false)
const result = ref<any>(null)
const error = ref<string | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const executionLog = ref<LogEntry[]>([])

const basicTools = computed(() => tools.filter(t => t.category === 'basic'))
const wikiTools = computed(() => tools.filter(t => t.category === 'wiki'))
const knowledgeTools = computed(() => tools.filter(t => t.category === 'knowledge'))
const canExecute = computed(() => {
  if (!selectedTool.value) return false
  if (needsFilePath.value && !filePath.value) return false
  if (selectedTool.value.name === 'search_keywords' && !keywords.value) return false
  if (needsKnowledgeBase.value && !knowledgeBase.value) return false
  if (selectedTool.value.name === 'search_knowledge' && !searchQuery.value) return false
  if (selectedTool.value.name === 'get_entry_context' && !entryPath.value) return false
  if (selectedTool.value.name === 'suggest_links' && !entryPath.value) return false
  if (selectedTool.value.name === 'export_concept_map' && !entryPath.value) return false
  if (selectedTool.value.name === 'recompile_entry' && !entryPath.value) return false
  return true
})
const needsFilePath = computed(() => ['extract_text', 'extract_structured', 'get_page_count', 'search_keywords', 'extrude_to_server_wiki', 'extrude_to_agent_payload', 'compile_to_wiki', 'micro_compile'].includes(selectedTool.value?.name || ''))
const needsKnowledgeBase = computed(() => ['compile_to_wiki', 'incremental_compile', 'search_knowledge', 'rebuild_index', 'get_entry_context', 'find_orphans', 'suggest_links', 'export_concept_map', 'check_quality', 'aggregate_entries', 'hypothesis_test', 'recompile_entry'].includes(selectedTool.value?.name || ''))
const successRate = computed(() => {
  if (executionLog.value.length === 0) return 100
  return (executionLog.value.filter(l => l.success).length / executionLog.value.length) * 100
})
const avgLatency = computed(() => {
  if (executionLog.value.length === 0) return 0
  return Math.round(executionLog.value.reduce((s, l) => s + l.duration, 0) / executionLog.value.length)
})
const recentLogs = computed(() => executionLog.value.slice(-10).reverse())

const triggerFileUpload = () => fileInput.value?.click()
const handleFileSelect = (e: Event) => { const f = (e.target as HTMLInputElement).files?.[0]; if (f) { filePath.value = f.name; error.value = null } }
const formatTime = (ts: number) => { const d = new Date(ts); return `${String(d.getHours()).padStart(2,'0')}:${String(d.getMinutes()).padStart(2,'0')}:${String(d.getSeconds()).padStart(2,'0')}` }
const formatResult = (data: any) => typeof data === 'string' ? data : JSON.stringify(data, null, 2)
const loadLog = () => { const s = localStorage.getItem('mcp-execution-log'); if (s) { try { executionLog.value = JSON.parse(s) } catch {} } }
const saveLog = (entry: LogEntry) => { executionLog.value.push(entry); localStorage.setItem('mcp-execution-log', JSON.stringify(executionLog.value.slice(-100))) }
const clearLog = () => { executionLog.value = []; localStorage.removeItem('mcp-execution-log') }
const checkMcpStatus = () => {
  const s = localStorage.getItem('mcp-config')
  const defaultCmd = '/root/rsut_pdf_mcp/pdf-module-rs/target/release/pdf-mcp'
  if (s) { 
    try { 
      const config = JSON.parse(s)
      mcpReady.value = !!(config.serverCommand || defaultCmd)
    } catch { 
      mcpReady.value = true // Default to ready
    } 
  }
  else mcpReady.value = true // Default to ready with default command
}

const executeTool = async () => {
  if (!canExecute.value) { error.value = '请完善参数'; return }
  loading.value = true; error.value = null; result.value = null
  const start = Date.now()
  try {
    const config = localStorage.getItem('mcp-config')
    const defaultCmd = '/root/rsut_pdf_mcp/pdf-module-rs/target/release/pdf-mcp'
    const cmd = config ? JSON.parse(config).serverCommand || defaultCmd : defaultCmd
    const args: Record<string, any> = {}
    
    if (needsFilePath.value) {
      args.file_path = filePath.value
    }
    
    if (needsKnowledgeBase.value) {
      args.knowledge_base = knowledgeBase.value
    }
    
    switch (selectedTool.value!.name) {
      case 'search_keywords':
        args.keywords = keywords.value.split(',').map(k => k.trim())
        args.case_sensitive = caseSensitive.value
        break
      case 'extract_structured':
        args.enable_highlight = enableHighlight.value
        break
      case 'extrude_to_server_wiki':
        args.wiki_base_path = wikiBasePath.value
        break
      case 'compile_to_wiki':
        args.pdf_path = filePath.value
        if (domain.value) args.domain = domain.value
        break
      case 'search_knowledge':
        args.query = searchQuery.value
        args.limit = limit.value
        break
      case 'get_entry_context':
        args.entry_path = entryPath.value
        args.hops = hops.value
        break
      case 'suggest_links':
        args.entry_path = entryPath.value
        args.top_k = topK.value
        break
      case 'export_concept_map':
        args.entry_path = entryPath.value
        args.depth = depth.value
        break
      case 'recompile_entry':
        args.entry_path = entryPath.value
        break
      case 'micro_compile':
        args.pdf_path = filePath.value
        if (pageRange.value) args.page_range = pageRange.value
        break
    }
    
    const request = { jsonrpc: '2.0', id: Date.now(), method: 'tools/call', params: { name: selectedTool.value!.name, arguments: args } }
    const response = await fetch('/api/mcp', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ command: cmd, request }) })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const data = await response.json()
    const text = data.result?.content?.[0]?.text || JSON.stringify(data.result)
    let parsed: any
    try { parsed = JSON.parse(text) } catch { parsed = text }
    const duration = Date.now() - start
    result.value = { data: parsed, duration }
    saveLog({ tool: selectedTool.value!.name, success: true, duration, timestamp: Date.now() })
  } catch (err: any) {
    const duration = Date.now() - start
    error.value = err.message || '执行失败'
    saveLog({ tool: selectedTool.value!.name, success: false, duration, timestamp: Date.now(), error: err.message })
  } finally { loading.value = false }
}

onMounted(() => { loadLog(); checkMcpStatus() })
</script>
