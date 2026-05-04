//! petgraph-based link graph for knowledge entries.
//!
//! Builds a directed graph from the `related` and `contradictions` fields
//! in entry front matter. Supports:
//! - N-hop neighbor discovery
//! - Orphan detection (no incoming or outgoing edges)
//! - Link suggestion (Jaccard similarity on tags)
//! - Concept map export (Mermaid.js format)

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PdfModuleError, PdfResult};
use crate::knowledge::entry::KnowledgeEntry;

/// Metadata stored at each graph node.
#[derive(Debug, Clone)]
pub struct NodeMeta {
    pub path: String,
    pub title: String,
    pub domain: String,
    pub tags: Vec<String>,
    pub level: String,
}

/// Edge type in the knowledge graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    Related,
    Contradiction,
    TagCooccurrence,
}

/// Result of a neighbor query.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeighborInfo {
    pub path: String,
    pub title: String,
    pub domain: String,
    pub hops: u32,
    pub edge_kind: String,
}

/// Result of a link suggestion.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkSuggestion {
    pub from: String,
    pub to: String,
    pub score: f64,
    pub reason: String,
}

/// Directed graph index over knowledge entries.
pub struct GraphIndex {
    graph: DiGraph<NodeMeta, EdgeKind>,
    /// Maps path → node index for fast lookup.
    path_to_node: HashMap<String, NodeIndex>,
}

impl GraphIndex {
    /// Create an empty graph index.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            path_to_node: HashMap::new(),
        }
    }

    /// Rebuild the graph by scanning all wiki entries.
    pub fn rebuild(&mut self, wiki_dir: &Path) -> PdfResult<usize> {
        self.graph.clear();
        self.path_to_node.clear();

        let mut entries = Vec::new();
        Self::scan_entries(wiki_dir, wiki_dir, &mut entries)?;

        // Add all nodes first
        for (path, entry) in &entries {
            let rel = path
                .strip_prefix(wiki_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let meta = NodeMeta {
                path: rel.clone(),
                title: entry.title.clone(),
                domain: entry.domain.clone(),
                tags: entry.tags.clone(),
                level: format!("{}", entry.level),
            };
            let idx = self.graph.add_node(meta);
            self.path_to_node.insert(rel, idx);
        }

        // Add edges from related and contradictions
        for (path, entry) in &entries {
            let rel = path
                .strip_prefix(wiki_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let from_idx = match self.path_to_node.get(&rel) {
                Some(idx) => *idx,
                None => continue,
            };

            for related in &entry.related {
                if let Some(&to_idx) = self.path_to_node.get(related) {
                    self.graph.add_edge(from_idx, to_idx, EdgeKind::Related);
                }
            }

            for contra in &entry.contradictions {
                if let Some(&to_idx) = self.path_to_node.get(contra) {
                    self.graph.add_edge(from_idx, to_idx, EdgeKind::Contradiction);
                }
            }
        }

        // Add tag co-occurrence edges (weak relations)
        let node_indices: Vec<NodeIndex> = self.graph.node_indices().collect();
        for i in 0..node_indices.len() {
            for j in (i + 1)..node_indices.len() {
                let ni = node_indices[i];
                let nj = node_indices[j];
                let tags_a: HashSet<&str> = self.graph[ni].tags.iter().map(|s| s.as_str()).collect();
                let tags_b: HashSet<&str> = self.graph[nj].tags.iter().map(|s| s.as_str()).collect();
                let intersection = tags_a.intersection(&tags_b).count();
                let union = tags_a.union(&tags_b).count();
                if union > 0 {
                    let jaccard = intersection as f64 / union as f64;
                    if jaccard >= 0.3 {
                        self.graph.add_edge(ni, nj, EdgeKind::TagCooccurrence);
                        self.graph.add_edge(nj, ni, EdgeKind::TagCooccurrence);
                    }
                }
            }
        }

        let count = self.graph.node_count();
        Ok(count)
    }

    /// Get N-hop neighbors of an entry.
    pub fn get_neighbors(&self, path: &str, max_hops: u32) -> Vec<NeighborInfo> {
        let start = match self.path_to_node.get(path) {
            Some(idx) => *idx,
            None => return Vec::new(),
        };

        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, 0u32));
        visited.insert(start);

        while let Some((node, hops)) = queue.pop_front() {
            if hops >= max_hops {
                continue;
            }

            for edge in self.graph.edges(node) {
                let target = edge.target();
                if visited.insert(target) {
                    let meta = &self.graph[target];
                    let edge_kind = match edge.weight() {
                        EdgeKind::Related => "related",
                        EdgeKind::Contradiction => "contradiction",
                        EdgeKind::TagCooccurrence => "tag_cooccurrence",
                    };
                    result.push(NeighborInfo {
                        path: meta.path.clone(),
                        title: meta.title.clone(),
                        domain: meta.domain.clone(),
                        hops: hops + 1,
                        edge_kind: edge_kind.to_string(),
                    });
                    queue.push_back((target, hops + 1));
                }
            }
        }

        result
    }

    /// Find orphan entries (no incoming or outgoing edges of type Related or Contradiction).
    pub fn find_orphans(&self) -> Vec<String> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                let has_relation = self.graph.edges(idx).any(|e| {
                    matches!(e.weight(), EdgeKind::Related | EdgeKind::Contradiction)
                });
                let has_incoming = self.graph.edges_directed(idx, petgraph::Direction::Incoming)
                    .any(|e| matches!(e.weight(), EdgeKind::Related | EdgeKind::Contradiction));
                !has_relation && !has_incoming
            })
            .map(|idx| self.graph[idx].path.clone())
            .collect()
    }

    /// Suggest potential links based on tag similarity.
    pub fn suggest_links(&self, path: &str, top_k: usize) -> Vec<LinkSuggestion> {
        let start = match self.path_to_node.get(path) {
            Some(idx) => *idx,
            None => return Vec::new(),
        };

        let tags_a: HashSet<&str> = self.graph[start].tags.iter().map(|s| s.as_str()).collect();
        if tags_a.is_empty() {
            return Vec::new();
        }

        // Find existing direct connections
        let existing: HashSet<NodeIndex> = self.graph.edges(start)
            .map(|e| e.target())
            .collect();

        let mut suggestions: Vec<LinkSuggestion> = self.graph
            .node_indices()
            .filter(|&idx| idx != start && !existing.contains(&idx))
            .filter_map(|idx| {
                let tags_b: HashSet<&str> = self.graph[idx].tags.iter().map(|s| s.as_str()).collect();
                let intersection = tags_a.intersection(&tags_b).count();
                let union = tags_a.union(&tags_b).count();
                if union == 0 || intersection == 0 {
                    return None;
                }
                let jaccard = intersection as f64 / union as f64;
                if jaccard < 0.1 {
                    return None;
                }
                let shared: Vec<String> = tags_a.intersection(&tags_b)
                    .map(|s| s.to_string())
                    .collect();
                Some(LinkSuggestion {
                    from: self.graph[start].path.clone(),
                    to: self.graph[idx].path.clone(),
                    score: jaccard,
                    reason: format!("Shared tags: {}", shared.join(", ")),
                })
            })
            .collect();

        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(top_k);
        suggestions
    }

    /// Export a local concept map around a given entry as Mermaid.js text.
    pub fn export_concept_map(&self, center_path: &str, depth: u32) -> String {
        let start = match self.path_to_node.get(center_path) {
            Some(idx) => *idx,
            None => {
                return format!("graph LR\n    error[\"Entry not found: {}\"]\n", center_path);
            }
        };

        // Collect all nodes within `depth` hops
        let mut nodes = HashSet::new();
        let mut edges = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, 0u32));
        nodes.insert(start);

        while let Some((node, hops)) = queue.pop_front() {
            if hops >= depth {
                continue;
            }
            for edge in self.graph.edges(node) {
                let target = edge.target();
                let is_new = nodes.insert(target);
                let kind = match edge.weight() {
                    EdgeKind::Related => "relates",
                    EdgeKind::Contradiction => "contradicts",
                    EdgeKind::TagCooccurrence => "co-tags",
                };
                edges.push((node, target, kind));
                if is_new {
                    queue.push_back((target, hops + 1));
                }
            }
            // Also check incoming edges
            for edge in self.graph.edges_directed(node, petgraph::Direction::Incoming) {
                let source = edge.source();
                let is_new = nodes.insert(source);
                let kind = match edge.weight() {
                    EdgeKind::Related => "relates",
                    EdgeKind::Contradiction => "contradicts",
                    EdgeKind::TagCooccurrence => "co-tags",
                };
                edges.push((source, node, kind));
                if is_new {
                    queue.push_back((source, hops + 1));
                }
            }
        }

        // Build Mermaid diagram
        let mut mermaid = String::from("graph LR\n");
        let mut node_ids = HashMap::new();
        let mut counter = 0u32;

        for idx in &nodes {
            let meta = &self.graph[*idx];
            let safe_id = format!("n{}", counter);
            counter += 1;
            let label = meta.title.replace('"', "'");
            let style = if *idx == start {
                format!("    {}[\"{}\"]:::center\n", safe_id, label)
            } else {
                format!("    {}[\"{}\"]\n", safe_id, label)
            };
            mermaid.push_str(&style);
            node_ids.insert(*idx, safe_id);
        }

        // Deduplicate edges
        let mut seen_edges = HashSet::new();
        for (from, to, kind) in &edges {
            let from_id = node_ids.get(from).unwrap();
            let to_id = node_ids.get(to).unwrap();
            let key = (from_id.clone(), to_id.clone(), *kind);
            if seen_edges.insert(key) {
                let arrow = match *kind {
                    "contradicts" => " --x ",
                    "co-tags" => " -.-> ",
                    _ => " --> ",
                };
                mermaid.push_str(&format!(
                    "    {} {}|{}| {}\n",
                    from_id, arrow, kind, to_id
                ));
            }
        }

        mermaid.push_str("    classDef center fill:#f96,stroke:#333,stroke-width:2px\n");
        mermaid
    }

    /// Get all entry paths in the graph.
    pub fn all_paths(&self) -> Vec<String> {
        self.path_to_node.keys().cloned().collect()
    }

    /// Get node count.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get edge count.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    fn scan_entries(
        base: &Path,
        dir: &Path,
        entries: &mut Vec<(PathBuf, KnowledgeEntry)>,
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
                Self::scan_entries(base, &path, entries)?;
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if filename == "index.md" || filename == "log.md" {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(entry) = KnowledgeEntry::from_markdown(&content) {
                        entries.push((path, entry));
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for GraphIndex {
    fn default() -> Self {
        Self::new()
    }
}
