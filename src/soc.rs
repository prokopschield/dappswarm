//! Single-Owner Chunk builder.
//!
//! Mirrors the wire layout validated by `ant_crypto::soc::soc_valid`:
//! `id (32) || sig (65) || span (8 LE) || payload (≤ 4088)`.
//!
//! The signing prehash is EIP-191(`id || inner_cac_address`) where
//! `inner_cac_address = bmt_hash_with_span(span, payload)` and
//! `span = u64::to_le_bytes(payload.len() as u64)`. The SOC address used
//! for storage and lookup is `keccak256(id || owner_eoa)`.

use ant_crypto::{
    CryptoError, cac_new, ethereum_address_from_public_key, keccak256, sign_handshake_data,
};
use k256::ecdsa::{SigningKey, VerifyingKey};
use thiserror::Error;

pub const SOC_ID_SIZE: usize = 32;
pub const SOC_SIG_SIZE: usize = 65;
pub const SOC_SPAN_SIZE: usize = 8;
pub const SOC_HEADER_SIZE: usize = SOC_ID_SIZE + SOC_SIG_SIZE;
pub const SOC_MAX_PAYLOAD: usize = 4096;

#[derive(Debug, Error)]
pub enum SocError {
    #[error("payload too large: {0} bytes (max 4096)")]
    PayloadTooLarge(usize),

    #[error("failed to build inner CAC")]
    CacBuild,

    #[error("invalid wire layout: {0}")]
    InvalidWire(&'static str),

    #[error("crypto: {0}")]
    Crypto(#[from] CryptoError),
}

/// A signed single-owner-chunk ready to upload via
/// `POST /soc/{owner}/{id}`.
#[derive(Debug, Clone)]
pub struct Soc {
    /// Storage address: `keccak256(id || owner_eoa)`.
    pub address: [u8; 32],
    /// Full wire bytes: `id (32) || sig (65) || span (8) || payload`.
    pub wire: Vec<u8>,
    /// Just the inner CAC: `span (8) || payload`. This is what the
    /// gateway's `POST /soc/{owner}/{id}` endpoint expects in the body.
    pub inner_cac: Vec<u8>,
    /// Recoverable secp256k1 signature over `keccak256(id || inner_cac_addr)`.
    pub signature: [u8; SOC_SIG_SIZE],
    /// Owner Ethereum address, derived from the signing key.
    pub owner_eoa: [u8; 20],
    /// SOC id (the second path segment of `POST /soc/{owner}/{id}`).
    pub id: [u8; SOC_ID_SIZE],
}

/// Build and sign a SOC envelope around `payload` for the chunk
/// addressed by `id` under the secp256k1 owner key `secret`.
pub fn build(secret: &[u8; 32], id: [u8; SOC_ID_SIZE], payload: &[u8]) -> Result<Soc, SocError> {
    if payload.len() > SOC_MAX_PAYLOAD {
        return Err(SocError::PayloadTooLarge(payload.len()));
    }

    let (inner_addr, inner_cac) = cac_new(payload).ok_or(SocError::CacBuild)?;

    // bee's `soc.SOC.Sign` calls `signer.Sign(hash(id, inner_addr))`.
    // `signer.Sign` wraps its input with the EIP-191 prefix, so the
    // SOC signature is over `EIP-191(keccak256(id || inner_addr))`.
    // `sign_handshake_data` does the EIP-191 wrapping for us; we just
    // hand it the 32-byte prehash. `soc_valid` recovers with the same
    // shape (see `ant_crypto::soc::soc_valid`).
    let mut prehash_input = [0u8; SOC_ID_SIZE + 32];
    prehash_input[..SOC_ID_SIZE].copy_from_slice(&id);
    prehash_input[SOC_ID_SIZE..].copy_from_slice(&inner_addr);
    let prehash = keccak256(&prehash_input);
    let signature = sign_handshake_data(secret, &prehash)?;

    let sk = SigningKey::from_bytes(secret.into()).map_err(CryptoError::Sign)?;
    let vk = VerifyingKey::from(&sk);
    let owner_eoa = ethereum_address_from_public_key(&vk);

    let mut addr_input = [0u8; SOC_ID_SIZE + 20];
    addr_input[..SOC_ID_SIZE].copy_from_slice(&id);
    addr_input[SOC_ID_SIZE..].copy_from_slice(&owner_eoa);
    let address = keccak256(&addr_input);

    let mut wire = Vec::with_capacity(SOC_HEADER_SIZE + inner_cac.len());
    wire.extend_from_slice(&id);
    wire.extend_from_slice(&signature);
    wire.extend_from_slice(&inner_cac);

    Ok(Soc {
        address,
        wire,
        inner_cac,
        signature,
        owner_eoa,
        id,
    })
}

/// SOC address derivation: `keccak256(id || owner_eth_address)`.
#[must_use]
pub fn soc_address(id: &[u8; SOC_ID_SIZE], owner_eoa: &[u8; 20]) -> [u8; 32] {
    let mut buf = [0u8; SOC_ID_SIZE + 20];

    buf[..SOC_ID_SIZE].copy_from_slice(id);
    buf[SOC_ID_SIZE..].copy_from_slice(owner_eoa);

    keccak256(&buf)
}

/// Parse a SOC wire and return its payload bytes (without span). The
/// caller is responsible for first verifying the wire with
/// `ant_crypto::soc_valid`.
pub fn payload_from_wire(wire: &[u8]) -> Result<&[u8], SocError> {
    if wire.len() < SOC_HEADER_SIZE + SOC_SPAN_SIZE {
        return Err(SocError::InvalidWire("too short"));
    }

    let span_off = SOC_HEADER_SIZE;
    let payload_off = span_off + SOC_SPAN_SIZE;
    let span_bytes: [u8; 8] = wire[span_off..payload_off].try_into().unwrap();
    let span = u64::from_le_bytes(span_bytes) as usize;

    if wire.len() < payload_off + span {
        return Err(SocError::InvalidWire("span exceeds wire"));
    }

    Ok(&wire[payload_off..payload_off + span])
}

#[cfg(test)]
mod tests {
    use super::*;
    use ant_crypto::soc_valid;

    fn fixed_secret() -> [u8; 32] {
        let mut s = [0u8; 32];
        s[31] = 1; // any non-zero scalar < n
        s
    }

    #[test]
    fn build_round_trip_validates() {
        let secret = fixed_secret();
        let id = [7u8; SOC_ID_SIZE];
        let payload = b"hello dappswarm";
        let soc = build(&secret, id, payload).expect("build");

        // soc_valid is the same check ant-gateway runs server-side.
        assert!(soc_valid(&soc.address, &soc.wire));
        assert_eq!(soc.address, soc_address(&id, &soc.owner_eoa));

        let recovered = payload_from_wire(&soc.wire).expect("payload");

        assert_eq!(recovered, payload);
    }

    #[test]
    fn distinct_ids_yield_distinct_addresses() {
        let secret = fixed_secret();
        let a = build(&secret, [0u8; 32], b"x").unwrap();

        let mut id_b = [0u8; 32];
        id_b[31] = 1;

        let b = build(&secret, id_b, b"x").unwrap();

        assert_ne!(a.address, b.address);
        assert_eq!(a.owner_eoa, b.owner_eoa);
    }

    #[test]
    fn payload_too_large_rejected() {
        let secret = fixed_secret();
        let big = vec![0u8; SOC_MAX_PAYLOAD + 1];

        assert!(matches!(
            build(&secret, [0u8; 32], &big),
            Err(SocError::PayloadTooLarge(_))
        ));
    }
}
