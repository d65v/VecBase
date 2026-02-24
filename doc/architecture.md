# VecBase — Architecture

```
┌─────────────────────────────────────────────────────┐
│                     Client                          │
│         (Rust API / CLI / future HTTP)              │
└───────────────────────┬─────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│                   VecBase Core                      │
│                  (vcore/src/lib.rs)                 │
│                                                     │
│  ┌──────────────┐  ┌──────────────┐                │
│  │ embedding.rs │  │processing.rs │                │
│  │ normalize    │  │ HnswIndex    │                │
│  │ similarity   │  │ batch ops    │                │
│  └──────────────┘  └──────────────┘                │
│                                                     │
│  ┌──────────────────────────────────┐               │
│  │        Plugin System             │               │
│  │  cdylib .so loaded at runtime    │               │
│  └──────────────────────────────────┘               │
└───────────────────────┬─────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│                 Storage Layer                       │
│          (in-memory HashMap + HNSW graph)           │
│          (optional: bincode disk persistence)       │
└─────────────────────────────────────────────────────┘
```

## Data Flow

**Insert**:
```
client.insert(id, vector, meta)
  → dimension check
  → normalize (if cosine)
  → HashMap<id, VecRecord>
  → HnswIndex.insert(id, vector)
```

**Search**:
```
client.search(query, top_k)
  → normalize query
  → HnswIndex.search(query, top_k)
    → brute-force (N ≤ 500) OR graph traversal
  → resolve ids → VecRecords → SearchResults
```

## Design Principles

1. **Minimal** — no unnecessary dependencies
2. **Fast** — zero-copy where possible, optimized similarity kernels
3. **Correct** — every public function has unit tests
4. **Extensible** — plugin system for custom hooks without modifying core
