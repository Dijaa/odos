<p align="center">
  <img src="https://raw.githubusercontent.com/Dijaa/odos/main/assets/odos-logo.svg" alt="odos" width="320">
</p>

# odos

File-based routing for [Dioxus](https://dioxuslabs.com) 0.7, inspired by the
Next.js App Router. Create files in `src/pages/` and the `Route` enum is
generated at compile time.

```text
src/pages/
├── layout.rs              root layout
├── page.rs                /
├── not_found.rs           404
├── about/page.rs          /about
├── (admin)/dashboard/page.rs   /dashboard   (group: not in the URL)
├── _components/           ignored
└── blog/
    ├── layout.rs          layout for /blog/*
    ├── page.rs            /blog
    ├── [slug]/page.rs     /blog/:slug
    └── [...rest]/page.rs  /blog/:..rest
```

## Quick start

```toml
# Cargo.toml
[dependencies]
dioxus = { version = "0.7", features = ["router"] }
odos = "0.1"

[build-dependencies]
odos-build = "0.1"
```

```rust
// build.rs
fn main() {
    odos_build::build();
}
```

```rust
// src/main.rs
use dioxus::prelude::*;

odos::routes!(); // defines `Route`

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! { Router::<Route> {} }
}
```

```rust
// src/pages/blog/[slug]/page.rs
use dioxus::prelude::*;

#[component]
pub fn Page(slug: String) -> Element {
    rsx! { h1 { "Post: {slug}" } }
}
```

Link to it with `Route::BlogSlug { slug: "hello".into() }`.

The full guide (layouts, parameters, route names, build errors) is on
[docs.rs/odos](https://docs.rs/odos). A working app is in
[`examples/basic`](examples/basic).

## Crates

| Crate | Goes in | What it does |
|-------|---------|--------------|
| [`odos`](https://crates.io/crates/odos) | `[dependencies]` | `routes!()` macro, no dependencies |
| [`odos-build`](https://crates.io/crates/odos-build) | `[build-dependencies]` | Scans `src/pages/` and generates the routes |

## License

MIT OR Apache-2.0
