// VecBase — plug-ins/example_plugin/src/lib.rs
// A concrete example plugin compiled as a `cdylib`.
// This plugin does two things:
//   1. on_insert  → clamps all vector components to [-1.0, 1.0]
//   2. on_search  → filters out results with score < threshold
//
// Build:
//   cargo build --release
//   cp target/release/libexample_plugin.so ../
//
// Cargo.toml for this crate:
//   [lib]
//   crate-type = ["cdylib"]
//   [dependencies]
//   vcore = { path = "../../.." }
//
// Author: d65v <https://github.com/d65v>

// NOTE: In a real build this would `use vcore::{Plugin, SearchResult}`.
// Here we inline the trait to keep this file self-contained and readable
// without requiring a workspace build. When you compile for real, replace
// the inline definitions with `use vcore::{Plugin, SearchResult};`.

// ── Inline types (mirrors vcore/src/lib.rs) ───────────────────────────────────

pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<String>,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn on_init(&self);
    fn on_insert(&self, id: &str, vector: &mut Vec<f32>, metadata: &mut Option<String>);
    fn on_search_results(&self, results: &mut Vec<SearchResult>);
}

// ── Plugin Config ─────────────────────────────────────────────────────────────

/// Minimum score threshold — results below this are dropped.
/// Set via VECBASE_PLUGIN_MIN_SCORE env var at load time.
struct ExamplePlugin {
    min_score: f32,
}

impl ExamplePlugin {
    fn from_env() -> Self {
        let min_score = std::env::var("VECBASE_PLUGIN_MIN_SCORE")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.0); // default: no filtering
        Self { min_score }
    }
}

// ── Plugin Implementation ─────────────────────────────────────────────────────

impl Plugin for ExamplePlugin {
    fn name(&self) -> &'static str {
        "example_plugin"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn on_init(&self) {
        eprintln!(
            "[example_plugin] loaded — min_score={:.3}",
            self.min_score
        );
    }

    /// Clamp each component to [-1.0, 1.0] before storage.
    /// Useful as a safety guard against out-of-range embeddings.
    fn on_insert(&self, id: &str, vector: &mut Vec<f32>, _metadata: &mut Option<String>) {
        let mut clamped = 0usize;
        for x in vector.iter_mut() {
            let before = *x;
            *x = x.clamp(-1.0, 1.0);
            if (*x - before).abs() > f32::EPSILON {
                clamped += 1;
            }
        }
        if clamped > 0 {
            eprintln!(
                "[example_plugin] insert '{}': clamped {} components",
                id, clamped
            );
        }
    }

    /// Drop results whose score falls below `min_score`.
    /// Results arrive pre-sorted descending by score.
    fn on_search_results(&self, results: &mut Vec<SearchResult>) {
        let before = results.len();
        results.retain(|r| r.score >= self.min_score);
        let dropped = before - results.len();
        if dropped > 0 {
            eprintln!(
                "[example_plugin] filtered {} low-score results (threshold={:.3})",
                dropped, self.min_score
            );
        }
    }
}

// ── FFI Entry Point ───────────────────────────────────────────────────────────

/// VecBase calls this symbol when loading the plugin via dlopen.
/// Returns a heap-allocated Plugin trait object.
///
/// # Safety
/// The returned pointer must be freed by VecBase via `Box::from_raw`.
#[no_mangle]
pub unsafe extern "C" fn vecbase_plugin_init() -> *mut dyn Plugin {
    let plugin = ExamplePlugin::from_env();
    plugin.on_init();
    Box::into_raw(Box::new(plugin))
}

/// Called by VecBase before unloading the plugin.
///
/// # Safety
/// `ptr` must have been returned by `vecbase_plugin_init`.
#[no_mangle]
pub unsafe extern "C" fn vecbase_plugin_destroy(ptr: *mut dyn Plugin) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_plugin() -> ExamplePlugin {
        ExamplePlugin { min_score: 0.5 }
    }

    #[test]
    fn test_clamp_on_insert() {
        let p = make_plugin();
        let mut v = vec![2.0f32, -3.0, 0.5, -0.5];
        let mut meta = None;
        p.on_insert("test", &mut v, &mut meta);
        assert!((v[0] - 1.0).abs() < f32::EPSILON, "2.0 → 1.0");
        assert!((v[1] - -1.0).abs() < f32::EPSILON, "-3.0 → -1.0");
        assert!((v[2] - 0.5).abs() < f32::EPSILON, "0.5 unchanged");
    }

    #[test]
    fn test_filter_low_scores() {
        let p = make_plugin();
        let mut results = vec![
            SearchResult { id: "a".into(), score: 0.9, metadata: None },
            SearchResult { id: "b".into(), score: 0.4, metadata: None }, // below threshold
            SearchResult { id: "c".into(), score: 0.6, metadata: None },
        ];
        p.on_search_results(&mut results);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.score >= 0.5));
    }

    #[test]
    fn test_no_filter_when_threshold_zero() {
        let p = ExamplePlugin { min_score: 0.0 };
        let mut results = vec![
            SearchResult { id: "x".into(), score: 0.01, metadata: None },
        ];
        p.on_search_results(&mut results);
        assert_eq!(results.len(), 1);
    }
}
