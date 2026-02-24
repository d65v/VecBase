# VecBase — vcore Documentation

`vcore` is the Rust crate at the heart of VecBase. It handles:

- Vector storage (in-memory, with optional disk persistence)
- Similarity computation (cosine, euclidean, dot product)
- Embedding input/output
- ANN search via HNSW
- Plugin loading via `cdylib`

---

## Module Overview

| File             | Role                                              |
|-----------------|--------------------------------------------------|
| `main.rs`       | Binary entry point, CLI/server startup            |
| `lib.rs`        | Public API surface, re-exports, plugin interface  |
| `embedding.rs`  | Embedding normalization, format parsing           |
| `processing.rs` | Batch insert, query processing, index management  |
| `algorithm/`    | ANN algorithm implementations (HNSW, brute-force) |
| `plug-ins/`     | Dynamic plugin system                             |

---

## Data Model

```
VecRecord {
    id:       String,
    vector:   Vec<f32>,
    metadata: Option<String>,
}
```

---

## Search Flow

```
query vector
    → normalize (if cosine)
    → HNSW graph traversal (or brute-force if small dataset)
    → score & rank
    → return top-k results
```

---

## Plugin Interface

Plugins are compiled as `cdylib` and expose:

```rust
#[no_mangle]
pub extern "C" fn vecbase_plugin_init() -> *mut dyn Plugin;
```

See `plug-ins/plugins.md` for full spec.

---

## Error Types

All errors are defined in `lib.rs` using `thiserror`:

- `VecBaseError::DimensionMismatch`
- `VecBaseError::NotFound`
- `VecBaseError::PluginLoadError`
- `VecBaseError::StorageError`
