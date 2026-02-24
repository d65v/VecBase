# VecBase Plugin System

VecBase supports plugins compiled as `cdylib` dynamic libraries (`.so` / `.dylib` / `.dll`).

---

## Plugin Interface

Every plugin must implement the `Plugin` trait defined in `vcore/src/lib.rs`:

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn on_init(&self);
    fn on_insert(&self, id: &str, vector: &mut Vec<f32>, metadata: &mut Option<String>);
    fn on_search_results(&self, results: &mut Vec<SearchResult>);
}
```

And export the init symbol:

```rust
#[no_mangle]
pub extern "C" fn vecbase_plugin_init() -> *mut dyn Plugin {
    Box::into_raw(Box::new(MyPlugin))
}
```

---

## Example Plugin Crate Layout

```
my_plugin/
├── Cargo.toml
└── src/
    └── lib.rs
```

`Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
vecbase = { path = "../../vcore" }
```

---

## Loading a Plugin

Place the compiled `.so` file in `vcore/src/plug-ins/` and set:

```env
VECBASE_PLUGINS=my_plugin.so,another.so
```

VecBase will `dlopen` each listed plugin on startup.

---

## Hook Descriptions

| Hook                | When Called                       | Use Case                          |
|--------------------|-----------------------------------|-----------------------------------|
| `on_init`          | Plugin loaded                     | Warm up resources                 |
| `on_insert`        | Before storing a vector           | Normalize, enrich, reject         |
| `on_search_results`| After search, before returning    | Rerank, filter, add context       |
