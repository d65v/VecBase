// VecBase — example/basic.rs
// Demonstrates basic insert, search, and delete operations.
// Run with:  cargo run --example basic  (from vcore/ directory)
//
// Add to vcore/Cargo.toml:
// [[example]]
// name = "basic"
// path = "../example/basic.rs"

use vcore::{VecBase, VecBaseConfig};

fn main() {
    env_logger::init();

    println!("── VecBase Basic Example ─────────────────────");

    let config = VecBaseConfig {
        dim: 4,
        metric: "cosine".to_string(),
        ..Default::default()
    };

    let mut db = VecBase::new(config);

    // Insert some vectors
    let records = vec![
        ("cat",  vec![0.9, 0.1, 0.0, 0.0]),
        ("dog",  vec![0.8, 0.2, 0.0, 0.0]),
        ("car",  vec![0.0, 0.0, 0.9, 0.1]),
        ("bus",  vec![0.0, 0.0, 0.8, 0.2]),
        ("fish", vec![0.5, 0.5, 0.0, 0.0]),
    ];

    for (id, v) in &records {
        db.insert(id.to_string(), v.clone(), Some(format!("label:{}", id)))
            .expect("insert failed");
    }

    println!("Inserted {} vectors.\n", db.len());

    // Query: something close to "cat"
    let query = vec![0.95, 0.05, 0.0, 0.0];
    let results = db.search(&query, 3);

    println!("Top-3 results for query [0.95, 0.05, 0.0, 0.0]:");
    for r in &results {
        println!("  {:6}  score={:.4}  meta={:?}", r.id, r.score, r.metadata);
    }

    println!("\nExpected: cat, dog, fish (in that order)");

    // Delete
    db.delete("fish").unwrap();
    println!("\nDeleted 'fish'. DB size: {}", db.len());

    println!("\n── Done ──────────────────────────────────────");
}
