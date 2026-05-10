//! `dappswarm resolve <name> --owner …` orchestrator.
//!
//! Steps: scan the owner's feed for `name` to find the latest payload,
//! decode the JSON, fetch each manifest entry under that bzz reference,
//! write the files into `out`, and re-validate the manifest.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::bundle::{self, Manifest};
use crate::feed::{self, FeedHit};
use crate::publish::FeedPayload;
use crate::swarm::Client;

/// Outcome of a successful resolve — handed to the CLI for display.
#[derive(Debug, Clone)]
pub struct ResolveResult {
    pub manifest: Manifest,
    pub bzz_ref: String,
    pub feed_index: u64,
    pub published_at: u64,
    pub out_dir: PathBuf,
}

/// Resolve `name` published by `owner_eoa` and unpack the bundle into
/// `out_dir`. The directory is created if it does not exist; files
/// already present are overwritten.
pub async fn run(
    client: &Client,
    owner_eoa: &[u8; 20],
    name: &str,
    out_dir: &Path,
) -> Result<ResolveResult> {
    let hit = feed::find_latest(client, owner_eoa, name)
        .await
        .with_context(|| format!("scanning feed for {name}"))?
        .ok_or_else(|| {
            anyhow!(
                "no feed entry found for {name} under owner {}",
                hex::encode(owner_eoa)
            )
        })?;
    let FeedHit { index, payload } = hit;

    let payload: FeedPayload = serde_json::from_slice(&payload)
        .with_context(|| format!("decoding feed payload at {name} @ {index}"))?;

    let bzz_ref = strip_0x(&payload.bzz_ref);

    if payload.files.is_empty() {
        return Err(anyhow!(
            "feed payload at {name} @ {index} has no files list — \
             republish with a newer dappswarm to embed it"
        ));
    }

    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("creating output directory at {}", out_dir.display()))?;

    for path in &payload.files {
        let rel = path.trim_start_matches('/');

        if rel.is_empty() {
            continue;
        }

        let bytes = client
            .get_bzz_path(bzz_ref, rel)
            .await
            .with_context(|| format!("fetching {rel} from bzz {bzz_ref}"))?;
        let dest = out_dir.join(rel);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }

        std::fs::write(&dest, &bytes).with_context(|| format!("writing {}", dest.display()))?;
    }

    let manifest = bundle::check_layout(out_dir)
        .with_context(|| format!("re-validating bundle at {}", out_dir.display()))?;

    Ok(ResolveResult {
        manifest,
        bzz_ref: ensure_0x_prefix(bzz_ref),
        feed_index: index,
        published_at: payload.published_at,
        out_dir: out_dir.to_path_buf(),
    })
}

/// Walk every feed entry for `(owner, name)` and decode each payload.
/// Returned newest-first. Backs `dappswarm info`.
pub async fn list_feed(
    client: &Client,
    owner_eoa: &[u8; 20],
    name: &str,
) -> Result<Vec<(u64, FeedPayload)>> {
    let latest = feed::find_latest(client, owner_eoa, name)
        .await
        .with_context(|| format!("scanning feed for {name}"))?;
    let Some(FeedHit {
        index: latest_index,
        ..
    }) = latest
    else {
        return Ok(Vec::new());
    };

    let mut out = Vec::with_capacity((latest_index as usize) + 1);

    for index in 0..=latest_index {
        let Some(bytes) = feed::probe_payload(client, owner_eoa, name, index)
            .await
            .with_context(|| format!("reading feed entry {name} @ {index}"))?
        else {
            continue;
        };

        match serde_json::from_slice::<FeedPayload>(&bytes) {
            Ok(p) => out.push((index, p)),
            Err(e) => {
                return Err(anyhow!(
                    "feed payload at {name} @ {index} is not valid JSON: {e}"
                ));
            }
        }
    }

    out.reverse();

    Ok(out)
}

fn strip_0x(s: &str) -> &str {
    s.strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s)
}

fn ensure_0x_prefix(s: &str) -> String {
    if s.starts_with("0x") || s.starts_with("0X") {
        s.to_string()
    } else {
        format!("0x{s}")
    }
}
