// VecBase — lib.rs
// Public API, error types, plugin interface, re-exports.
// Compiled as both `cdylib` (for plugins / FFI) and `rlib` (for the binary).
// Author: d65v <https://github.com/d65v>

pub mod embedding;
pub mod processing;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::embedding::{normalize, Metric};
use crate::processing::HnswIndex;

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum VecBaseError {
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    #[error("Record not found: {id}")]
    NotFound { id: String },

    #[error("Plugin load error: {0}")]
    PluginLoadError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, VecBaseError>;

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VecBaseConfig {
    /// Dimensionality of all stored vectors
    pub dim: usize,
    /// Similarity metric
    pub metric: String,
    /// Maximum elements before eviction / overflow error
    pub max_elements: usize,
    /// Path for optional persistence
    pub storage_path: String,
}

impl Default for VecBaseConfig {
    fn default() -> Self {
        Self {
            dim: 128,
            metric: "cosine".to_string(),
            max_elements: 1_000_000,
            storage_path: "./data".to_string(),
        }
    }
}

impl VecBaseConfig {
    /// Load config from environment variables, falling back to defaults.
    pub fn from_env() -> Self {
        let dim = std::env::var("VECBASE_DIM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(128);

        let metric = std::env::var("VECBASE_METRIC").unwrap_or_else(|_| "cosine".to_string());

        let max_elements = std::env::var("VECBASE_MAX_ELEMENTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1_000_000);

        let storage_path =
            std::env::var("VECBASE_STORAGE_PATH").unwrap_or_else(|_| "./data".to_string());

        Self {
            dim,
            metric,
            max_elements,
            storage_path,
        }
    }
}

// ── Core Data Types ───────────────────────────────────────────────────────────

/// A single stored vector record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VecRecord {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<String>,
}

/// A single search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<String>,
}

// ── Main Database Struct ──────────────────────────────────────────────────────

pub struct VecBase {
    pub config: VecBaseConfig,
    records: HashMap<String, VecRecord>,
    index: HnswIndex,
    metric: Metric,
}

impl VecBase {
    /// Create a new VecBase instance with the given config.
    pub fn new(config: VecBaseConfig) -> Self {
        let metric = match config.metric.as_str() {
            "euclidean" => Metric::Euclidean,
            "dot" => Metric::DotProduct,
            _ => Metric::Cosine,
        };

        let index = HnswIndex::new(config.dim, config.max_elements);

        Self {
            config,
            records: HashMap::new(),
            index,
            metric,
        }
    }

    /// Insert a vector record.
    ///
    /// # Errors
    /// Returns `VecBaseError::DimensionMismatch` if vector length ≠ config.dim.
    pub fn insert(
        &mut self,
        id: String,
        vector: Vec<f32>,
        metadata: Option<String>,
    ) -> Result<()> {
        if vector.len() != self.config.dim {
            return Err(VecBaseError::DimensionMismatch {
                expected: self.config.dim,
                got: vector.len(),
            });
        }

        // Normalize for cosine similarity
        let stored_vec = if matches!(self.metric, Metric::Cosine) {
            normalize(&vector)
        } else {
            vector.clone()
        };

        let record = VecRecord {
            id: id.clone(),
            vector: stored_vec.clone(),
            metadata,
        };

        self.records.insert(id.clone(), record);
        self.index.insert(id, stored_vec);

        Ok(())
    }

    /// Search for the top-k nearest neighbors to the query vector.
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        if query.len() != self.config.dim {
            log::warn!(
                "search: query dim {} ≠ config dim {}",
                query.len(),
                self.config.dim
            );
            return vec![];
        }

        let q = if matches!(self.metric, Metric::Cosine) {
            normalize(query)
        } else {
            query.to_vec()
        };

        let ids = self.index.search(&q, top_k, &self.metric);

        ids.into_iter()
            .filter_map(|(id, score)| {
                self.records.get(&id).map(|rec| SearchResult {
                    id: rec.id.clone(),
                    score,
                    metadata: rec.metadata.clone(),
                })
            })
            .collect()
    }

    /// Delete a record by id.
    ///
    /// # Errors
    /// Returns `VecBaseError::NotFound` if the id does not exist.
    pub fn delete(&mut self, id: &str) -> Result<()> {
        if self.records.remove(id).is_none() {
            return Err(VecBaseError::NotFound { id: id.to_string() });
        }
        self.index.remove(id);
        Ok(())
    }

    /// Return the total number of stored vectors.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Retrieve a record by id.
    pub fn get(&self, id: &str) -> Option<&VecRecord> {
        self.records.get(id)
    }
}

// ── Plugin Interface (cdylib) ─────────────────────────────────────────────────

/// Trait that all VecBase plugins must implement.
/// Plugins are compiled as separate `cdylib` crates and loaded at runtime.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    /// Called once when the plugin is loaded.
    fn on_init(&self);
    /// Called on every insert — can transform or enrich the vector/metadata.
    fn on_insert(&self, id: &str, vector: &mut Vec<f32>, metadata: &mut Option<String>);
    /// Called on every search result — can rerank or filter.
    fn on_search_results(&self, results: &mut Vec<SearchResult>);
}

/// FFI entry point every plugin `.so` must export.
///
/// ```c
/// Plugin* vecbase_plugin_init(void);
/// ```
#[no_mangle]
pub extern "C" fn vecbase_plugin_version() -> *const std::ffi::c_char {
    // Safety: static string, NUL-terminated
    b"0.1.0\0".as_ptr() as *const std::ffi::c_char
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> VecBase {
        VecBase::new(VecBaseConfig {
            dim: 4,
            ..Default::default()
        })
    }

    #[test]
    fn test_insert_and_len() {
        let mut db = make_db();
        db.insert("a".into(), vec![0.1, 0.2, 0.3, 0.4], None)
            .unwrap();
        assert_eq!(db.len(), 1);
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut db = make_db();
        let err = db.insert("x".into(), vec![1.0, 2.0], None).unwrap_err();
        assert!(matches!(err, VecBaseError::DimensionMismatch { .. }));
    }

    #[test]
    fn test_search_returns_results() {
        let mut db = make_db();
        db.insert("a".into(), vec![1.0, 0.0, 0.0, 0.0], None)
            .unwrap();
        db.insert("b".into(), vec![0.0, 1.0, 0.0, 0.0], None)
            .unwrap();
        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 2);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "a");
    }

    #[test]
    fn test_delete() {
        let mut db = make_db();
        db.insert("del".into(), vec![0.5, 0.5, 0.5, 0.5], None)
            .unwrap();
        db.delete("del").unwrap();
        assert!(db.get("del").is_none());
    }

    #[test]
    fn test_delete_not_found() {
        let mut db = make_db();
        let err = db.delete("ghost").unwrap_err();
        assert!(matches!(err, VecBaseError::NotFound { .. }));
    }

    #[test]
    fn test_config_from_default() {
        let cfg = VecBaseConfig::default();
        assert_eq!(cfg.dim, 128);
        assert_eq!(cfg.metric, "cosine");
    }
}
