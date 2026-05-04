//! Knowledge Engine — orchestrates compilation, indexing, and quality operations.
//!
//! This is the primary interface for the knowledge compilation pipeline.
//! It coordinates between PDF extraction, wiki storage, hash cache, and quality analysis.

use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::dto::{ExtractOptions, StructuredExtractionResult};
use crate::error::{PdfModuleError, PdfResult};
use crate::extractor::McpPdfPipeline;
use crate::knowledge::entry::{CompileStatus, EntryLevel, KnowledgeEntry};
use crate::knowledge::hash_cache::HashCache;
use crate::knowledge::quality::{self, QualityReport};
use crate::wiki::WikiStorage;

/// Result of a single compile operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompileResult {
    /// Path to the raw extraction file.
    pub raw_path: PathBuf,
    /// Paths to wiki entries created or updated.
    pub entries: Vec<CompileEntryResult>,
    /// Source PDF path.
    pub source: String,
    /// Hash of the source PDF.
    pub source_hash: String,
    /// Number of pages extracted.
    pub page_count: u32,
}

/// Result for a single entry within a compile operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompileEntryResult {
    pub title: String,
    pub domain: String,
    pub path: PathBuf,
    pub status: CompileStatus,
}

/// Result of incremental compilation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IncrementalResult {
    /// Total PDFs scanned.
    pub total_scanned: usize,
    /// PDFs that needed compilation.
    pub compiled: usize,
    /// PDFs that were skipped (already up to date).
    pub skipped: usize,
    /// Per-file results.
    pub results: Vec<CompileResult>,
}

/// The knowledge engine — central coordinator for the compilation pipeline.
///
/// Orchestrates PDF extraction, wiki storage, hash caching, and quality analysis
/// to build and maintain a knowledge base from PDF documents.
///
/// # Architecture
///
/// ```text
/// ┌──────────────────────┐
/// │  KnowledgeEngine     │
/// ├──────────────────────┤
/// │ - McpPdfPipeline     │ → PDF extraction
/// │ - WikiStorage        │ → Wiki file management
/// │ - HashCache          │ → Incremental compilation
/// │ - FulltextIndex      │ → Tantivy search
/// │ - GraphIndex         │ → Concept graph
/// └──────────────────────┘
/// ```
///
/// # Features
///
/// - **Incremental compilation**: Only recompile changed PDFs
/// - **Full-text search**: Tantivy-based search with CJK support
/// - **Concept graph**: Link suggestions and contradiction detection
/// - **Quality analysis**: Automated quality scoring
///
/// # Example
///
/// ```no_run
/// use pdf_core::{McpPdfPipeline, KnowledgeEngine};
/// use std::sync::Arc;
///
/// let pipeline = Arc::new(McpPdfPipeline::new(&config)?);
/// let engine = KnowledgeEngine::new(pipeline, "./kb")?;
///
/// // Compile a PDF to wiki
/// let result = engine.compile_to_wiki(path, Some("IT")).await?;
/// println!("Created {} entries", result.entries.len());
///
/// // Search the knowledge base
/// let hits = engine.search_knowledge("machine learning", 10)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct KnowledgeEngine {
    pipeline: Arc<McpPdfPipeline>,
    wiki: WikiStorage,
    knowledge_base: PathBuf,
}

impl KnowledgeEngine {
    pub fn new(
        pipeline: Arc<McpPdfPipeline>,
        knowledge_base: impl AsRef<Path>,
    ) -> PdfResult<Self> {
        let kb = knowledge_base.as_ref().to_path_buf();
        let wiki = WikiStorage::new(&kb)?;
        Ok(Self {
            pipeline,
            wiki,
            knowledge_base: kb,
        })
    }

    /// Get the path to the raw directory.
    pub fn raw_dir(&self) -> PathBuf {
        self.knowledge_base.join("raw")
    }

    /// Get the path to the wiki directory.
    pub fn wiki_dir(&self) -> PathBuf {
        self.knowledge_base.join("wiki")
    }

    /// Get the path to the knowledge base root.
    pub fn knowledge_base(&self) -> &Path {
        &self.knowledge_base
    }

    /// Compile a single PDF into the knowledge base.
    ///
    /// 1. Extract text from PDF
    /// 2. Save to raw/
    /// 3. Generate a compilation prompt payload
    /// 4. Update hash cache
    #[tracing::instrument(skip(self))]
    pub async fn compile_to_wiki(
        &self,
        pdf_path: &Path,
        domain: Option<&str>,
    ) -> PdfResult<CompileResult> {
        info!(source = ?pdf_path, "Starting compile_to_wiki");

        // Extract structured text
        let extraction = self
            .pipeline
            .extract_structured(pdf_path, &ExtractOptions::default())
            .await
            .map_err(|e| PdfModuleError::Extraction(format!("PDF extraction failed: {}", e)))?;

        // Save to raw/
        let wiki_result = self.wiki.save_raw(&extraction, pdf_path, 0.0)?;

        // Compute source hash
        let source_hash = HashCache::hash_file(pdf_path)?;

        // Determine domain
        let domain = domain.unwrap_or("未分类");
        let source_name = pdf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Create a pending L0 entry metadata
        let entry = KnowledgeEntry {
            title: source_name.to_string(),
            domain: domain.to_string(),
            source: Some(format!("raw/{}.md", source_name)),
            source_hash: Some(source_hash.clone()),
            level: EntryLevel::L0,
            status: CompileStatus::Pending,
            ..KnowledgeEntry::new(source_name, domain)
        };

        // Generate the compilation prompt using the AgentPayload pattern
        let prompt = self.build_compile_prompt(&entry, &extraction);
        let prompt_path = self
            .knowledge_base
            .join("raw")
            .join(format!("{}.compile_prompt.md", source_name));
        fs::write(&prompt_path, &prompt).map_err(|e| {
            PdfModuleError::StorageError(format!("Failed to write compile prompt: {}", e))
        })?;

        info!(
            source = ?pdf_path,
            raw_path = ?wiki_result.raw_path,
            prompt_path = ?prompt_path,
            pages = extraction.page_count,
            "compile_to_wiki complete"
        );

        Ok(CompileResult {
            raw_path: wiki_result.raw_path,
            entries: vec![CompileEntryResult {
                title: entry.title.clone(),
                domain: entry.domain.clone(),
                path: prompt_path,
                status: CompileStatus::Pending,
            }],
            source: pdf_path.to_string_lossy().to_string(),
            source_hash,
            page_count: extraction.page_count,
        })
    }

    /// Incremental compilation: scan raw/ for new or changed PDFs and compile only those.
    #[tracing::instrument(skip(self))]
    pub async fn incremental_compile(&self, raw_dir: &Path) -> PdfResult<IncrementalResult> {
        let mut cache = HashCache::load_or_create(&self.knowledge_base)?;

        // Find all PDF files in the raw directory
        let mut pdf_files = Vec::new();
        if raw_dir.exists() {
            for entry in fs::read_dir(raw_dir)
                .map_err(|e| PdfModuleError::StorageError(format!("Failed to read raw dir: {}", e)))?
            {
                let entry = entry.map_err(|e| {
                    PdfModuleError::StorageError(format!("Failed to read entry: {}", e))
                })?;
                let path = entry.path();
                if path.extension().map(|e| e == "pdf").unwrap_or(false) {
                    pdf_files.push(path);
                }
            }
        }

        let total = pdf_files.len();
        let mut compiled = 0usize;
        let mut skipped = 0usize;
        let mut results = Vec::new();

        for pdf_path in pdf_files {
            if cache.needs_compile(&pdf_path)? {
                match self.compile_to_wiki(&pdf_path, None).await {
                    Ok(result) => {
                        let entry_paths: Vec<String> = result
                            .entries
                            .iter()
                            .map(|e| e.path.to_string_lossy().to_string())
                            .collect();
                        cache.record_compile(&result.raw_path, entry_paths)?;
                        compiled += 1;
                        results.push(result);
                    }
                    Err(e) => {
                        warn!(source = ?pdf_path, error = %e, "Failed to compile PDF");
                        // Don't fail the entire batch for one file
                    }
                }
            } else {
                debug!(source = ?pdf_path, "Skipping unchanged PDF");
                skipped += 1;
            }
        }

        cache.save()?;

        info!(
            total = total,
            compiled = compiled,
            skipped = skipped,
            "Incremental compile complete"
        );

        Ok(IncrementalResult {
            total_scanned: total,
            compiled,
            skipped,
            results,
        })
    }

    /// Run quality analysis on the wiki directory.
    pub fn check_quality(&self) -> PdfResult<QualityReport> {
        let wiki_dir = self.wiki_dir();
        quality::analyze_wiki(&wiki_dir)
    }

    /// Build a compilation prompt for AI agent to process extracted content.
    fn build_compile_prompt(
        &self,
        entry: &KnowledgeEntry,
        extraction: &StructuredExtractionResult,
    ) -> String {
        format!(
            r#"# 知识编译任务

## 文档元数据

| 字段 | 值 |
|------|-----|
| 标题 | {} |
| 领域 | {} |
| 来源 | {} |
| 页数 | {} |
| 内容哈希 | {} |
| 编译时间 | {} |

## 编译指令

你是一个专业的**知识库管理员**。请根据以下规范处理这份 PDF 提取内容：

### 1. 概念提炼
- 深度通读提取内容，提炼 **10-15 个核心概念**（原子化技术概念，非按章节切片）
- 每个概念应该是一个独立、可复用的知识单元

### 2. 存量检查
- 检查 `wiki/` 目录中是否已存在相关词条
- 若概念已存在：将新见解**融合**到现有词条，更新 `related` 和 `updated` 字段
- 若概念不存在：创建新词条

### 3. 条目格式规范
每个条目必须使用如下 YAML front matter：

```yaml
---
title: "概念名称"
domain: "{}"
source: "raw/xxx.pdf"
page: 3
tags: ["tag1", "tag2"]
level: L1
status: compiled
quality_score: 0.85
created: 2026-05-04
updated: 2026-05-04
related: ["wiki/other/concept.md"]
---
```

### 4. 命名规范
- 文件名格式：`[领域] 概念名称.md`
- 示例：`[IT] Nginx_多进程通信架构.md`
- 领域目录：`wiki/{}/`

### 5. 完成后
- 更新 `wiki/index.md`（添加新条目到对应领域分组）
- 更新 `wiki/log.md`（记录本次编译操作）

---

# 提取内容

以下内容已保存到 `raw/{}.md`：

{}
"#,
            entry.title,
            entry.domain,
            entry.source.as_deref().unwrap_or("unknown"),
            extraction.page_count,
            entry.source_hash.as_deref().unwrap_or(""),
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            entry.domain,
            entry.domain.to_lowercase().replace(' ', "_"),
            entry.title,
            extraction.extracted_text,
        )
    }

    /// Re-read and re-analyze a single wiki entry.
    ///
    /// Reads the entry, checks if its source PDF still exists, re-extracts if needed,
    /// bumps the version counter, and marks status as `NeedsRecompile` or `Compiled`.
    /// The AI client then uses the returned context to regenerate the entry content.
    pub fn recompile_entry(
        &self,
        entry_path: &Path,
    ) -> PdfResult<RecompileResult> {
        let wiki_dir = self.wiki_dir();
        let full_path = if entry_path.is_relative() {
            wiki_dir.join(entry_path)
        } else {
            entry_path.to_path_buf()
        };

        if !full_path.exists() {
            return Err(PdfModuleError::FileNotFound(
                full_path.to_string_lossy().to_string(),
            ));
        }

        let content = fs::read_to_string(&full_path).map_err(|e| {
            PdfModuleError::StorageError(format!("Failed to read entry: {}", e))
        })?;

        let mut entry = KnowledgeEntry::from_markdown(&content).ok_or_else(|| {
            PdfModuleError::StorageError("Failed to parse front matter from entry".to_string())
        })?;

        // Check if source PDF still exists
        let source_exists = entry
            .source
            .as_ref()
            .map(|s| self.knowledge_base.join(s).exists())
            .unwrap_or(false);

        // If source exists, verify hash hasn't changed
        let mut source_changed = false;
        let mut current_source_hash = String::new();
        if let Some(ref source_rel) = entry.source {
            let source_path = self.knowledge_base.join(source_rel);
            if source_path.exists() {
                current_source_hash = HashCache::hash_file(&source_path)?;
                source_changed = entry
                    .source_hash
                    .as_ref()
                    .map(|h| h != &current_source_hash)
                    .unwrap_or(true);
            }
        }

        // Determine new status
        let new_status = if source_exists && source_changed {
            CompileStatus::NeedsRecompile
        } else if source_exists {
            CompileStatus::NeedsRecompile
        } else {
            CompileStatus::NeedsRecompile
        };

        // Bump version
        entry.touch();
        entry.status = new_status;
        if source_changed {
            entry.source_hash = Some(current_source_hash.clone());
        }

        // Write back updated front matter
        let body = content
            .split("---")
            .nth(2)
            .unwrap_or("")
            .trim_start();
        let new_content = entry.to_markdown(body)?;

        // Back up old version
        let backup_dir = self.knowledge_base.join("wiki").join(".versions");
        fs::create_dir_all(&backup_dir).map_err(|e| {
            PdfModuleError::StorageError(format!("Failed to create versions dir: {}", e))
        })?;
        let backup_name = format!(
            "{}_v{}.md",
            full_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown"),
            entry.version - 1
        );
        let backup_path = backup_dir.join(&backup_name);
        if full_path.exists() {
            fs::copy(&full_path, &backup_path).map_err(|e| {
                PdfModuleError::StorageError(format!("Failed to create backup: {}", e))
            })?;
        }

        // Write updated entry
        fs::write(&full_path, &new_content).map_err(|e| {
            PdfModuleError::StorageError(format!("Failed to write updated entry: {}", e))
        })?;

        // Build recompile prompt for AI
        let recompile_prompt = self.build_recompile_prompt(&entry, source_changed, source_exists);

        info!(
            entry = ?entry_path,
            version = entry.version,
            source_changed = source_changed,
            source_exists = source_exists,
            "Recompile entry complete"
        );

        Ok(RecompileResult {
            entry_path: full_path,
            version: entry.version,
            title: entry.title,
            domain: entry.domain,
            source_changed,
            source_exists,
            backup_path,
            recompile_prompt,
        })
    }

    /// Build a prompt for AI to recompile a single entry.
    fn build_recompile_prompt(
        &self,
        entry: &KnowledgeEntry,
        source_changed: bool,
        source_exists: bool,
    ) -> String {
        let mut instructions = String::from(
            r#"## 重编译指令

请根据以下信息重新生成这个知识条目：

### 执行步骤
1. 通读当前条目的正文内容
"#,
        );

        if source_exists {
            instructions.push_str(&format!(
                "2. 阅读源文件 `{}` 的最新提取内容\n",
                entry.source.as_deref().unwrap_or("")
            ));
            if source_changed {
                instructions.push_str(
                    "3. **注意：源 PDF 已变更**，对比新旧内容，保留有价值的历史见解，融合新内容\n",
                );
            } else {
                instructions.push_str(
                    "3. 按照最新编译规范重新提炼概念（可引用 `schema/` 中的规范）\n",
                );
            }
        } else {
            instructions.push_str("2. 源 PDF 已不存在，仅根据当前正文内容重新格式化\n");
        }

        instructions.push_str(&format!(
            r#"
4. 更新 front matter：版本号 -> `{}`，`updated` -> 当前时间，`status` -> `compiled`
5. 确保所有 `related` 链接仍然有效

### 当前条目元数据

| 字段 | 值 |
|------|-----|
| 标题 | {} |
| 领域 | {} |
| 版本 | {} |
| 源文件 | {} |
| 标签 | {} |

---
"#,
            entry.version,
            entry.title,
            entry.domain,
            entry.version,
            entry.source.as_deref().unwrap_or("无"),
            entry.tags.join(", "),
        ));

        instructions
    }

    /// Identify clusters of L1 entries that can be aggregated into L2 summaries.
    ///
    /// Uses a simple community detection approach:
    /// 1. Group entries by domain
    /// 2. Within each domain, find clusters connected by shared tags (Jaccard ≥ 0.3)
    /// 3. Return clusters with ≥ 2 members as aggregation candidates
    pub fn identify_aggregation_candidates(
        &self,
    ) -> PdfResult<Vec<AggregationCandidate>> {
        use crate::knowledge::index::GraphIndex;

        let wiki_dir = self.wiki_dir();
        let mut graph = GraphIndex::new();
        graph.rebuild(&wiki_dir)?;

        // Group entries by domain
        let mut domain_entries: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for path in graph.all_paths() {
            // We need to read domain from the graph, but it's private.
            // Instead, scan files directly.
            let full_path = wiki_dir.join(&path);
            if let Ok(content) = fs::read_to_string(&full_path) {
                if let Some(entry) = crate::knowledge::entry::KnowledgeEntry::from_markdown(&content) {
                    if entry.level == crate::knowledge::entry::EntryLevel::L1 {
                        domain_entries
                            .entry(entry.domain.clone())
                            .or_default()
                            .push(path.clone());
                    }
                }
            }
        }

        let mut candidates = Vec::new();

        for (domain, paths) in &domain_entries {
            if paths.len() < 2 {
                continue;
            }

            // Find connected components via tag co-occurrence
            let mut visited = std::collections::HashSet::new();
            for path in paths {
                if visited.contains(path) {
                    continue;
                }
                let neighbors = graph.get_neighbors(path, 1);
                let cluster_paths: Vec<String> = neighbors
                    .iter()
                    .filter(|n| paths.contains(&n.path) && n.edge_kind == "tag_cooccurrence")
                    .map(|n| n.path.clone())
                    .collect();

                if cluster_paths.len() >= 2 {
                    let mut cluster = vec![path.clone()];
                    cluster.extend(cluster_paths);
                    cluster.sort();
                    cluster.dedup();

                    // Check if this cluster overlaps with already-visited
                    let is_new = cluster.iter().any(|p| !visited.contains(p));
                    if is_new {
                        for p in &cluster {
                            visited.insert(p.clone());
                        }
                        candidates.push(AggregationCandidate {
                            domain: domain.clone(),
                            entry_paths: cluster,
                            suggested_title: format!("{} 领域综合", domain),
                        });
                    }
                }
            }
        }

        Ok(candidates)
    }

    /// Find entries that contradict each other for hypothesis testing.
    ///
    /// Returns pairs of entry paths that have explicit `contradictions` links.
    pub fn find_contradictions(&self) -> PdfResult<Vec<ContradictionPair>> {
        let wiki_dir = self.wiki_dir();
        let mut pairs = Vec::new();
        let mut seen = std::collections::HashSet::new();

        self.scan_contradictions(&wiki_dir, &wiki_dir, &mut pairs, &mut seen)?;
        Ok(pairs)
    }

    fn scan_contradictions(
        &self,
        base: &Path,
        dir: &Path,
        pairs: &mut Vec<ContradictionPair>,
        seen: &mut std::collections::HashSet<String>,
    ) -> PdfResult<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to read dir: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                PdfModuleError::StorageError(format!("Failed to read entry: {}", e))
            })?;
            let path = entry.path();
            if path.is_dir() {
                self.scan_contradictions(base, &path, pairs, seen)?;
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(entry) = crate::knowledge::entry::KnowledgeEntry::from_markdown(&content) {
                        let rel = path
                            .strip_prefix(base)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();

                        for contra in &entry.contradictions {
                            let mut pair_key = vec![rel.clone(), contra.clone()];
                            pair_key.sort();
                            let key = pair_key.join("↔");
                            if seen.insert(key) {
                                pairs.push(ContradictionPair {
                                    entry_a: rel.clone(),
                                    entry_b: contra.clone(),
                                    title_a: entry.title.clone(),
                                    title_b: String::new(), // Will be filled if we can read B
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// A cluster of entries that should be aggregated into an L2 summary.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AggregationCandidate {
    pub domain: String,
    pub entry_paths: Vec<String>,
    pub suggested_title: String,
}

/// Result of a recompile operation for a single entry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecompileResult {
    pub entry_path: PathBuf,
    pub version: u32,
    pub title: String,
    pub domain: String,
    pub source_changed: bool,
    pub source_exists: bool,
    pub backup_path: PathBuf,
    pub recompile_prompt: String,
}

/// A pair of entries that contradict each other.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContradictionPair {
    pub entry_a: String,
    pub entry_b: String,
    pub title_a: String,
    pub title_b: String,
}
