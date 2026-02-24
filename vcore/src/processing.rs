// VecBase — processing.rs
// In-memory HNSW-lite index, batch insert/query processing.
// Author: d65v <https://github.com/d65v>
//
// NOTE: This is a simplified HNSW-inspired structure. A production HNSW
// implementation requires skip-list layers, entry-point management, and more
// careful neighbor pruning. This provides the skeleton with correct API and
// brute-force fallback for correctness.

use std::collections::HashMap;

use crate::embedding::{score, Metric};

// ── HNSW Node ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Node {
    id: String,
    vector: Vec<f32>,
    /// Neighbor lists per layer (layer 0 = densest)
    neighbors: Vec<Vec<String>>,
}

// ── HNSW Index ────────────────────────────────────────────────────────────────

/// A lightweight HNSW-inspired approximate nearest neighbor index.
/// Falls back to brute-force when the dataset is small (< BRUTE_THRESHOLD).
pub struct HnswIndex {
    dim: usize,
    max_elements: usize,
    nodes: HashMap<String, Node>,
    /// Maximum neighbors per node per layer
    m: usize,
    /// Entry point (id of the top-layer node)
    entry: Option<String>,
}

const BRUTE_THRESHOLD: usize = 500;

impl HnswIndex {
    pub fn new(dim: usize, max_elements: usize) -> Self {
        Self {
            dim,
            max_elements,
            nodes: HashMap::new(),
            m: 16,
            entry: None,
        }
    }

    /// Insert a new vector into the index.
    pub fn insert(&mut self, id: String, vector: Vec<f32>) {
        debug_assert_eq!(
            vector.len(),
            self.dim,
            "insert: vector dim {} ≠ index dim {}",
            vector.len(),
            self.dim
        );

        if self.nodes.len() >= self.max_elements {
            log::warn!("HnswIndex: max_elements ({}) reached, skipping insert for '{}'",
                self.max_elements, id);
            return;
        }

        let node = Node {
            id: id.clone(),
            vector,
            neighbors: vec![Vec::new()], // layer 0 only for now
        };

        // If we have existing nodes, wire up nearest neighbors
        if !self.nodes.is_empty() {
            let nearest = self.brute_search(&node.vector, self.m, &Metric::Cosine);
            let mut n = node.clone();
            n.neighbors[0] = nearest.iter().map(|(nid, _)| nid.clone()).collect();
            self.nodes.insert(id.clone(), n);

            // Back-link: add this node to its neighbors' neighbor lists
            for (nid, _) in &nearest {
                if let Some(neighbor) = self.nodes.get_mut(nid) {
                    if neighbor.neighbors[0].len() < self.m {
                        neighbor.neighbors[0].push(id.clone());
                    }
                }
            }
        } else {
            self.nodes.insert(id.clone(), node);
        }

        if self.entry.is_none() {
            self.entry = Some(id);
        }
    }

    /// Remove a node from the index.
    pub fn remove(&mut self, id: &str) {
        self.nodes.remove(id);
        // Remove back-references
        for node in self.nodes.values_mut() {
            node.neighbors[0].retain(|nid| nid != id);
        }
        // Update entry point if needed
        if self.entry.as_deref() == Some(id) {
            self.entry = self.nodes.keys().next().cloned();
        }
    }

    /// Search for top-k nearest neighbors.
    /// Uses brute-force for small datasets, graph traversal for larger ones.
    pub fn search(&self, query: &[f32], top_k: usize, metric: &Metric) -> Vec<(String, f32)> {
        if self.nodes.is_empty() {
            return vec![];
        }

        if self.nodes.len() <= BRUTE_THRESHOLD {
            return self.brute_search(query, top_k, metric);
        }

        self.graph_search(query, top_k, metric)
    }

    // ── Private: Brute-Force Search ───────────────────────────────────────────

    fn brute_search(&self, query: &[f32], top_k: usize, metric: &Metric) -> Vec<(String, f32)> {
        let mut scored: Vec<(String, f32)> = self
            .nodes
            .values()
            .map(|node| {
                let s = score(metric, query, &node.vector);
                (node.id.clone(), s)
            })
            .collect();

        // Sort descending by score (higher = better)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    // ── Private: Graph-Based Search (HNSW-lite) ───────────────────────────────

    fn graph_search(&self, query: &[f32], top_k: usize, metric: &Metric) -> Vec<(String, f32)> {
        let entry_id = match &self.entry {
            Some(e) => e.clone(),
            None => return vec![],
        };

        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        // candidate queue: (score, id) — max-heap by score
        let mut candidates: Vec<(ordered_float::OrderedFloat<f32>, String)> = Vec::new();
        let mut results: Vec<(String, f32)> = Vec::new();

        // Seed with entry point
        if let Some(entry_node) = self.nodes.get(&entry_id) {
            let s = score(metric, query, &entry_node.vector);
            candidates.push((ordered_float::OrderedFloat(s), entry_id.clone()));
            visited.insert(entry_id.clone());
        }

        let ef = top_k * 4; // exploration factor

        while !candidates.is_empty() && results.len() < ef {
            // Pick best candidate
            candidates.sort_by(|a, b| b.0.cmp(&a.0));
            let (cur_score, cur_id) = candidates.remove(0);
            results.push((cur_id.clone(), cur_score.into_inner()));

            // Explore neighbors
            if let Some(node) = self.nodes.get(&cur_id) {
                for nid in &node.neighbors[0] {
                    if visited.contains(nid) {
                        continue;
                    }
                    visited.insert(nid.clone());
                    if let Some(n) = self.nodes.get(nid) {
                        let s = score(metric, query, &n.vector);
                        candidates.push((ordered_float::OrderedFloat(s), nid.clone()));
                    }
                }
            }
        }

        // Final sort and truncate
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    /// Number of indexed vectors.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

// ── Batch Processing ──────────────────────────────────────────────────────────

/// Batch insert descriptor.
pub struct BatchInsert {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<String>,
}

/// Result of a batch operation.
#[derive(Debug)]
pub struct BatchResult {
    pub inserted: usize,
    pub failed: Vec<(String, String)>, // (id, reason)
}

/// Process a batch of inserts against a VecBase instance.
/// Returns how many succeeded and which failed with reasons.
pub fn batch_insert(db: &mut crate::VecBase, items: Vec<BatchInsert>) -> BatchResult {
    let mut inserted = 0usize;
    let mut failed = Vec::new();

    for item in items {
        match db.insert(item.id.clone(), item.vector, item.metadata) {
            Ok(()) => inserted += 1,
            Err(e) => failed.push((item.id, e.to_string())),
        }
    }

    BatchResult { inserted, failed }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_insert_search() {
        let mut idx = HnswIndex::new(3, 1000);
        idx.insert("a".into(), vec![1.0, 0.0, 0.0]);
        idx.insert("b".into(), vec![0.0, 1.0, 0.0]);
        idx.insert("c".into(), vec![0.0, 0.0, 1.0]);

        let results = idx.search(&[1.0, 0.0, 0.0], 2, &Metric::Cosine);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "a");
    }

    #[test]
    fn test_hnsw_remove() {
        let mut idx = HnswIndex::new(2, 100);
        idx.insert("x".into(), vec![1.0, 0.0]);
        idx.remove("x");
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_hnsw_empty_search() {
        let idx = HnswIndex::new(4, 100);
        let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 5, &Metric::Cosine);
        assert!(results.is_empty());
    }

    #[test]
    fn test_batch_insert() {
        use crate::{VecBase, VecBaseConfig};

        let mut db = VecBase::new(VecBaseConfig { dim: 3, ..Default::default() });
        let items = vec![
            BatchInsert { id: "v1".into(), vector: vec![1.0, 0.0, 0.0], metadata: None },
            BatchInsert { id: "v2".into(), vector: vec![0.0, 1.0, 0.0], metadata: None },
            // Wrong dimension — should fail
            BatchInsert { id: "v3".into(), vector: vec![1.0, 2.0], metadata: None },
        ];
        let result = batch_insert(&mut db, items);
        assert_eq!(result.inserted, 2);
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].0, "v3");
    }
}
