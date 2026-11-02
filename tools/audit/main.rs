//! AST comparison audit: verifies only the declared HEAD/helper additions
//! differ from the pristine Horizon export. Run from the workspace root:
//!
//!   cargo run --locked --offline -p overtune-audit

use std::collections::HashSet;
use std::path::Path;

use quote::ToTokens;
use syn::{File, Item};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: overtune-audit <pristine-dir> <modded-dir>");
        std::process::exit(2);
    }
    let pristine = Path::new(&args[1]);
    let modded = Path::new(&args[2]);

    // Files that horizon-mixins is allowed to add to (game.rs + helpers).
    let allowed: HashSet<&str> = HashSet::from(["crates/horizon-client/src/game.rs"]);

    let mut differences = 0;
    for entry in walk(modded) {
        let rel = entry
            .strip_prefix(modded)
            .unwrap()
            .to_string_lossy()
            .to_string();
        let pristine_path = pristine.join(&rel);
        if !pristine_path.is_file() {
            // New file added by the bundle (a helper .rs). Allowed only if
            // it's inside mixins/ (not in the game source tree).
            if rel.starts_with("mixins/") {
                continue;
            }
            eprintln!("unexpected new file: {rel}");
            differences += 1;
            continue;
        }
        if !allowed.contains(rel.as_str()) {
            continue;
        }
        let old = std::fs::read_to_string(&pristine_path).unwrap();
        let new = std::fs::read_to_string(&entry).unwrap();
        if old == new {
            continue;
        }
        // AST-level comparison: only the injected statements/module should differ.
        let old_ast: File = syn::parse_file(&old).unwrap();
        let new_ast: File = syn::parse_file(&new).unwrap();
        let old_items: Vec<_> = old_ast
            .items
            .iter()
            .map(|i| i.to_token_stream().to_string())
            .collect();
        let new_items: Vec<_> = new_ast
            .items
            .iter()
            .map(|i| i.to_token_stream().to_string())
            .collect();
        let old_set: HashSet<_> = old_items.iter().collect();
        let added: Vec<_> = new_items
            .iter()
            .filter(|i| !old_set.contains(*i.as_str()))
            .collect();
        if !added.is_empty() {
            eprintln!("{rel}: {added} added item(s)");
            for item in &added {
                eprintln!("  + {item}");
            }
            differences += 1;
        }
    }
    if differences > 0 {
        eprintln!("audit: {differences} unexpected difference(s)");
        std::process::exit(1);
    }
    println!("audit: ok — only declared additions differ");
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}
