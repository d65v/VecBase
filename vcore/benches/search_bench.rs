// VecBase — search_bench.rs
// Criterion benchmark suite for vector search performance.
// Author: d65v <https://github.com/d65v>
//
// Run with:
//   cargo bench --bench search_bench
//   cargo bench --bench search_bench -- --output-format bencher
//
// Benchmarks:
//   - brute_force_search    (N=100,  D=128)
//   - hnsw_search_small     (N=500,  D=128)
//   - hnsw_search_medium    (N=5000, D=128)
//   - hnsw_search_large     (N=50000,D=128)
//   - insert_single         (one insert, D=128)
//   - insert_batch_1k       (1000 inserts, D=128)
//   - cosine_similarity_raw (raw math, no DB)
//   - normalize_raw         (raw normalization, D=128)
//   - search_by_dim/32      (fixed N=1000, varying D)
//   - search_by_dim/128
//   - search_by_dim/512
//   - search_by_dim/1536    (OpenAI ada-002 dim)
//   - search_topk/1
//   - search_topk/10
//   - search_topk/100

use criterion::{
    black_box, criterion_group, criterion_main,
    BenchmarkId, Criterion, Throughput,
};

use vcore::{VecBase, VecBaseConfig};
use vcore::embedding::{cosine_similarity, normalize};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Generate a pseudo-random f32 vector of length `dim`.
/// Uses a simple LCG so there are no rand dependencies here —
/// criterion benches should be self-contained.
fn gen_vec(seed: u64, dim: usize) -> Vec<f32> {
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (0..dim)
        .map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            // Map to [-1.0, 1.0]
            ((state >> 33) as f32) / (u32::MAX as f32) * 2.0 - 1.0
        })
        .collect()
}

/// Build a VecBase filled with `n` random vectors of dimension `dim`.
fn build_db(n: usize, dim: usize, metric: &str) -> VecBase {
    let mut db = VecBase::new(VecBaseConfig {
        dim,
        metric: metric.to_string(),
        max_elements: n + 64,
        ..VecBaseConfig::default()
    });
    for i in 0..n {
        let v = gen_vec(i as u64, dim);
        db.insert(format!("v{}", i), v, None).unwrap();
    }
    db
}

// ── Search by Dataset Size ────────────────────────────────────────────────────

fn bench_search_by_size(c: &mut Criterion) {
    const DIM: usize = 128;
    const TOP_K: usize = 10;

    let mut group = c.benchmark_group("search_by_size");

    for &n in &[100usize, 500, 5_000, 50_000] {
        let db = build_db(n, DIM, "cosine");
        let query = gen_vec(99999, DIM);

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(db.search(black_box(&query), TOP_K))
            });
        });
    }

    group.finish();
}

// ── Search by Vector Dimension ────────────────────────────────────────────────

fn bench_search_by_dim(c: &mut Criterion) {
    const N: usize = 1_000;
    const TOP_K: usize = 10;

    let mut group = c.benchmark_group("search_by_dim");

    for &dim in &[32usize, 128, 512, 1536] {
        let db = build_db(N, dim, "cosine");
        let query = gen_vec(42, dim);

        group.throughput(Throughput::Elements(dim as u64));
        group.bench_with_input(BenchmarkId::from_parameter(dim), &dim, |b, _| {
            b.iter(|| {
                black_box(db.search(black_box(&query), TOP_K))
            });
        });
    }

    group.finish();
}

// ── Search by Top-K ───────────────────────────────────────────────────────────

fn bench_search_by_topk(c: &mut Criterion) {
    const DIM: usize = 128;
    const N: usize = 10_000;

    let db = build_db(N, DIM, "cosine");
    let query = gen_vec(777, DIM);

    let mut group = c.benchmark_group("search_topk");

    for &k in &[1usize, 10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(k), &k, |b, &k| {
            b.iter(|| {
                black_box(db.search(black_box(&query), k))
            });
        });
    }

    group.finish();
}

// ── Insert Benchmarks ─────────────────────────────────────────────────────────

fn bench_insert_single(c: &mut Criterion) {
    const DIM: usize = 128;

    c.bench_function("insert_single", |b| {
        b.iter_batched(
            || {
                // Setup: fresh DB each iteration batch
                let db = VecBase::new(VecBaseConfig {
                    dim: DIM,
                    max_elements: 2,
                    ..VecBaseConfig::default()
                });
                let v = gen_vec(1234, DIM);
                (db, v)
            },
            |(mut db, v)| {
                black_box(db.insert("x".to_string(), v, None).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_insert_batch_1k(c: &mut Criterion) {
    const DIM: usize = 128;
    const BATCH: usize = 1_000;

    c.bench_function("insert_batch_1k", |b| {
        b.iter_batched(
            || {
                let db = VecBase::new(VecBaseConfig {
                    dim: DIM,
                    max_elements: BATCH + 64,
                    ..VecBaseConfig::default()
                });
                let vecs: Vec<(String, Vec<f32>)> = (0..BATCH)
                    .map(|i| (format!("v{}", i), gen_vec(i as u64, DIM)))
                    .collect();
                (db, vecs)
            },
            |(mut db, vecs)| {
                for (id, v) in vecs {
                    black_box(db.insert(id, v, None).unwrap());
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

// ── Raw Math Benchmarks ───────────────────────────────────────────────────────

fn bench_cosine_raw(c: &mut Criterion) {
    const DIM: usize = 128;
    let a = gen_vec(1, DIM);
    let b = gen_vec(2, DIM);

    c.bench_function("cosine_similarity_raw_128d", |b_fn| {
        b_fn.iter(|| {
            black_box(cosine_similarity(black_box(&a), black_box(&b)))
        });
    });
}

fn bench_normalize_raw(c: &mut Criterion) {
    const DIM: usize = 128;
    let v = gen_vec(3, DIM);

    c.bench_function("normalize_raw_128d", |b| {
        b.iter(|| {
            black_box(normalize(black_box(&v)))
        });
    });
}

// ── Metric Comparison ─────────────────────────────────────────────────────────

fn bench_search_by_metric(c: &mut Criterion) {
    const DIM: usize = 128;
    const N: usize = 5_000;
    const TOP_K: usize = 10;

    let mut group = c.benchmark_group("search_by_metric");

    for metric in &["cosine", "euclidean", "dot"] {
        let db = build_db(N, DIM, metric);
        let query = gen_vec(55, DIM);

        group.bench_with_input(
            BenchmarkId::from_parameter(metric),
            metric,
            |b, _| {
                b.iter(|| black_box(db.search(black_box(&query), TOP_K)));
            },
        );
    }

    group.finish();
}

// ── Criterion Groups ──────────────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_search_by_size,
    bench_search_by_dim,
    bench_search_by_topk,
    bench_insert_single,
    bench_insert_batch_1k,
    bench_cosine_raw,
    bench_normalize_raw,
    bench_search_by_metric,
);

criterion_main!(benches);
