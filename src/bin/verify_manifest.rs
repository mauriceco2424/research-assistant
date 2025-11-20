use anyhow::{Context, Result};
use researchbase::reports::manifest::{hash_path, read_manifest};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let manifest_arg = env::args()
        .nth(1)
        .context("Usage: cargo run --bin verify_manifest -- <path-to-manifest.json>")?;
    let manifest_path = PathBuf::from(manifest_arg);
    let manifest = read_manifest(&manifest_path)?;
    let mut failures = Vec::new();
    for output in &manifest.outputs {
        if !output.path.exists() {
            failures.push(format!(
                "[missing] {} ({})",
                output.path.display(),
                output.scope
            ));
            continue;
        }
        if output.hash.is_empty() {
            continue;
        }
        let current_hash = hash_path(&output.path)?;
        if current_hash != output.hash {
            failures.push(format!(
                "[mismatch] {} ({}) expected {}, found {}",
                output.path.display(),
                output.scope,
                output.hash,
                current_hash
            ));
        }
    }
    if failures.is_empty() {
        println!(
            "Manifest {} verified: {} outputs match recorded hashes.",
            manifest_path.display(),
            manifest.outputs.len()
        );
        Ok(())
    } else {
        println!("Manifest verification failed:");
        for failure in &failures {
            println!("  - {}", failure);
        }
        anyhow::bail!(
            "Manifest verification failed ({} mismatches)",
            failures.len()
        )
    }
}
