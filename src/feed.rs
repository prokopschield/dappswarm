//! Sequence-indexed Swarm Feeds, layered on Single-Owner Chunks.
//!
//! Conventions used by `dappswarm`:
//!
//! - `topic = keccak256("dappswarm:" || package_name)`
//! - `id_n  = keccak256(topic || u64_be(index))`
//! - SOC address `= keccak256(id_n || owner_eoa)`
//!
//! The publisher writes successive SOCs at `index = 0, 1, 2, …`. Readers
//! find the latest by exponential probe (1, 2, 4, 8, …) followed by
//! binary search across the gap between the last hit and the first miss.

use ant_crypto::{keccak256, soc_valid};
use thiserror::Error;

use crate::soc::{self, Soc, SocError, payload_from_wire, soc_address};
use crate::swarm::{Client, SwarmError};

pub const TOPIC_PREFIX: &str = "dappswarm:";
/// Cap on the exponential probe so a missing feed can't drive the
/// probe loop into hundreds of round-trips. `2^32` indices is far
/// more than any real package will ever ship.
const MAX_PROBE_LOG2: u32 = 32;

#[derive(Debug, Error)]
pub enum FeedError {
    #[error(transparent)]
    Swarm(#[from] SwarmError),

    #[error(transparent)]
    Soc(#[from] SocError),

    #[error("retrieved chunk failed SOC validation at address {0}")]
    InvalidSoc(String),
}

/// `topic = keccak256("dappswarm:" || package_name)`.
pub fn topic(package_name: &str) -> [u8; 32] {
    let mut buf = Vec::with_capacity(TOPIC_PREFIX.len() + package_name.len());

    buf.extend_from_slice(TOPIC_PREFIX.as_bytes());
    buf.extend_from_slice(package_name.as_bytes());

    keccak256(&buf)
}

/// `id_n = keccak256(topic || u64_be(index))`.
pub fn id_at(topic: &[u8; 32], index: u64) -> [u8; 32] {
    let mut buf = [0u8; 32 + 8];

    buf[..32].copy_from_slice(topic);
    buf[32..].copy_from_slice(&index.to_be_bytes());

    keccak256(&buf)
}

/// Sign a feed payload at `(name, index)` and upload it through `client`.
/// Returns the SOC address for logging.
pub async fn write(
    client: &Client,
    secret: &[u8; 32],
    name: &str,
    index: u64,
    payload: &[u8],
) -> Result<Soc, FeedError> {
    let topic = topic(name);
    let id = id_at(&topic, index);
    let envelope = soc::build(secret, id, payload)?;

    let owner_hex = hex::encode(envelope.owner_eoa);
    let id_hex = hex::encode(envelope.id);

    client
        .post_soc(
            &owner_hex,
            &id_hex,
            &envelope.signature,
            envelope.inner_cac.clone(),
        )
        .await?;

    Ok(envelope)
}

/// Lookup result for `find_latest`.
#[derive(Debug, Clone)]
pub struct FeedHit {
    pub index: u64,
    pub payload: Vec<u8>,
}

/// Find the largest index `n` such that the SOC at `(name, n)` exists,
/// owned by `owner_eoa`. Returns `None` when no feed entry exists.
///
/// Strategy: probe `0, 1, 2, 4, 8, …` until either the first probe
/// misses (no feed at all) or a probe misses after one or more hits;
/// then binary-search `[last_hit + 1, first_miss)` for the cliff. One
/// round-trip per probe; payload at the latest hit is fetched once.
pub async fn find_latest(
    client: &Client,
    owner_eoa: &[u8; 20],
    name: &str,
) -> Result<Option<FeedHit>, FeedError> {
    let topic = topic(name);

    // Probe index 0.
    let Some(zero_payload) = probe(client, &topic, owner_eoa, 0).await? else {
        return Ok(None);
    };

    let mut last_hit: u64 = 0;
    let mut last_hit_payload = zero_payload;
    let mut step: u64 = 1;
    let mut first_miss: Option<u64> = None;

    for _ in 0..MAX_PROBE_LOG2 {
        let probe_idx = last_hit + step;

        if let Some(payload) = probe(client, &topic, owner_eoa, probe_idx).await? {
            last_hit = probe_idx;
            last_hit_payload = payload;
            step = step.saturating_mul(2);
        } else {
            first_miss = Some(probe_idx);

            break;
        }
    }

    // Probe ran to its cap without finding a miss — accept the highest hit.
    let Some(first_miss) = first_miss else {
        return Ok(Some(FeedHit {
            index: last_hit,
            payload: last_hit_payload,
        }));
    };

    // Binary search: [last_hit + 1, first_miss). Loop invariant: every
    // index ≤ last_hit hits, every index ≥ first_miss misses.
    let mut lo = last_hit + 1;
    let mut hi = first_miss;

    while lo < hi {
        let mid = lo + (hi - lo) / 2;

        match probe(client, &topic, owner_eoa, mid).await? {
            Some(payload) => {
                last_hit = mid;
                last_hit_payload = payload;
                lo = mid + 1;
            }
            None => {
                hi = mid;
            }
        }
    }

    Ok(Some(FeedHit {
        index: last_hit,
        payload: last_hit_payload,
    }))
}

/// Read the SOC payload for a specific `(name, index)` entry. Returns
/// `None` if the entry doesn't exist. Used by `dappswarm info` to
/// enumerate every published version.
pub async fn probe_payload(
    client: &Client,
    owner_eoa: &[u8; 20],
    name: &str,
    index: u64,
) -> Result<Option<Vec<u8>>, FeedError> {
    let topic = topic(name);
    probe(client, &topic, owner_eoa, index).await
}

/// Single-index existence probe. Returns the payload bytes on hit and
/// `None` on a clean 404.
async fn probe(
    client: &Client,
    topic: &[u8; 32],
    owner_eoa: &[u8; 20],
    index: u64,
) -> Result<Option<Vec<u8>>, FeedError> {
    let id = id_at(topic, index);
    let address = soc_address(&id, owner_eoa);
    let address_hex = hex::encode(address);

    match client.get_chunk(&address_hex).await {
        Ok(bytes) => {
            let wire = bytes.to_vec();

            if !soc_valid(&address, &wire) {
                return Err(FeedError::InvalidSoc(address_hex));
            }

            let payload = payload_from_wire(&wire)?.to_vec();

            Ok(Some(payload))
        }
        Err(SwarmError::NotFound) => Ok(None),
        Err(e) => Err(FeedError::Swarm(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_is_deterministic() {
        let a = topic("hello.dnp.dappnode.eth");
        let b = topic("hello.dnp.dappnode.eth");
        assert_eq!(a, b);
    }

    #[test]
    fn topic_distinguishes_names() {
        let a = topic("hello.dnp.dappnode.eth");
        let b = topic("world.dnp.dappnode.eth");
        assert_ne!(a, b);
    }

    #[test]
    fn id_at_indices_differ() {
        let t = topic("x");
        assert_ne!(id_at(&t, 0), id_at(&t, 1));
    }
}
