# Contributing to VecBase

Thanks for being here. VecBase is open source and welcomes contributions from anyone.

---

## Ground Rules

- Keep it minimal. If it adds bloat without clear benefit, it probably doesn't belong.
- Write clean Rust. `cargo clippy` should be happy.
- Document what you add. Even a short comment is better than none.
- Open an issue before a large PR — saves both of our time.

---

## How to Contribute

1. Fork the repo
2. Create a branch: `git checkout -b feat/your-feature`
3. Make your changes
4. Run: `cargo test && cargo clippy`
5. Commit: `git commit -m "feat: short description"`
6. Push and open a PR

---

## What's Needed

- [ ] More ANN algorithm implementations
- [ ] Persistence layer (disk-based storage)
- [ ] REST API / gRPC interface
- [ ] Python bindings via PyO3
- [ ] Benchmarks
- [ ] More embedding format support

---

## Code Style

- Use `rustfmt` (`cargo fmt`)
- Prefer `Result<T, E>` over panics in library code
- Write unit tests for new logic

---

## Author

[d65v](https://github.com/d65v)
