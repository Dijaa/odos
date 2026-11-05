# odos-build

Build-time route generator for [odos](https://crates.io/crates/odos),
file-based routing for [Dioxus](https://dioxuslabs.com).

Add it to `[build-dependencies]` and call it from `build.rs`:

```rust
fn main() {
    odos_build::build();
}
```

It scans `src/pages/`, checks every `page.rs`, and writes the `Route` enum to
`$OUT_DIR/odos_routes.rs`, which `odos::routes!()` includes.

See the [odos docs](https://docs.rs/odos/latest/odos/) for the file conventions.

## License

MIT OR Apache-2.0
