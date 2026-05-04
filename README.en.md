# PDF MCP Module

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/smile9493/rsut_pdf_mcp)](https://github.com/smile9493/rsut_pdf_mcp/releases)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue.svg)](https://modelcontextprotocol.io/)

**AI-Native Knowledge Compilation Engine** — Compile PDF documents into structured knowledge bases, providing long-term memory and reasoning backend for AI clients like Claude and Cursor.

English | [简体中文](./README.md)

## ✨ Features

- 🔥 **Karpathy Compiler Pattern** — PDFs pre-compiled to structured Markdown, knowledge is cumulative and explainable
- 🧠 **Cognitive Index Layer** — Tantivy full-text search + petgraph knowledge graph with CJK tokenizer
- 🚀 **Pure Rust** — Single binary deployment, zero external dependencies, high-performance FFI levee
- 🔄 **Incremental Compilation** — Merkle hash detection, only compile changed PDFs
- 🎯 **20 MCP Tools** — Covering PDF extraction, knowledge compilation, and cognitive indexing

## 📦 Installation

### One-line Install

```bash
curl -fsSL https://raw.githubusercontent.com/smile9493/rsut_pdf_mcp/main/install.sh | bash
```

### Docker

```bash
docker pull smile9493/pdf-mcp:latest
```

### Build from Source

```bash
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp/pdf-module-rs
cargo build --release --bin pdf-mcp
```

## 🚀 Quick Start

### 1. Configure AI Client

**Cursor** (`~/.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "pdf-mcp": {
      "command": "/opt/pdf-module/pdf-mcp",
      "env": {
        "PDFIUM_LIB_PATH": "/opt/pdf-module/lib/libpdfium.so"
      }
    }
  }
}
```

**Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "pdf-mcp": {
      "command": "/opt/pdf-module/pdf-mcp"
    }
  }
}
```

### 2. Compile PDF to Knowledge Base

```
User: Compile /path/to/paper.pdf into the knowledge base

AI: [Calls compile_to_wiki tool]
PDF compiled to knowledge base:
- Raw extraction: raw/paper.md
- Compile prompt: raw/paper.compile_prompt.md

Please read the extracted content, extract core concepts, and create atomic entries...
```

### 3. Search Knowledge Base

```
User: Search for knowledge about HTTP/2

AI: [Calls search_knowledge tool]
Found 3 related entries:
1. [IT] HTTP/2 Multiplexing (score: 0.92)
2. [IT] HTTP/2 Header Compression (score: 0.85)
3. [Network] HTTP/2 vs HTTP/1.1 Comparison (score: 0.78)
```

## 🛠️ MCP Tools (20)

### PDF Extraction (6)

| Tool | Description |
|------|-------------|
| `extract_text` | Extract plain text from PDF |
| `extract_structured` | Extract structured data (per-page text + bbox) |
| `get_page_count` | Get PDF page count |
| `search_keywords` | Search keywords within PDF |
| `extrude_to_server_wiki` | Extract to server-side Wiki |
| `extrude_to_agent_payload` | Return Markdown payload |

### Knowledge Compilation (7)

| Tool | Description |
|------|-------------|
| `compile_to_wiki` | PDF → knowledge base compilation entry point |
| `incremental_compile` | Incremental compilation (hash detection) |
| `recompile_entry` | Single entry recompilation + version backup |
| `aggregate_entries` | L1→L2 aggregation candidate discovery |
| `check_quality` | Wiki quality scan |
| `micro_compile` | On-demand extraction (not persisted) |
| `hypothesis_test` | Contradiction discovery + debate framework |

### Cognitive Index (6)

| Tool | Description |
|------|-------------|
| `search_knowledge` | Tantivy full-text search (CJK support) |
| `rebuild_index` | Rebuild all indexes |
| `get_entry_context` | N-hop neighbor discovery |
| `find_orphans` | Orphan entry detection |
| `suggest_links` | Link suggestions (Jaccard similarity) |
| `export_concept_map` | Mermaid.js concept map export |

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────┐
│            AI Client (Claude / Cursor)            │
│            20 MCP tools via JSON-RPC              │
└──────────────────────┬───────────────────────────┘
                       │ stdio
                       ▼
┌──────────────────────────────────────────────────┐
│                 pdf-mcp (server)                  │
├──────────────────────────────────────────────────┤
│  PDF Extraction │ Knowledge Engine │ Cog. Index  │
└──────────────────────┬───────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        ▼                             ▼
┌───────────────┐         ┌───────────────────┐
│  PdfiumEngine │         │  VlmGateway       │
│  (FFI levee)  │         │  (conditional)    │
└───────────────┘         └───────────────────┘
```

## 📁 Knowledge Base Structure

```
knowledge_base/
├── raw/                   # Raw PDF extractions
├── wiki/                  # Compiled knowledge
│   ├── index.md           # Global navigation
│   ├── log.md             # Operation log
│   ├── .versions/         # Recompile backups
│   └── <domain>/          # Domain entries
├── schema/                # Compilation instructions
├── .hash_cache            # Merkle hash cache
└── .rsut_index/           # Rebuildable indexes
    └── tantivy/           # Full-text search index
```

## 📝 Entry Format

Each wiki entry uses standardized YAML front matter:

```yaml
---
title: "HTTP/2 Multiplexing"
domain: "IT"
source: "raw/rfc7540.pdf"
page: 12
tags: ["http", "networking", "protocol"]
level: L1
status: compiled
quality_score: 0.85
version: 1
contradictions: []
related: ["wiki/it/http1.md"]
created: 2026-05-04T00:00:00Z
updated: 2026-05-04T00:00:00Z
---

# HTTP/2 Multiplexing

Body content...
```

## 🗺️ Knowledge Pyramid

```
L3  Domain Map      (Navigation layer, 1 per domain)
    ↑ aggregated from
L2  Aggregation      (Summary, multiple L1 on same sub-topic)
    ↑ aggregated from
L1  Atomic Concept   (Atomic concept, core knowledge unit)
    ↑ compiled from
L0  Raw Extraction   (Raw extraction, PDF → text)
```

## ⚙️ Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PDFIUM_LIB_PATH` | PDFium library path | Auto-detect |
| `VLM_API_KEY` | VLM API key | - |
| `VLM_MODEL` | Model name | `glm-4v-flash` |
| `VLM_ENDPOINT` | API endpoint | Zhipu API |
| `DASHBOARD_PORT` | Dashboard port | `8000` |

## 📥 Downloads

| Platform | File |
|----------|------|
| Linux x64 | `pdf-mcp-linux-x64.tar.gz` |
| Linux ARM64 | `pdf-mcp-linux-arm64.tar.gz` |
| macOS Intel | `pdf-mcp-macos-x64.tar.gz` |
| macOS Apple Silicon | `pdf-mcp-macos-arm64.tar.gz` |
| Windows x64 | `pdf-mcp-windows-x64.zip` |

[GitHub Releases](https://github.com/smile9493/rsut_pdf_mcp/releases)

## 📄 License

[MIT](LICENSE)
