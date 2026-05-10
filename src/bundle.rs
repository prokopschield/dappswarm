//! DNP bundle model — minimal, just enough to publish + install.
//!
//! A Dappnode package on disk is a flat directory containing
//! `dappnode_package.json`, `docker-compose.yml`, and zero or more
//! `*.tar.xz` Docker image archives. We pack it as an uncompressed tar
//! for upload (Swarm content-addresses chunks; recompressing would
//! defeat dedup) and unpack on the resolver side.
//!
//! Manifest validation here is intentionally shallow — only the fields
//! `dappswarm` itself needs to act on. Anything else (hardware reqs,
//! categories, links) passes through opaquely.

use std::fs::File;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MANIFEST_FILENAME: &str = "dappnode_package.json";
pub const COMPOSE_FILENAME: &str = "docker-compose.yml";

/// Required + commonly-present fields from `dappnode_package.json`.
///
/// Unknown fields are preserved verbatim for round-trip safety via
/// `extra` so we never strip data the user might rely on.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Error)]
pub enum BundleError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("manifest at {path:?} is missing or unreadable: {source}")]
    ManifestMissing {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("manifest is not valid JSON: {0}")]
    ManifestInvalid(#[from] serde_json::Error),

    #[error("docker-compose.yml not found in bundle")]
    ComposeMissing,

    #[error("manifest field `{0}` is required and missing or empty")]
    ManifestField(&'static str),

    #[error("walk error: {0}")]
    Walk(#[from] walkdir::Error),
}

/// Load and validate `<dir>/dappnode_package.json`.
pub fn load_manifest(dir: &Path) -> Result<Manifest, BundleError> {
    let path = dir.join(MANIFEST_FILENAME);
    let bytes = std::fs::read(&path).map_err(|e| BundleError::ManifestMissing {
        path: path.clone(),
        source: e,
    })?;
    let manifest: Manifest = serde_json::from_slice(&bytes)?;

    validate(&manifest)?;

    Ok(manifest)
}

fn validate(m: &Manifest) -> Result<(), BundleError> {
    if m.name.trim().is_empty() {
        return Err(BundleError::ManifestField("name"));
    }

    if m.version.trim().is_empty() {
        return Err(BundleError::ManifestField("version"));
    }

    Ok(())
}

/// Pack a bundle directory into an uncompressed tar. The bundle root is
/// the tar's root (paths are stored relative to `dir`, e.g.
/// `./dappnode_package.json`), matching the layout `POST /bzz`'s
/// `swarm-collection: true` mode expects for a Mantaray manifest.
pub fn pack(dir: &Path) -> Result<Vec<u8>, BundleError> {
    let mut buf = Vec::new();

    {
        let mut builder = tar::Builder::new(&mut buf);
        builder.mode(tar::HeaderMode::Deterministic);
        builder.follow_symlinks(false);
        builder.append_dir_all(".", dir)?;
        builder.finish()?;
    }

    Ok(buf)
}

/// Unpack a tar (as produced by [`pack`] or by Swarm's `bzz` GET on a
/// collection) into `dir`. `dir` is created if it does not exist.
pub fn unpack(tar_bytes: &[u8], dir: &Path) -> Result<(), BundleError> {
    std::fs::create_dir_all(dir)?;

    let mut archive = tar::Archive::new(Cursor::new(tar_bytes));
    archive.unpack(dir)?;

    Ok(())
}

/// Verify the unpacked bundle has the files we need to install — the
/// manifest and the compose file. Image tarballs are optional (a
/// package might use registry images only).
pub fn check_layout(dir: &Path) -> Result<Manifest, BundleError> {
    let manifest = load_manifest(dir)?;
    let compose = dir.join(COMPOSE_FILENAME);

    if !compose.is_file() {
        return Err(BundleError::ComposeMissing);
    }

    Ok(manifest)
}

/// `true` when `path` ends in `.tar` or `.tar.xz` (case-insensitively).
fn is_image_archive(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return false;
    };

    if ext.eq_ignore_ascii_case("tar") {
        return true;
    }

    if !ext.eq_ignore_ascii_case("xz") {
        return false;
    }

    path.file_stem()
        .and_then(|s| s.to_str())
        .and_then(|stem| Path::new(stem).extension().and_then(|e| e.to_str()))
        .is_some_and(|inner| inner.eq_ignore_ascii_case("tar"))
}

/// Enumerate `*.tar.xz` files at the bundle root (used by `install`).
pub fn image_tarballs(dir: &Path) -> Result<Vec<PathBuf>, BundleError> {
    let mut out = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_image_archive(&path) {
            out.push(path);
        }
    }

    out.sort();

    Ok(out)
}

/// Read every regular file in `dir` recursively, returning
/// `(relative_path, bytes)`. Used by tests for round-trip equality.
pub fn read_tree(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, BundleError> {
    let mut entries = Vec::new();

    for entry in walkdir::WalkDir::new(dir).sort_by_file_name() {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        let rel = entry
            .path()
            .strip_prefix(dir)
            .expect("walkdir under root")
            .to_string_lossy()
            .into_owned();
        let mut buf = Vec::new();

        File::open(entry.path())?.read_to_end(&mut buf)?;
        entries.push((rel, buf));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_fixture(dir: &Path) {
        std::fs::write(
            dir.join("dappnode_package.json"),
            r#"{
                "name": "hello.dnp.dappnode.eth",
                "version": "0.1.0",
                "description": "fixture",
                "type": "service",
                "license": "MIT"
            }"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("docker-compose.yml"),
            "version: '3'\nservices:\n  web:\n    image: nginx:alpine\n",
        )
        .unwrap();
        std::fs::write(dir.join("binary.bin"), [0u8, 1, 255, 42, 7, 7, 7]).unwrap();
    }

    #[test]
    fn manifest_loads_required_fields() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path());

        let m = load_manifest(tmp.path()).unwrap();

        assert_eq!(m.name, "hello.dnp.dappnode.eth");
        assert_eq!(m.version, "0.1.0");
    }

    #[test]
    fn manifest_rejects_empty_name() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("dappnode_package.json"),
            r#"{"name": "", "version": "0.1.0"}"#,
        )
        .unwrap();

        let err = load_manifest(tmp.path()).unwrap_err();

        assert!(matches!(err, BundleError::ManifestField("name")));
    }

    #[test]
    fn pack_unpack_roundtrip() {
        let src = TempDir::new().unwrap();
        write_fixture(src.path());

        let tar_bytes = pack(src.path()).unwrap();

        let dst = TempDir::new().unwrap();
        unpack(&tar_bytes, dst.path()).unwrap();

        let before = read_tree(src.path()).unwrap();
        let after = read_tree(dst.path()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn check_layout_requires_compose() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("dappnode_package.json"),
            r#"{"name": "x", "version": "0.1.0"}"#,
        )
        .unwrap();

        let err = check_layout(tmp.path()).unwrap_err();

        assert!(matches!(err, BundleError::ComposeMissing));
    }
}
