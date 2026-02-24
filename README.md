# VecBase

> A Minimal, Fast & Lightweight Vector Database — written in Rust.

```
 __   __        ____
 \ \ / /__ ___ | __ )  __ _ ___  ___
  \ V / _ \/ __||  _ \ / _` / __|/ _ \
   | |  __/ (__ | |_) | (_| \__ \  __/
   |_|\___|\___||____/ \__,_|___/\___|
```

**Status**: 🚧 Being Worked On

---

## What is VecBase?

VecBase is a minimal vector database built for AI embedding storage, similarity search, and lightweight deployments. It is designed to be fast, with zero unnecessary bloat.

**Made by**: [d65v](https://github.com/d65v) — *A Normal Rust Developer. Experience: 6 months in Rust, 1 day in C. Worked on: VecBase.*

---

## Features

- ⚡ Fast — Cosine & Euclidean similarity search
- 🪶 Lightweight — minimal dependencies
- 🧠 AI-Ready — designed for embedding vectors from LLMs, CNNs, etc.
- 🔌 Plugin system via `cdylib` dynamic libraries
- 🔍 ANN (Approximate Nearest Neighbor) via HNSW
- 🐳 Docker-ready

---

## Project Structure

```
VecBase/
├── vcore/              # Vector core engine (Rust crate)
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── embedding.rs
│   │   ├── processing.rs
│   │   ├── plug-ins/
│   │   └── algorithm/
├── database/           # Database storage layers
│   ├── Common/
│   ├── Advanced/
│   └── OTHERS/
├── SetUp/              # Setup scripts
├── doc/                # Documentation
├── tools/              # CLI & utilities
├── example/            # Usage examples
├── Dockerfile
├── docker-compose.yml
└── Makefile
```

---

## Quick Start

```bash
# Setup
make setup

# Build
make build

# Run
make run

# Or via Docker
docker-compose up --build
```

---

## Documentation

- [USAGE.md](./USAGE.md) — How to use VecBase
- [CONTRIBUTING.md](./CONTRIBUTING.md) — How to contribute
- [vcore/doc-core.md](./vcore/doc-core.md) — Core internals
- [vcore/src/algorithm/algorithm.md](./vcore/src/algorithm/algorithm.md) — ANN algorithm docs
- [doc/](./doc/) — Full documentation

---

## License

apache2 © [d65v](https://github.com/d65v)
