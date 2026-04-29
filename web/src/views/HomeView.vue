<template>
  <div class="p-2xl">
    <header class="mb-2xl">
      <h1 class="text-h1 font-bold">
        {{ t('dashboard.title') }}
      </h1>
      <p class="text-text-secondary mt-sm">
        {{ t('dashboard.subtitle') }}
      </p>
    </header>

    <!-- MCP Status -->
    <section class="mb-2xl">
      <div class="grid grid-cols-4 gap-lg">
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="text-sm text-text-secondary mb-xs">
            {{ t('dashboard.mcpTools') }}
          </div>
          <div class="text-2xl font-bold text-primary">
            4
          </div>
          <div class="text-xs text-text-muted mt-xs">
            extract_text, extract_structured, get_page_count, search_keywords
          </div>
        </div>
        
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="text-sm text-text-secondary mb-xs">
            {{ t('dashboard.engine') }}
          </div>
          <div class="text-2xl font-bold text-primary">
            PDFium
          </div>
          <div class="text-xs text-text-muted mt-xs">
            {{ t('dashboard.engineDesc') }}
          </div>
        </div>
        
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="text-sm text-text-secondary mb-xs">
            {{ t('dashboard.protocol') }}
          </div>
          <div class="text-2xl font-bold text-primary">
            stdio
          </div>
          <div class="text-xs text-text-muted mt-xs">
            MCP JSON-RPC 2.0
          </div>
        </div>
        
        <div class="bg-surface rounded-lg p-lg border border-border">
          <div class="text-sm text-text-secondary mb-xs">
            {{ t('dashboard.vlm') }}
          </div>
          <div
            class="text-2xl font-bold"
            :class="vlmEnabled ? 'text-primary' : 'text-text-muted'"
          >
            {{ vlmEnabled ? 'ON' : 'OFF' }}
          </div>
          <div class="text-xs text-text-muted mt-xs">
            {{ vlmModel || t('dashboard.vlmDisabled') }}
          </div>
        </div>
      </div>
    </section>

    <!-- Tools -->
    <section class="mb-2xl">
      <h2 class="text-lg font-semibold mb-lg">
        {{ t('dashboard.availableTools') }}
      </h2>
      <div class="grid grid-cols-2 gap-lg">
        <div
          v-for="tool in tools"
          :key="tool.name"
          class="bg-surface rounded-lg p-lg border border-border hover:border-primary transition-colors"
        >
          <div class="flex items-start gap-md">
            <div class="w-10 h-10 rounded bg-primary/10 flex items-center justify-center flex-shrink-0">
              <component
                :is="tool.icon"
                class="w-5 h-5 text-primary"
              />
            </div>
            <div class="flex-1">
              <div class="font-medium text-text-primary">
                {{ tool.name }}
              </div>
              <div class="text-sm text-text-secondary mt-xs">
                {{ tool.description }}
              </div>
              <div class="mt-sm">
                <span
                  v-for="param in tool.params"
                  :key="param"
                  class="inline-block text-xs font-mono bg-bg px-sm py-xs rounded mr-xs"
                >
                  {{ param }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Quick Actions -->
    <section class="mb-2xl">
      <h2 class="text-lg font-semibold mb-lg">
        {{ t('dashboard.quickActions') }}
      </h2>
      <div class="grid grid-cols-4 gap-lg">
        <router-link
          to="/extract"
          class="bg-surface rounded-lg p-lg border border-border hover:border-primary transition-colors text-center"
        >
          <DocumentTextIcon class="w-8 h-8 mx-auto mb-sm text-primary" />
          <div class="font-medium">
            {{ t('nav.extract') }}
          </div>
        </router-link>
        
        <router-link
          to="/search"
          class="bg-surface rounded-lg p-lg border border-border hover:border-primary transition-colors text-center"
        >
          <MagnifyingGlassIcon class="w-8 h-8 mx-auto mb-sm text-primary" />
          <div class="font-medium">
            {{ t('nav.search') }}
          </div>
        </router-link>
        
        <router-link
          to="/batch"
          class="bg-surface rounded-lg p-lg border border-border hover:border-primary transition-colors text-center"
        >
          <DocumentDuplicateIcon class="w-8 h-8 mx-auto mb-sm text-primary" />
          <div class="font-medium">
            {{ t('nav.batch') }}
          </div>
        </router-link>
        
        <router-link
          to="/mcp-tools"
          class="bg-surface rounded-lg p-lg border border-border hover:border-primary transition-colors text-center"
        >
          <CommandLineIcon class="w-8 h-8 mx-auto mb-sm text-primary" />
          <div class="font-medium">
            {{ t('dashboard.testTools') }}
          </div>
        </router-link>
      </div>
    </section>

    <!-- Architecture -->
    <section>
      <h2 class="text-lg font-semibold mb-lg">
        {{ t('dashboard.architecture') }}
      </h2>
      <div class="bg-surface rounded-lg p-lg border border-border">
        <pre class="font-mono text-sm text-text-secondary overflow-x-auto">
AI Agent (Cursor/Claude Desktop)
    │
    └──► pdf-mcp (stdio JSON-RPC)
             │
             ├── extract_text
             ├── extract_structured
             ├── get_page_count
             └── search_keywords
                       │
                       └──► PdfiumEngine (FFI safe)
                                │
                                └──► PDF File
        </pre>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  DocumentTextIcon,
  MagnifyingGlassIcon,
  DocumentDuplicateIcon,
  CommandLineIcon,
  CubeIcon,
  DocumentIcon,
  CircleStackIcon
} from '@heroicons/vue/24/outline'

const { t } = useI18n()

const tools = [
  {
    name: 'extract_text',
    description: '提取 PDF 纯文本内容，移除格式信息',
    params: ['file_path'],
    icon: DocumentTextIcon
  },
  {
    name: 'extract_structured',
    description: '提取结构化数据（段落、表格），包含 bbox 信息',
    params: ['file_path'],
    icon: CubeIcon
  },
  {
    name: 'get_page_count',
    description: '获取 PDF 总页数',
    params: ['file_path'],
    icon: DocumentIcon
  },
  {
    name: 'search_keywords',
    description: '在 PDF 中搜索关键词，返回匹配位置和上下文',
    params: ['file_path', 'keywords', 'case_sensitive?'],
    icon: MagnifyingGlassIcon
  }
]

const vlmConfig = localStorage.getItem('vlm-config')
const vlmEnabled = vlmConfig ? JSON.parse(vlmConfig).provider : false
const vlmModel = vlmConfig ? JSON.parse(vlmConfig).provider : ''
</script>
