//! Quality analysis for knowledge entries.
//!
//! Detects common issues in the wiki: missing tags, orphan entries,
//! broken links, contradictions, and style drift.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PdfModuleError, PdfResult};
use crate::knowledge::entry::KnowledgeEntry;

/// Severity level for quality issues.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A single quality issue found during analysis.
#[derive(Debug, Clone)]
pub struct QualityIssue {
    pub severity: IssueSeverity,
    pub entry_path: String,
    pub message: String,
}

impl std::fmt::Display for QualityIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.entry_path, self.message)
    }
}

/// Comprehensive quality report for the wiki.
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// Total entries scanned.
    pub total_entries: usize,
    /// All issues found.
    pub issues: Vec<QualityIssue>,
    /// Entry paths that have no incoming or outgoing links.
    pub orphan_entries: Vec<String>,
    /// Entry paths referenced by `related` or `contradictions` but not found on disk.
    pub broken_links: Vec<String>,
    /// Domains found in the wiki.
    pub domains: HashSet<String>,
    /// Average quality score across entries with score > 0.
    pub avg_quality_score: f32,
}

impl QualityReport {
    /// Check if the report has any errors.
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error)
    }

    /// Check if the report has any warnings.
    pub fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Warning)
    }

    /// Format the report as Markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# Knowledge Quality Report\n\n");
        md.push_str(&format!("- **Total entries**: {}\n", self.total_entries));
        md.push_str(&format!("- **Domains**: {}\n", self.domains.len()));
        md.push_str(&format!(
            "- **Average quality score**: {:.1}%\n",
            self.avg_quality_score * 100.0
        ));
        md.push_str(&format!("- **Issues found**: {}\n", self.issues.len()));
        md.push_str(&format!(
            "- **Orphan entries**: {}\n",
            self.orphan_entries.len()
        ));
        md.push_str(&format!(
            "- **Broken links**: {}\n\n",
            self.broken_links.len()
        ));

        if !self.issues.is_empty() {
            md.push_str("## Issues\n\n");
            md.push_str("| Severity | Entry | Issue |\n");
            md.push_str("|----------|-------|-------|\n");
            for issue in &self.issues {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    issue.severity, issue.entry_path, issue.message
                ));
            }
            md.push('\n');
        }

        if !self.orphan_entries.is_empty() {
            md.push_str("## Orphan Entries\n\n");
            md.push_str("These entries have no incoming or outgoing links:\n\n");
            for path in &self.orphan_entries {
                md.push_str(&format!("- `{}`\n", path));
            }
            md.push('\n');
        }

        if !self.broken_links.is_empty() {
            md.push_str("## Broken Links\n\n");
            md.push_str("These paths are referenced but do not exist:\n\n");
            for path in &self.broken_links {
                md.push_str(&format!("- `{}`\n", path));
            }
            md.push('\n');
        }

        md
    }
}

/// Scan all wiki entries under `wiki/` and produce a quality report.
pub fn analyze_wiki(wiki_dir: &Path) -> PdfResult<QualityReport> {
    let mut issues = Vec::new();
    let mut entries: Vec<(PathBuf, KnowledgeEntry)> = Vec::new();
    let mut all_paths: HashSet<String> = HashSet::new();
    let mut domains = HashSet::new();
    let mut referenced_paths: HashSet<String> = HashSet::new();

    // Walk wiki/ recursively and parse front matter
    scan_entries_recursive(wiki_dir, wiki_dir, &mut entries, &mut all_paths)?;

    let total_entries = entries.len();

    for (path, entry) in &entries {
        let rel = path
            .strip_prefix(wiki_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Check minimal quality
        if entry.title.is_empty() {
            issues.push(QualityIssue {
                severity: IssueSeverity::Error,
                entry_path: rel.clone(),
                message: "Missing title".into(),
            });
        }
        if entry.domain.is_empty() {
            issues.push(QualityIssue {
                severity: IssueSeverity::Error,
                entry_path: rel.clone(),
                message: "Missing domain".into(),
            });
        }
        if entry.tags.is_empty() {
            issues.push(QualityIssue {
                severity: IssueSeverity::Warning,
                entry_path: rel.clone(),
                message: "No tags assigned".into(),
            });
        }
        if entry.quality_score == 0.0 {
            issues.push(QualityIssue {
                severity: IssueSeverity::Info,
                entry_path: rel.clone(),
                message: "Quality score is 0 (not yet assessed)".into(),
            });
        }

        domains.insert(entry.domain.clone());

        // Collect referenced paths for broken link detection
        for r in &entry.related {
            referenced_paths.insert(r.clone());
        }
        for c in &entry.contradictions {
            referenced_paths.insert(c.clone());
        }
    }

    // Detect broken links
    let mut broken_links = Vec::new();
    for referenced in &referenced_paths {
        // Try to resolve relative to wiki_dir
        let resolved = wiki_dir.join(referenced);
        if !resolved.exists() {
            // Also check if it's a direct path
            if !all_paths.contains(referenced) {
                broken_links.push(referenced.clone());
            }
        }
    }

    // Detect orphan entries (no related links in or out)
    let mut linked_entries: HashSet<String> = HashSet::new();
    for (_, entry) in &entries {
        let rel = entry.relative_path().to_string_lossy().to_string();
        if !entry.related.is_empty() || !entry.contradictions.is_empty() {
            linked_entries.insert(rel.clone());
        }
        for r in &entry.related {
            linked_entries.insert(r.clone());
        }
    }
    let orphan_entries: Vec<String> = entries
        .iter()
        .filter(|(_, entry)| {
            let rel = entry.relative_path().to_string_lossy().to_string();
            !linked_entries.contains(&rel)
                && entry.level != crate::knowledge::entry::EntryLevel::L0
        })
        .map(|(path, _)| {
            path.strip_prefix(wiki_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string()
        })
        .collect();

    // Average quality score
    let scored: Vec<f32> = entries
        .iter()
        .map(|(_, e)| e.quality_score)
        .filter(|&s| s > 0.0)
        .collect();
    let avg_quality_score = if scored.is_empty() {
        0.0
    } else {
        scored.iter().sum::<f32>() / scored.len() as f32
    };

    Ok(QualityReport {
        total_entries,
        issues,
        orphan_entries,
        broken_links,
        domains,
        avg_quality_score,
    })
}

fn scan_entries_recursive(
    base: &Path,
    dir: &Path,
    entries: &mut Vec<(PathBuf, KnowledgeEntry)>,
    all_paths: &mut HashSet<String>,
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
            scan_entries_recursive(base, &path, entries, all_paths)?;
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            all_paths.insert(rel);
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(entry) = KnowledgeEntry::from_markdown(&content) {
                    entries.push((path, entry));
                }
            }
        }
    }
    Ok(())
}
