//! HTTP client to a local `antd` gateway.
//!
//! `antd` exposes a bee-shaped HTTP API on `127.0.0.1:1633` by default.
//! This module wraps the four endpoints `dappswarm` needs:
//! `GET /health`, `POST /bzz` (tar collection), `GET /bzz/<ref>/<path>`,
//! and the SOC upload + chunk read endpoints used by [`crate::feed`].
//!
//! Errors are surfaced as a single typed enum so callers can react to
//! `NotFound` (used by feed lookup) without parsing strings.

use std::time::Duration;

use bytes::Bytes;
use reqwest::{Client as HttpClient, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Default antd gateway address.
pub const DEFAULT_GATEWAY: &str = "http://127.0.0.1:1633";

/// HTTP client to a single antd gateway.
#[derive(Debug, Clone)]
pub struct Client {
    base: String,
    http: HttpClient,
    postage_batch: Option<String>,
}

#[derive(Debug, Error)]
pub enum SwarmError {
    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("antd returned {status}: {body}")]
    Status { status: StatusCode, body: String },

    #[error("not found")]
    NotFound,

    #[error("postage batch id is required for uploads but none was configured")]
    MissingPostage,

    #[error("response body decode error: {0}")]
    Decode(String),
}

#[derive(Debug, Deserialize)]
pub struct HealthBody {
    pub status: String,
    pub version: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UploadResponse {
    pub reference: String,
}

impl Client {
    /// Construct a client targeting `base` (e.g. `http://127.0.0.1:1633`).
    pub fn new(base: impl Into<String>) -> Self {
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(900))
            .build()
            .expect("reqwest client");

        Self {
            base: base.into(),
            http,
            postage_batch: None,
        }
    }

    /// Attach a postage batch ID. Required for `POST /bzz` and `POST /soc`.
    #[must_use]
    pub fn with_postage_batch(mut self, batch: impl Into<String>) -> Self {
        self.postage_batch = Some(batch.into());
        self
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn postage_batch(&self) -> Option<&str> {
        self.postage_batch.as_deref()
    }

    /// `GET /health`. Returns the antd version + API version.
    pub async fn health(&self) -> Result<HealthBody, SwarmError> {
        let url = format!("{}/health", self.base);
        let resp = self.http.get(url).send().await?;
        let status = resp.status();

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(SwarmError::Status { status, body });
        }

        resp.json::<HealthBody>()
            .await
            .map_err(|e| SwarmError::Decode(e.to_string()))
    }

    /// `POST /bzz` with a tar collection body.
    ///
    /// The tar must be uncompressed; the gateway streams it through bee's
    /// `tar`-collection upload path which builds a Mantaray manifest.
    pub async fn post_bzz_tar(
        &self,
        tar_bytes: Vec<u8>,
        index_document: &str,
    ) -> Result<String, SwarmError> {
        let batch = self
            .postage_batch
            .as_deref()
            .ok_or(SwarmError::MissingPostage)?;

        let url = format!("{}/bzz", self.base);
        let resp = self
            .http
            .post(url)
            .header("content-type", "application/x-tar")
            .header("swarm-collection", "true")
            .header("swarm-index-document", index_document)
            .header("swarm-postage-batch-id", batch)
            .body(tar_bytes)
            .send()
            .await?;

        let status = resp.status();

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();

            return Err(SwarmError::Status { status, body });
        }

        let parsed: UploadResponse = resp
            .json()
            .await
            .map_err(|e| SwarmError::Decode(e.to_string()))?;

        Ok(parsed.reference)
    }

    /// `GET /bzz/<ref>/<path>` — manifest walk + body fetch.
    pub async fn get_bzz_path(&self, reference: &str, path: &str) -> Result<Bytes, SwarmError> {
        let url = format!("{}/bzz/{}/{}", self.base, reference, path);
        let resp = self.http.get(url).send().await?;
        let status = resp.status();

        if status == StatusCode::NOT_FOUND {
            return Err(SwarmError::NotFound);
        }

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();

            return Err(SwarmError::Status { status, body });
        }

        Ok(resp.bytes().await?)
    }

    /// `GET /bzz/<ref>` — fetch the bundle root (returns the tar bytes
    /// when the manifest is a collection).
    pub async fn get_bzz_root(&self, reference: &str) -> Result<Bytes, SwarmError> {
        let url = format!("{}/bzz/{}", self.base, reference);
        let resp = self.http.get(url).send().await?;
        let status = resp.status();

        if status == StatusCode::NOT_FOUND {
            return Err(SwarmError::NotFound);
        }

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();

            return Err(SwarmError::Status { status, body });
        }

        Ok(resp.bytes().await?)
    }

    /// `GET /chunks/<addr>` — raw chunk bytes (CAC or SOC).
    pub async fn get_chunk(&self, address_hex: &str) -> Result<Bytes, SwarmError> {
        let url = format!("{}/chunks/{}", self.base, address_hex);
        let resp = self.http.get(url).send().await?;
        let status = resp.status();

        if status == StatusCode::NOT_FOUND {
            return Err(SwarmError::NotFound);
        }

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();

            return Err(SwarmError::Status { status, body });
        }

        Ok(resp.bytes().await?)
    }

    /// `POST /soc/<owner>/<id>` — upload a Single-Owner Chunk.
    ///
    /// Body is the inner CAC payload (`span(8) || data`). The signature
    /// is supplied via the `swarm-soc-signature` header. `owner_hex`
    /// is the 20-byte Ethereum address of the signing key (40 hex
    /// chars); `id_hex` is the 32-byte SOC id (64 hex chars). Neither
    /// expects an `0x` prefix.
    pub async fn post_soc(
        &self,
        owner_hex: &str,
        id_hex: &str,
        signature: &[u8; 65],
        inner_cac: Vec<u8>,
    ) -> Result<String, SwarmError> {
        let batch = self
            .postage_batch
            .as_deref()
            .ok_or(SwarmError::MissingPostage)?;

        let url = format!("{}/soc/{}/{}", self.base, owner_hex, id_hex);
        let resp = self
            .http
            .post(url)
            .header("content-type", "application/octet-stream")
            .header("swarm-soc-signature", hex::encode(signature))
            .header("swarm-postage-batch-id", batch)
            .body(inner_cac)
            .send()
            .await?;

        let status = resp.status();

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();

            return Err(SwarmError::Status { status, body });
        }

        let parsed: UploadResponse = resp
            .json()
            .await
            .map_err(|e| SwarmError::Decode(e.to_string()))?;

        Ok(parsed.reference)
    }
}
