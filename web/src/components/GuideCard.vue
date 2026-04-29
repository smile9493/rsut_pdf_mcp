<template>
  <div class="bg-surface border border-border rounded-lg p-lg mb-lg">
    <div class="flex items-center justify-between mb-md">
      <h3 class="text-lg font-semibold text-text-primary flex items-center gap-sm">
        <BookOpenIcon class="w-5 h-5 text-primary" />
        {{ t('guide.title') }}
      </h3>
      <button
        class="text-sm text-primary hover:underline"
        @click="expanded = !expanded"
      >
        {{ expanded ? t('guide.collapse') : t('guide.expand') }}
      </button>
    </div>

    <div v-if="expanded" class="space-y-lg">
      <section class="border-l-4 border-primary pl-md">
        <h4 class="font-semibold text-text-primary mb-sm">
          {{ t('guide.philosophy.title') }}
        </h4>
        <div class="grid grid-cols-2 gap-md">
          <div class="bg-bg rounded p-sm">
            <div class="flex items-center gap-xs mb-xs">
              <BoltIcon class="w-4 h-4 text-warning" />
              <span class="font-medium text-sm">{{ t('guide.philosophy.economy') }}</span>
            </div>
            <p class="text-xs text-text-secondary">
              {{ t('guide.philosophy.economyDesc') }}
            </p>
          </div>
          <div class="bg-bg rounded p-sm">
            <div class="flex items-center gap-xs mb-xs">
              <ShieldCheckIcon class="w-4 h-4 text-success" />
              <span class="font-medium text-sm">{{ t('guide.philosophy.intercept') }}</span>
            </div>
            <p class="text-xs text-text-secondary">
              {{ t('guide.philosophy.interceptDesc') }}
            </p>
          </div>
          <div class="bg-bg rounded p-sm">
            <div class="flex items-center gap-xs mb-xs">
              <CpuChipIcon class="w-4 h-4 text-info" />
              <span class="font-medium text-sm">{{ t('guide.philosophy.hardware') }}</span>
            </div>
            <p class="text-xs text-text-secondary">
              {{ t('guide.philosophy.hardwareDesc') }}
            </p>
          </div>
          <div class="bg-bg rounded p-sm">
            <div class="flex items-center gap-xs mb-xs">
              <AdjustmentsHorizontalIcon class="w-4 h-4 text-accent" />
              <span class="font-medium text-sm">{{ t('guide.philosophy.dialectics') }}</span>
            </div>
            <p class="text-xs text-text-secondary">
              {{ t('guide.philosophy.dialecticsDesc') }}
            </p>
          </div>
        </div>
      </section>

      <section>
        <h4 class="font-semibold text-text-primary mb-sm">
          {{ t('guide.tools.title') }}
        </h4>
        <div class="space-y-sm">
          <div class="bg-bg rounded p-md">
            <div class="flex items-center gap-sm mb-xs">
              <span class="badge badge-primary text-xs">extract_text</span>
              <span class="text-sm font-medium">{{ t('guide.tools.extractText') }}</span>
            </div>
            <p class="text-xs text-text-secondary">
              {{ t('guide.tools.extractTextDesc') }}
            </p>
          </div>

          <div class="bg-bg rounded p-md">
            <div class="flex items-center gap-sm mb-xs">
              <span class="badge badge-primary text-xs">extract_structured</span>
              <span class="text-sm font-medium">{{ t('guide.tools.structured') }}</span>
            </div>
            <p class="text-xs text-text-secondary">
              {{ t('guide.tools.structuredDesc') }}
            </p>
          </div>

          <div class="bg-bg rounded p-md">
            <div class="flex items-center gap-sm mb-xs">
              <span class="badge badge-primary text-xs">search_keywords</span>
              <span class="text-sm font-medium">{{ t('guide.tools.search') }}</span>
            </div>
            <p class="text-xs text-text-secondary">
              {{ t('guide.tools.searchDesc') }}
            </p>
          </div>

          <div class="bg-bg rounded p-md border border-primary/30">
            <div class="flex items-center gap-sm mb-xs">
              <span class="badge badge-success text-xs">NEW</span>
              <span class="badge badge-primary text-xs">extrude_to_server_wiki</span>
              <span class="text-sm font-medium">{{ t('guide.tools.serverWiki') }}</span>
            </div>
            <p class="text-xs text-text-secondary mb-sm">
              {{ t('guide.tools.serverWikiDesc') }}
            </p>
            <div class="bg-surface rounded p-xs font-mono text-xs">
              <div class="text-text-muted mb-xs">// {{ t('guide.example') }}</div>
              <div class="text-success">file_path:</div> <span class="text-text-primary">"/path/to/doc.pdf"</span>
              <div class="text-success">wiki_base_path:</div> <span class="text-text-primary">"./wiki"</span>
            </div>
          </div>

          <div class="bg-bg rounded p-md border border-primary/30">
            <div class="flex items-center gap-sm mb-xs">
              <span class="badge badge-success text-xs">NEW</span>
              <span class="badge badge-primary text-xs">extrude_to_agent_payload</span>
              <span class="text-sm font-medium">{{ t('guide.tools.agentPayload') }}</span>
            </div>
            <p class="text-xs text-text-secondary mb-sm">
              {{ t('guide.tools.agentPayloadDesc') }}
            </p>
            <div class="bg-surface rounded p-xs font-mono text-xs">
              <div class="text-text-muted mb-xs">// {{ t('guide.returns') }}</div>
              <div class="text-warning">YAML Frontmatter</div> + <div class="text-info">Markdown Content</div>
            </div>
          </div>
        </div>
      </section>

      <section>
        <h4 class="font-semibold text-text-primary mb-sm">
          {{ t('guide.quality.title') }}
        </h4>
        <div class="bg-bg rounded p-md">
          <div class="flex items-center justify-between mb-sm">
            <span class="text-sm font-medium">{{ t('guide.quality.autoDetect') }}</span>
            <span class="badge badge-success text-xs">ON</span>
          </div>
          <div class="space-y-xs text-xs text-text-secondary">
            <div class="flex items-center gap-xs">
              <span class="w-2 h-2 rounded-full bg-success"></span>
              <span>{{ t('guide.quality.digital') }}</span>
            </div>
            <div class="flex items-center gap-xs">
              <span class="w-2 h-2 rounded-full bg-warning"></span>
              <span>{{ t('guide.quality.scanned') }}</span>
            </div>
            <div class="flex items-center gap-xs">
              <span class="w-2 h-2 rounded-full bg-info"></span>
              <span>{{ t('guide.quality.hybrid') }}</span>
            </div>
          </div>
        </div>
      </section>

      <section>
        <h4 class="font-semibold text-text-primary mb-sm">
          {{ t('guide.wiki.title') }}
        </h4>
        <div class="bg-bg rounded p-md font-mono text-xs">
          <div class="text-text-muted mb-xs">wiki/</div>
          <div class="ml-md">
            <div class="text-success">├── raw/</div>
            <div class="ml-md text-text-secondary">└── [hash].raw.md</div>
            <div class="text-info">├── wiki/</div>
            <div class="ml-md text-text-secondary">└── (entity pages)</div>
            <div class="text-warning">├── scheme/</div>
            <div class="ml-md text-text-secondary">└── (type constraints)</div>
            <div class="text-primary">└── MAP.md</div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  BookOpenIcon,
  BoltIcon,
  ShieldCheckIcon,
  CpuChipIcon,
  AdjustmentsHorizontalIcon
} from '@heroicons/vue/24/outline'

const { t } = useI18n()
const expanded = ref(true)
</script>
