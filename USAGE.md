# VecBase — Usage Guide

---

## Installation

### From Source

```bash
git clone https://github.com/d65v/vecbase
cd vecbase
make setup
make build
```

### Docker

```bash
docker-compose up --build
```

---

## Basic Usage

### Insert a Vector

```rust
use vcore::VecBase;

fn main() {
    let mut db = VecBase::new();

    let id = "doc_001".to_string();
    let vector = vec![0.1, 0.4, 0.9, 0.3];
    let metadata = Some("my first vector".to_string());

    db.insert(id, vector, metadata).unwrap();
    println!("Inserted.");
}
```

### Query (Nearest Neighbor)

```rust
let query = vec![0.1, 0.4, 0.8, 0.35];
let results = db.search(&query, 5); // top 5 neighbors

for r in results {
    println!("id={} score={:.4}", r.id, r.score);
}
```

### Delete

```rust
db.delete("doc_001").unwrap();
```

---

## Configuration via `.env`

Copy the example environment file:

```bash
cp vcore/src/plug-ins/env.example .env
```

Edit `.env` to configure:

```env
VECBASE_DIM=128
VECBASE_METRIC=cosine
VECBASE_MAX_ELEMENTS=100000
VECBASE_STORAGE_PATH=./data
```

---

## Plugins

Place compiled plugin `.so` / `.dylib` files in `vcore/src/plug-ins/`.

See [plug-ins/plugins.md](./vcore/src/plug-ins/plugins.md) for the plugin interface spec.

---

## Makefile Targets

| Command         | Description              |
|----------------|--------------------------|
| `make setup`   | Install dependencies     |
| `make build`   | Build release binary     |
| `make run`     | Run the server           |
| `make test`    | Run all tests            |
| `make clean`   | Clean build artifacts    |
| `make fmt`     | Format code              |
| `make lint`    | Run clippy               |
| `make docker`  | Build docker image       |
