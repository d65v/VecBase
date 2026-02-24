# VecBase — ANN Algorithm Documentation

## Approximate Nearest Neighbor (ANN) Search

Exact nearest-neighbor search scales as O(N·D) — fine for thousands of vectors, painful for millions. ANN trades a small accuracy loss for dramatic speed gains.

---

## Implemented: HNSW-lite

VecBase uses a simplified **Hierarchical Navigable Small World (HNSW)** graph.

### How HNSW Works

1. **Graph construction**: When a vector is inserted, it is connected to its `M` nearest neighbors in the existing graph (layer 0). With probability `1/ln(M)` it is also inserted into higher layers for long-range navigation.
2. **Search**: Starting from the entry point (top layer), greedily descend to closer nodes layer by layer until reaching layer 0, then do a beam search with `ef` candidates.
3. **Result**: Top-k by similarity score, much faster than brute-force for large N.

### Complexity

| Operation | Time         |
|-----------|-------------|
| Insert    | O(M · log N) |
| Search    | O(log N)     |
| Memory    | O(N · M)     |

### Parameters

| Parameter | Default | Description                        |
|----------|---------|------------------------------------|
| `M`      | 16      | Max neighbors per node             |
| `ef`     | top_k×4 | Exploration factor during search   |

---

## Brute-Force Fallback

For datasets with ≤ 500 vectors, VecBase automatically uses brute-force exact search (O(N·D)) — it's faster in practice because HNSW overhead dominates at small N.

---

## Metrics

| Metric     | Formula                         | Best For               |
|-----------|----------------------------------|------------------------|
| Cosine    | dot(a,b) / (|a|·|b|)            | Text embeddings        |
| Euclidean | √Σ(aᵢ−bᵢ)²                     | Image/spatial vectors  |
| Dot       | Σ(aᵢ·bᵢ)                        | Recommendation models  |

---

## Future Algorithms

- [ ] Product Quantization (PQ) for memory compression
- [ ] IVF (Inverted File Index) for billion-scale
- [ ] LSH (Locality Sensitive Hashing) as an alternative ANN strategy
- [ ] FAISS integration via FFI
