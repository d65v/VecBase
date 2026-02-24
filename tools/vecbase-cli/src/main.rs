// VecBase — tools/vecbase-cli/src/main.rs
// Interactive REPL for a running VecBase instance.
// Commands: insert, search, delete, get, len, bench, quit
//
// Build:
//   cd tools/vecbase-cli && cargo build --release
//
// Usage:
//   vecbase-cli [--dim 128] [--metric cosine]
//
// Author: d65v <https://github.com/d65v>

use std::io::{self, BufRead, Write};
use std::time::Instant;

// ── Inline types (mirrors vcore public API) ───────────────────────────────────
// In a real workspace, replace with: use vcore::{VecBase, VecBaseConfig};

struct VecBaseConfig {
    dim: usize,
    metric: String,
    max_elements: usize,
    storage_path: String,
}

impl Default for VecBaseConfig {
    fn default() -> Self {
        Self {
            dim: 128,
            metric: "cosine".into(),
            max_elements: 1_000_000,
            storage_path: "./data".into(),
        }
    }
}

// ── CLI State ─────────────────────────────────────────────────────────────────

struct CliState {
    config: VecBaseConfig,
    // In a real build this would be: db: VecBase,
    // Here we simulate it with a simple counter for portability.
    record_count: usize,
    history: Vec<String>,
}

impl CliState {
    fn new(config: VecBaseConfig) -> Self {
        Self {
            config,
            record_count: 0,
            history: Vec::new(),
        }
    }
}

// ── Command Parser ────────────────────────────────────────────────────────────

#[derive(Debug)]
enum Cmd {
    Insert { id: String, values: Vec<f32> },
    Search { values: Vec<f32>, top_k: usize },
    Delete { id: String },
    Get { id: String },
    Len,
    Bench { n: usize },
    Config,
    History,
    Help,
    Quit,
    Unknown(String),
}

fn parse_cmd(line: &str) -> Cmd {
    let parts: Vec<&str> = line.trim().splitn(3, ' ').collect();
    match parts.as_slice() {
        ["quit"] | ["exit"] | ["q"] => Cmd::Quit,
        ["len"] | ["count"] => Cmd::Len,
        ["help"] | ["h"] | ["?"] => Cmd::Help,
        ["config"] => Cmd::Config,
        ["history"] => Cmd::History,

        ["insert", id, rest] => {
            let values: Option<Vec<f32>> = rest
                .split(',')
                .map(|s| s.trim().parse::<f32>().ok())
                .collect();
            match values {
                Some(v) => Cmd::Insert { id: id.to_string(), values: v },
                None => Cmd::Unknown(format!("insert: invalid float values in '{}'", rest)),
            }
        }

        ["search", rest, k_str] => {
            let top_k = k_str.trim().parse::<usize>().unwrap_or(5);
            let values: Option<Vec<f32>> = rest
                .split(',')
                .map(|s| s.trim().parse::<f32>().ok())
                .collect();
            match values {
                Some(v) => Cmd::Search { values: v, top_k },
                None => Cmd::Unknown(format!("search: invalid float values in '{}'", rest)),
            }
        }

        ["search", rest] => {
            let values: Option<Vec<f32>> = rest
                .split(',')
                .map(|s| s.trim().parse::<f32>().ok())
                .collect();
            match values {
                Some(v) => Cmd::Search { values: v, top_k: 5 },
                None => Cmd::Unknown(format!("search: invalid float values")),
            }
        }

        ["delete", id] | ["del", id] | ["rm", id] => Cmd::Delete { id: id.to_string() },
        ["get", id] => Cmd::Get { id: id.to_string() },

        ["bench", n_str] => {
            let n = n_str.parse::<usize>().unwrap_or(1000);
            Cmd::Bench { n }
        }
        ["bench"] => Cmd::Bench { n: 1000 },

        _ if line.trim().is_empty() => Cmd::Unknown(String::new()),
        _ => Cmd::Unknown(line.to_string()),
    }
}

// ── Command Executor ──────────────────────────────────────────────────────────

fn exec(cmd: Cmd, state: &mut CliState) -> bool {
    match cmd {
        Cmd::Quit => {
            println!("bye.");
            return false;
        }

        Cmd::Help => {
            println!(
                r#"
Commands:
  insert <id> <v1,v2,...,vN>   Insert a vector
  search <v1,v2,...> [top_k]   Search nearest neighbors (default top_k=5)
  delete <id>                  Delete a record
  get    <id>                  Retrieve a record
  len                          Show record count
  bench  [n]                   Insert n random vectors and time search
  config                       Show current configuration
  history                      Show command history
  help                         Show this message
  quit                         Exit

Examples:
  insert doc1 0.1,0.4,0.9,0.3
  search 0.1,0.4,0.8,0.35 3
  delete doc1
"#
            );
        }

        Cmd::Config => {
            println!("  dim          : {}", state.config.dim);
            println!("  metric       : {}", state.config.metric);
            println!("  max_elements : {}", state.config.max_elements);
            println!("  storage_path : {}", state.config.storage_path);
        }

        Cmd::Len => {
            println!("records: {}", state.record_count);
        }

        Cmd::Insert { id, values } => {
            if values.len() != state.config.dim {
                eprintln!(
                    "error: dimension mismatch — expected {}, got {}",
                    state.config.dim,
                    values.len()
                );
            } else {
                // In real build: state.db.insert(id.clone(), values, None).unwrap();
                state.record_count += 1;
                println!("inserted '{}' ({} dims)", id, values.len());
            }
        }

        Cmd::Search { values, top_k } => {
            if values.len() != state.config.dim {
                eprintln!(
                    "error: dimension mismatch — expected {}, got {}",
                    state.config.dim,
                    values.len()
                );
            } else {
                let t = Instant::now();
                // In real build: let results = state.db.search(&values, top_k);
                // Simulated output:
                let elapsed_us = t.elapsed().as_micros();
                println!("top-{} results ({}μs):", top_k, elapsed_us);
                if state.record_count == 0 {
                    println!("  (no records — insert some first)");
                } else {
                    println!("  [connect to a running VecBase instance for real results]");
                }
            }
        }

        Cmd::Delete { id } => {
            if state.record_count == 0 {
                eprintln!("error: no records / '{}' not found", id);
            } else {
                state.record_count = state.record_count.saturating_sub(1);
                println!("deleted '{}'", id);
            }
        }

        Cmd::Get { id } => {
            println!("get '{}': [connect to a running VecBase instance]", id);
        }

        Cmd::Bench { n } => {
            println!("bench: inserting {} random vectors (dim={})...", n, state.config.dim);
            let t0 = Instant::now();
            // Simulate insert time
            for i in 0..n {
                let _ = black_box_u64(i as u64);
            }
            let insert_ms = t0.elapsed().as_millis();
            println!("  insert {}  : ~{}ms (simulated)", n, insert_ms);

            let t1 = Instant::now();
            let _ = black_box_u64(42);
            let search_us = t1.elapsed().as_micros();
            println!("  search top-10: ~{}μs (simulated)", search_us);
            println!("  (run `cargo bench` in vcore/ for real criterion benchmarks)");
        }

        Cmd::History => {
            if state.history.is_empty() {
                println!("(no history)");
            } else {
                for (i, h) in state.history.iter().enumerate() {
                    println!("  {:3}  {}", i + 1, h);
                }
            }
        }

        Cmd::Unknown(s) if s.is_empty() => {} // blank line — do nothing

        Cmd::Unknown(s) => {
            eprintln!("unknown command: '{}'. Type 'help' for commands.", s);
        }
    }
    true
}

/// Minimal black-box to prevent the bench loop from being optimized away.
#[inline(never)]
fn black_box_u64(x: u64) -> u64 {
    unsafe {
        let ret: u64;
        std::arch::asm!("/* {0} */", in(reg) x, out(reg) ret, options(nostack, nomem, pure));
        ret
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    // Parse CLI args
    let args: Vec<String> = std::env::args().collect();

    let mut dim = 128usize;
    let mut metric = "cosine".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dim" | "-d" => {
                i += 1;
                dim = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(128);
            }
            "--metric" | "-m" => {
                i += 1;
                metric = args.get(i).cloned().unwrap_or_else(|| "cosine".into());
            }
            "--help" | "-h" => {
                println!("vecbase-cli [--dim N] [--metric cosine|euclidean|dot]");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let config = VecBaseConfig {
        dim,
        metric: metric.clone(),
        ..VecBaseConfig::default()
    };

    let mut state = CliState::new(config);

    println!("VecBase CLI  •  dim={}  metric={}  •  type 'help'", dim, metric);
    println!("────────────────────────────────────────────────────");

    let stdin = io::stdin();
    loop {
        print!("vecbase> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => break, // EOF
            Ok(_) => {}
        }

        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            state.history.push(trimmed.clone());
        }

        let cmd = parse_cmd(&trimmed);
        if !exec(cmd, &mut state) {
            break;
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quit() {
        assert!(matches!(parse_cmd("quit"), Cmd::Quit));
        assert!(matches!(parse_cmd("q"), Cmd::Quit));
        assert!(matches!(parse_cmd("exit"), Cmd::Quit));
    }

    #[test]
    fn test_parse_insert() {
        let cmd = parse_cmd("insert vec1 0.1,0.2,0.3");
        assert!(matches!(cmd, Cmd::Insert { .. }));
        if let Cmd::Insert { id, values } = cmd {
            assert_eq!(id, "vec1");
            assert_eq!(values.len(), 3);
            assert!((values[0] - 0.1).abs() < 1e-6);
        }
    }

    #[test]
    fn test_parse_search_with_topk() {
        let cmd = parse_cmd("search 0.1,0.2,0.3 10");
        assert!(matches!(cmd, Cmd::Search { top_k: 10, .. }));
    }

    #[test]
    fn test_parse_search_default_topk() {
        let cmd = parse_cmd("search 0.1,0.2");
        assert!(matches!(cmd, Cmd::Search { top_k: 5, .. }));
    }

    #[test]
    fn test_parse_delete() {
        assert!(matches!(parse_cmd("del abc"), Cmd::Delete { .. }));
        assert!(matches!(parse_cmd("rm abc"), Cmd::Delete { .. }));
    }

    #[test]
    fn test_parse_bench_default() {
        assert!(matches!(parse_cmd("bench"), Cmd::Bench { n: 1000 }));
    }

    #[test]
    fn test_parse_bench_custom() {
        assert!(matches!(parse_cmd("bench 5000"), Cmd::Bench { n: 5000 }));
    }

    #[test]
    fn test_parse_len() {
        assert!(matches!(parse_cmd("len"), Cmd::Len));
        assert!(matches!(parse_cmd("count"), Cmd::Len));
    }

    #[test]
    fn test_blank_line_is_unknown_empty() {
        assert!(matches!(parse_cmd("  "), Cmd::Unknown(_)));
    }

    #[test]
    fn test_insert_dim_check() {
        let mut state = CliState::new(VecBaseConfig {
            dim: 3,
            ..VecBaseConfig::default()
        });
        // Insert with correct dim
        exec(
            Cmd::Insert { id: "x".into(), values: vec![1.0, 0.0, 0.0] },
            &mut state,
        );
        assert_eq!(state.record_count, 1);

        // Insert with wrong dim — count should not increase
        exec(
            Cmd::Insert { id: "y".into(), values: vec![1.0, 0.0] },
            &mut state,
        );
        assert_eq!(state.record_count, 1);
    }
}
