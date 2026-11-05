//! Build-time route generator for [`odos`](https://crates.io/crates/odos).
//!
//! Call [`build`] from your `build.rs` to turn
//! the files in `src/pages/` into a Dioxus `Route` enum.
//!
//! ```ignore
//! // build.rs
//! fn main() {
//!     odos_build::build();
//! }
//! ```
mod codegen;
mod naming;
mod scan;
mod segment;
mod signature;

use std::path::PathBuf;
use std::{env, fs, process};

/// Scans `src/pages/` and writes `$OUT_DIR/odos_routes.rs`. Call it from `build.rs`.
///
/// On error, prints the message and fails the build.
pub fn build() {
    // A folder here makes Cargo watch every file inside it.
    println!("cargo:rerun-if-changed=src/pages");
    if let Err(e) = try_build() {
        eprintln!("odos: {e}");
        process::exit(1);
    }
}

fn try_build() -> Result<(), String> {
    let var = |name| env::var(name).map_err(|e| format!("{name}: {e} (run this from build.rs)"));
    let root = PathBuf::from(var("CARGO_MANIFEST_DIR")?).join("src").join("pages");
    let out = PathBuf::from(var("OUT_DIR")?).join("odos_routes.rs");

    let mut tree = scan::scan(&root)?;
    signature::read_signatures(&mut tree)?;
    naming::check_collisions(&tree)?;
    let code = codegen::generate(&tree);

    fs::write(&out, code).map_err(|e| format!("{}: {e}", out.display()))
}
