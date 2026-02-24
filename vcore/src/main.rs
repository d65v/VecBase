// VecBase — main.rs
// Binary entry point for the VecBase server / CLI.
// Author: d65v <https://github.com/d65v>

use std::env;

use vcore::{VecBase, VecBaseConfig};

fn main() {
    // Initialize logger — respects RUST_LOG env var
    env_logger::init();

    // Load .env if present (non-fatal if missing)
    let _ = dotenv::dotenv();

    log::info!("VecBase starting up...");

    // Parse a simple CLI arg for now
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("run");

    match mode {
        "run" => run_server(),
        "bench" => run_bench(),
        "help" | "--help" | "-h" => print_help(),
        unknown => {
            eprintln!("[VecBase] Unknown mode: '{}'. Try --help.", unknown);
            std::process::exit(1);
        }
    }
}

fn run_server() {
    let config = VecBaseConfig::from_env();

    log::info!(
        "Config: dim={}, metric={}, max_elements={}",
        config.dim,
        config.metric,
        config.max_elements
    );

    let mut db = VecBase::new(config);

    // Demo: insert a few vectors and query
    // In a real deployment this would be replaced by a TCP/HTTP/gRPC server loop.
    log::info!("Inserting demo vectors...");

    for i in 0..10u32 {
        let id = format!("vec_{}", i);
        let vector: Vec<f32> = (0..db.config.dim)
            .map(|j| (i as f32 + j as f32) / 100.0)
            .collect();
        db.insert(id.clone(), vector, Some(format!("demo metadata {}", i)))
            .expect("insert failed");
    }

    log::info!("Inserted 10 demo vectors.");

    // Query with a random-ish vector
    let query: Vec<f32> = (0..db.config.dim).map(|j| j as f32 / 100.0).collect();
    let results = db.search(&query, 3);

    println!("\n[VecBase] Top-3 results for demo query:");
    for r in &results {
        println!("  id={:8}  score={:.6}  meta={:?}", r.id, r.score, r.metadata);
    }

    log::info!("VecBase demo complete.");
}

fn run_bench() {
    use std::time::Instant;

    println!("[VecBase] Running internal benchmark...");

    let config = VecBaseConfig {
        dim: 128,
        ..VecBaseConfig::default()
    };

    let mut db = VecBase::new(config.clone());
    let n = 10_000usize;

    let t0 = Instant::now();
    for i in 0..n {
        let id = format!("b_{}", i);
        let v: Vec<f32> = (0..config.dim).map(|j| (i as f32 * j as f32).sin()).collect();
        db.insert(id, v, None).unwrap();
    }
    let insert_ms = t0.elapsed().as_millis();
    println!("  Inserted {} vectors in {}ms", n, insert_ms);

    let query: Vec<f32> = (0..config.dim).map(|j| (j as f32).cos()).collect();
    let t1 = Instant::now();
    let _ = db.search(&query, 10);
    let search_us = t1.elapsed().as_micros();
    println!("  Search (top-10) in {}μs", search_us);
}

fn print_help() {
    println!(
        r#"
VecBase — A Minimal Vector Database

USAGE:
  vecbase [MODE]

MODES:
  run     Start VecBase (default)
  bench   Run internal performance benchmark
  help    Show this message

ENVIRONMENT:
  VECBASE_DIM             Vector dimensionality (default: 128)
  VECBASE_METRIC          Similarity metric: cosine | euclidean | dot (default: cosine)
  VECBASE_MAX_ELEMENTS    Max vectors to hold in memory (default: 1000000)
  VECBASE_STORAGE_PATH    Path for persistence (default: ./data)
  RUST_LOG                Log level: info | debug | warn | error

AUTHOR:
  d65v <https://github.com/d65v>
"#
    );
}
