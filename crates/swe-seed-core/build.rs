//! Embeds the curated harness resource bundle (`resources/harness/`) into the
//! crate so `swe-seed init` works from a crates.io or native install with no
//! source checkout. Generates `$OUT_DIR/bundle.rs` with a sorted
//! `(relative_path, bytes)` table.

use std::fs;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let root = manifest.join("resources").join("harness");
    println!("cargo:rerun-if-changed=resources/harness");

    let mut files = Vec::new();
    walk(&root, &mut files);

    let mut src = String::from("pub const BUNDLE: &[(&str, &[u8])] = &[\n");
    for f in &files {
        println!("cargo:rerun-if-changed={}", f.display());
        let rel = f
            .strip_prefix(&root)
            .expect("under root")
            .to_string_lossy()
            .replace('\\', "/");
        src.push_str(&format!(
            "    ({:?}, include_bytes!({:?})),\n",
            rel,
            f.display().to_string()
        ));
    }
    src.push_str("];\n");

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("out dir")).join("bundle.rs");
    fs::write(out, src).expect("write bundle.rs");
}
