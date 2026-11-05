//! Build-time route generator for [`odos`](https://crates.io/crates/odos).
//!
//! **Work in progress.** Call [`build`] from your `build.rs` to turn
//! the files in `src/pages/` into a Dioxus `Route` enum.
//!
//! ```ignore
//! // build.rs
//! fn main() {
//!     odos_build::build();
//! }
//! ```

/// Scans `src/pages/` and generates the routes. Call it from `build.rs`.
pub fn build() {
    println!("cargo:rerun-if-changed=src/pages");
}
