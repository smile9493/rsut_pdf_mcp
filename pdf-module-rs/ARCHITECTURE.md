# rsut-pdf-mcp Architecture

AI-native knowledge compilation engine — PDF extraction + Karpathy compiler pattern + fulltext search + knowledge graph. Pure Rust, single binary.

## Design Principles

1. **Karpathy Compiler Mode**: Knowledge pre-compiled to structured Markdown. Markdown is the single source of truth.
2. **AI-Agent as UI**: No external GUI. All interaction via MCP tool calls from AI clients.
3. **Rebuildable Indexes**: All indexes (Tantivy, petgraph) can be fully reconstructed from wiki Markdown files. Zero data risk.
4. **FFI Safety**: `catch_unwind` levee isolates C++ pdfium panics from Rust.
5. **Pure Rust**: Single binary, zero external services, no database.

## Architecture

```
┌──────────────────────────────────────────────────┐
│            AI Client (Claude / Cursor)            │
│            20 MCP tools via JSON-RPC              │
└──────────────────────┬───────────────────────────┘
                       │ stdio
                       ▼
┌──────────────────────────────────────────────────┐
│                 pdf-mcp (server)                  │
│  server.rs → handle_request() → tool dispatch    │
├──────────────────────────────────────────────────┤
│                                                  │
│  ┌─── PDF Extraction (6 tools) ─────────────────┐│
│  │  extract_text / extract_structured           ││
│  │  get_page_count / search_keywords            ││
│  │  extrude_to_server_wiki                      ││
│  │  extrude_to_agent_payload                    ││
│  └──────────────────────────────────────────────┘│
│                                                  │
│  ┌─── Knowledge Engine (6 tools) ───────────────┐│
│  │  compile_to_wiki / incremental_compile       ││
│  │  recompile_entry / aggregate_entries         ││
│  │  check_quality / micro_compile               ││
│  │  hypothesis_test                             ││
│  └──────────────────────────────────────────────┘│
│                                                  │
│  ┌─── Cognitive Index (6 tools) ────────────────┐│
│  │  search_knowledge (Tantivy + CJK n-gram)    ││
│  │  rebuild_index                               ││
│  │  get_entry_context / find_orphans            ││
│  │  suggest_links / export_concept_map          ││
│  └──────────────────────────────────────────────┘│
└──────────────────────┬───────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        ▼                             ▼
┌───────────────┐         ┌───────────────────┐
│  PdfiumEngine │         │  VlmGateway       │
│  (local)      │         │  (conditional)    │
│  FFI levee    │         │  GPT-4o / Claude  │
└───────────────┘         │  GLM-4.6v / OCR  │
                          └───────────────────┘
```

## Knowledge Base Layout

```
knowledge_base/
├── raw/                   # Source PDFs + extraction markdown
├── wiki/                  # Compiled knowledge (Markdown)
│   ├── index.md           # Auto-generated navigation
│   ├── log.md             # Operation log
│   ├── .versions/         # Backup before recompile (v{N}.md)
│   └── <domain>/          # L1/L2/L3 entries
├── schema/                # Compilation instructions
├── .hash_cache            # Merkle hash for incremental compile
└── .rsut_index/           # Rebuildable indexes
    └── tantivy/           # Fulltext search index
```

## Crates

| Crate | 职责 |
|-------|------|
| `pdf-common` | 统一错误、DTO、配置、traits |
| `pdf-macros` | 过程宏 `#[derive(Builder)]` |
| `pdf-core` | PdfiumEngine + FileValidator + VlmPipeline + **KnowledgeEngine** + **FulltextIndex** + **GraphIndex** |
| `pdf-mcp` | MCP stdio 入口 (JSON-RPC) — 20 tools |
| `vlm-visual-gateway` | VLM 条件升级网关 |
| `pdf-dashboard` | HTTP 监控面板 |
| `pdf-wasm` | WASM 引擎 |

## Knowledge Engine Module (`pdf-core::knowledge`)

| 子模块 | 职责 | 关键类型 |
|--------|------|----------|
| `entry` | 统一 front matter 规范 | `KnowledgeEntry`, `EntryLevel`, `CompileStatus` |
| `hash_cache` | Merkle 增量变更检测 | `HashCache` |
| `engine` | 编译调度核心 | `KnowledgeEngine`, `CompileResult`, `RecompileResult` |
| `quality` | 质量分析 | `QualityReport`, `QualityIssue` |
| `index::fulltext` | Tantivy 全文检索 (CJK-aware) | `FulltextIndex`, `SearchHit` |
| `index::graph` | petgraph 知识图谱 | `GraphIndex`, `NeighborInfo`, `LinkSuggestion` |
| `index::tokenizer` | CJK n-gram 分词器 | `register_cjk_tokenizer()` |

## MCP Tool Inventory (20 tools)

### PDF Extraction (6)
| Tool | Description |
|------|-------------|
| `extract_text` | 纯文本提取 |
| `extract_structured` | 结构化提取 (per-page + bbox) |
| `get_page_count` | 页数查询 |
| `search_keywords` | 关键词搜索 (正则 + 二分页定位) |
| `extrude_to_server_wiki` | 提取到 server wiki |
| `extrude_to_agent_payload` | 提取 + 返回 Agent 编译提示 |

### Compilation (7)
| Tool | Description |
|------|-------------|
| `compile_to_wiki` | PDF → raw/ + 编译提示 (知识库入口) |
| `incremental_compile` | Merkle 哈希增量扫描，只编译变更的 PDF |
| `recompile_entry` | 单条目重编译 + 版本备份 + 漂移检测 |
| `aggregate_entries` | L1→L2 聚合候选发现 (标签社区检测) |
| `check_quality` | 全 wiki 质量扫描 |
| `micro_compile` | 即时 PDF 提取 (不写 wiki，注入对话) |
| `hypothesis_test` | 矛盾对发现 + 辩论框架生成 |

### Indexing (6)
| Tool | Description |
|------|-------------|
| `search_knowledge` | Tantivy 全文搜索 (CJK n-gram) |
| `rebuild_index` | 完全重建 Tantivy + petgraph |
| `get_entry_context` | N 跳邻居发现 |
| `find_orphans` | 孤立条目检测 |
| `suggest_links` | Jaccard 相似度链接建议 |
| `export_concept_map` | Mermaid.js 概念图导出 |

## Entry Format (YAML Front Matter)

```yaml
---
title: "概念名称"
domain: "IT"
source: "raw/paper.pdf"
page: 3
source_hash: "abc123..."
tags: ["http", "networking"]
level: L1
status: compiled
quality_score: 0.85
version: 1
contradictions: ["wiki/other/concept.md"]
related: ["wiki/it/related_concept.md"]
aggregated_from: []
created: 2026-05-04T00:00:00Z
updated: 2026-05-04T00:00:00Z
---
```

## Knowledge Pyramid

```
L3  Domain Map         (导航层，1 per domain)
    ↑ aggregated from
L2  Aggregation         (综述，同子主题多 L1 合并)
    ↑ aggregated from
L1  Atomic Concept      (原子概念，核心知识单元)
    ↑ compiled from
L0  Raw Extraction      (原始提取，PDF → text)
```

## Key Dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| `tantivy` | 0.22 | Full-text search index |
| `petgraph` | 0.7 | Knowledge graph (link analysis) |
| `pdfium-render` | 0.8 | PDF text extraction (FFI) |
| `sha2` | 0.10 | Content hashing (incremental compile) |
| `serde_yaml` | 0.9 | Front matter serialization |
| `tokio` | 1.x | Async runtime |
| `memmap2` | 0.9 | Zero-copy PDF loading |

## FFI Levee

```rust
pub fn safe_extract_text(data: &[u8]) -> PdfResult<String> {
    catch_unwind(AssertUnwindSafe(|| {
        // pdfium C++ FFI
    }))
    .map_err(|_| PdfModuleError::Extraction("FFI panic".into()))?
    .map_err(|e| PdfModuleError::Extraction(format!("Pdfium: {}", e)))
}
```
