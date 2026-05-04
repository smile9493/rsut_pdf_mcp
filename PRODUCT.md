# PDF Module - Product Context

## Users

**Primary users**: AI Agent users who need to compile PDF documents into structured knowledge bases
- AI Agent users (Cursor, Claude Desktop) — need PDF extraction and knowledge compilation
- Developers — need API integration, SDK usage, deployment
- Knowledge workers — need to build personal knowledge bases from PDF documents

**Secondary users**: 
- Data analysts — need batch processing, structured extraction
- Content managers — need keyword search, document analysis

## Product Purpose

AI-native knowledge compilation engine with:
- **Karpathy Compiler Pattern**: PDFs pre-compiled to structured Markdown
- **Cognitive Index Layer**: Tantivy full-text search + petgraph knowledge graph
- **Incremental Compilation**: Merkle hash-based change detection
- **MCP Protocol**: 20 tools for AI Agent integration
- **Pure Rust**: Single binary, zero external services

## Key Features

### PDF Extraction (6 tools)
- Text extraction with pdfium engine
- Structured extraction with page-level bbox
- Keyword search within PDFs
- Wiki export for knowledge compilation

### Knowledge Compilation (7 tools)
- PDF → knowledge base compilation
- Incremental compilation with hash detection
- Single entry recompilation with version backup
- L1→L2 aggregation discovery
- Quality analysis and drift detection

### Cognitive Index (6 tools)
- Full-text search with CJK support
- Knowledge graph with neighbor discovery
- Orphan detection and link suggestions
- Mermaid.js concept map export

## Brand

**Tone**: Technical, professional, efficient, developer-friendly
**Personality**: Reliable, fast, intelligent, well-architected
**Voice**: Clear documentation, practical examples, performance-focused

## Anti-references

- Avoid: Overly complex UIs, unnecessary animations, marketing fluff
- Avoid: Generic SaaS landing page templates
- Avoid: Dark theme just because "tools look cool dark"

## Strategic Principles

1. **Knowledge compilation first** — PDFs become structured, searchable knowledge
2. **Developer experience** — Clear API docs, easy integration, good examples
3. **Transparency** — Show compilation status, index stats, quality reports
4. **Simplicity** — Single binary, no external services, Markdown as source of truth

## Register

**product** — This is a tool/admin interface where design serves the product
