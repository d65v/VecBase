# VecBase — Roadmap

## v0.1.0 — Foundation (current)
- [x] In-memory vector store
- [x] Cosine, Euclidean, Dot-product metrics
- [x] HNSW-lite ANN index
- [x] cdylib plugin interface
- [x] Docker support
- [x] Batch insert API
- [x] CLI entry point

## v0.2.0 — Persistence
- [ ] Disk persistence (bincode snapshots)
- [ ] WAL for crash recovery
- [ ] Index save/load (`.vbi` format)

## v0.3.0 — Interface
- [ ] HTTP REST API (axum or actix)
- [ ] gRPC (tonic)
- [ ] Python bindings (PyO3)

## v0.4.0 — Scale
- [ ] Sharding / partitioned index
- [ ] Product Quantization (memory compression)
- [ ] Streaming insert (channel-based)

## Future
- [ ] Distributed mode
- [ ] GPU similarity (CUDA via cudarc)
- [ ] WebAssembly target
