//! `dappswarm publish <dir>` orchestrator.
//!
//! Steps: load + validate manifest, pack the bundle, post the tar to
//! `/bzz`, find the next feed index by reading the SOC chain, sign +
//! upload a new SOC pointing at the bzz reference. Returns the
//! published reference + index for the CLI to print.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::bundle::{self, MANIFEST_FILENAME};
use crate::feed::{self, FeedHit};
use crate::soc::Soc;
use crate::swarm::Client;

/// JSON envelope stored in each feed SOC. ≤ 4 KiB by construction.
///
/// `files` is a sorted list of bundle-relative paths, populated at
/// publish time from the source tree. Embedding it lets `resolve` walk
/// the bundle via plain `GET /bzz/<ref>/<path>` calls without needing a
/// manifest-listing endpoint, useful when reads happen through a
/// public Bee gateway that lacks `/v0/manifest/<ref>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedPayload {
    pub version: String,
    /// 0x-prefixed lowercase 64-hex bzz reference of the bundle's
    /// Mantaray manifest.
    #[serde(rename = "ref")]
    pub bzz_ref: String,
    pub published_at: u64,
    #[serde(default)]
    pub files: Vec<String>,
}

/// Outcome of a successful publish, handed to the CLI for display.
#[derive(Debug, Clone)]
pub struct PublishResult {
    pub manifest_name: String,
    pub manifest_version: String,
    pub bzz_ref: String,
    pub feed_index: u64,
    pub soc: Soc,
}

/// Pack `dir`, push to bzz, append a feed entry. `secret` is the
/// publisher's secp256k1 signing key (32 bytes).
pub async fn run(client: &Client, secret: &[u8; 32], dir: &Path) -> Result<PublishResult> {
    let manifest = bundle::check_layout(dir)
        .with_context(|| format!("invalid bundle at {}", dir.display()))?;

    let tar_bytes =
        bundle::pack(dir).with_context(|| format!("packing bundle at {}", dir.display()))?;

    let files: Vec<String> = bundle::read_tree(dir)
        .with_context(|| format!("scanning bundle tree at {}", dir.display()))?
        .into_iter()
        .map(|(rel, _)| rel)
        .collect();

    let bzz_ref = client
        .post_bzz_tar(tar_bytes, MANIFEST_FILENAME)
        .await
        .context("uploading bundle tar to /bzz")?;

    let payload = FeedPayload {
        version: manifest.version.clone(),
        bzz_ref: ensure_0x_prefix(&bzz_ref),
        published_at: now_unix_secs(),
        files,
    };
    let payload_bytes = serde_json::to_vec(&payload).expect("FeedPayload serializes");

    // Owner_eoa is derived from `secret` by the SOC builder; we need it
    // up-front to look up the latest existing index for this name.
    let probe_soc =
        crate::soc::build(secret, [0u8; 32], &[]).context("deriving owner from secret")?;

    let next_index = match feed::find_latest(client, &probe_soc.owner_eoa, &manifest.name)
        .await
        .with_context(|| format!("scanning feed for {}", manifest.name))?
    {
        Some(FeedHit { index, .. }) => index + 1,
        None => 0,
    };

    let soc = feed::write(client, secret, &manifest.name, next_index, &payload_bytes)
        .await
        .with_context(|| format!("writing feed entry for {} @ {next_index}", manifest.name))?;

    Ok(PublishResult {
        manifest_name: manifest.name,
        manifest_version: manifest.version,
        bzz_ref: ensure_0x_prefix(&bzz_ref),
        feed_index: next_index,
        soc,
    })
}

fn ensure_0x_prefix(s: &str) -> String {
    if s.starts_with("0x") || s.starts_with("0X") {
        s.to_string()
    } else {
        format!("0x{s}")
    }
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
