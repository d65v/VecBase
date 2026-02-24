// VecBase — example/batch.rs
// Demonstrates batch insert with error reporting.

use vcore::{VecBase, VecBaseConfig};
use vcore::processing::{BatchInsert, batch_insert};

fn main() {
    let mut db = VecBase::new(VecBaseConfig {
        dim: 3,
        ..Default::default()
    });

    let items = vec![
        BatchInsert { id: "a".into(), vector: vec![1.0, 0.0, 0.0], metadata: Some("alpha".into()) },
        BatchInsert { id: "b".into(), vector: vec![0.0, 1.0, 0.0], metadata: None },
        BatchInsert { id: "c".into(), vector: vec![0.0, 0.0, 1.0], metadata: None },
        // Wrong dim — will fail
        BatchInsert { id: "bad".into(), vector: vec![1.0, 2.0], metadata: None },
    ];

    let result = batch_insert(&mut db, items);

    println!("Inserted: {}", result.inserted);
    println!("Failed:   {}", result.failed.len());
    for (id, reason) in &result.failed {
        println!("  {} => {}", id, reason);
    }
}
