//! Filesystem helpers for reading/writing AI profile artifacts.

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Result returned after writing a profile artifact.
#[derive(Debug, Clone)]
pub struct ProfileWriteOutcome {
    pub path: PathBuf,
    pub hash: String,
}

/// Loads a JSON profile artifact if it exists.
pub fn read_profile<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<Option<T>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    let data =
        fs::read(path).with_context(|| format!("Failed reading profile artifact {:?}", path))?;
    let value = serde_json::from_slice(&data)
        .with_context(|| format!("Failed parsing profile artifact {:?}", path))?;
    Ok(Some(value))
}

/// Writes a JSON profile artifact with deterministic ordering and returns its hash.
pub fn write_profile<T: Serialize, P: AsRef<Path>>(
    path: P,
    value: &T,
) -> Result<ProfileWriteOutcome> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed creating profile directory {:?}", parent))?;
    }
    let payload = serde_json::to_vec_pretty(value)
        .with_context(|| format!("Failed serializing profile artifact {:?}", path))?;
    let hash = compute_hash(&payload);
    let mut file = fs::File::create(path)
        .with_context(|| format!("Failed opening profile artifact {:?}", path))?;
    file.write_all(&payload)?;
    Ok(ProfileWriteOutcome {
        path: path.to_path_buf(),
        hash,
    })
}

/// Writes an HTML summary artifact alongside the JSON representation.
pub fn write_profile_html<P: AsRef<Path>>(path: P, contents: &str) -> Result<PathBuf> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed creating summary directory {:?}", parent))?;
    }
    fs::write(path, contents).with_context(|| format!("Failed writing profile HTML {:?}", path))?;
    Ok(path.to_path_buf())
}

/// Computes a lowercase hex SHA-256 hash of the provided bytes.
pub fn compute_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{:x}", digest)
}
