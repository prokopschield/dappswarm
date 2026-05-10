//! CLI surface for `dappswarm`.
//!
//! Subcommands:
//! - `doctor`  — verify the configured antd is reachable; print the
//!   gateway version, API version, and configured postage batch id.
//! - `publish` — pack + upload + sign a new feed entry.
//! - `resolve` — find latest SOC for `(name, owner)` and unpack into a directory.
//! - `info`    — list every known feed entry for `(name, owner)`.
//! - `install` — resolve, then `docker compose up -d`.

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};

use crate::install;
use crate::publish;
use crate::resolve;
use crate::swarm::{Client, DEFAULT_GATEWAY};

#[derive(Debug, Parser)]
#[command(
    name = "dappswarm",
    version,
    about = "Publish and install Dappnode packages over Swarm."
)]
pub struct Cli {
    /// antd gateway base URL (default: <http://127.0.0.1:1633>).
    #[arg(long, env = "DAPPSWARM_GATEWAY", global = true)]
    pub gateway: Option<String>,

    /// Postage batch ID for uploads (32-byte hex, no `0x` prefix).
    #[arg(long, env = "STAMP_BATCH_ID", global = true)]
    pub postage_batch: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Report antd reachability and configuration.
    Doctor,
    /// Publish a Dappnode package directory to Swarm.
    ///
    /// Requires `DAPPSWARM_KEY` (32-byte hex secp256k1 secret, with or
    /// without `0x` prefix) in the environment.
    Publish {
        /// Path to the package directory.
        path: std::path::PathBuf,
    },
    /// Resolve a package name to an on-disk bundle.
    Resolve {
        /// Package name (e.g. hello.dnp.dappnode.eth).
        name: String,
        /// Owner EOA hex (20 bytes, with or without `0x`).
        #[arg(long)]
        owner: String,
        /// Output directory.
        #[arg(long)]
        out: std::path::PathBuf,
    },
    /// Resolve, unpack, `docker load` any image archives, and `docker compose up -d`.
    Install {
        name: String,
        #[arg(long)]
        owner: String,
        /// Optional working directory. Defaults to a fresh tempdir.
        #[arg(long)]
        data_dir: Option<std::path::PathBuf>,
    },
    /// List known feed indices for a package, newest first.
    Info {
        name: String,
        #[arg(long)]
        owner: String,
    },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let gateway = cli.gateway.unwrap_or_else(|| DEFAULT_GATEWAY.to_string());
    let mut client = Client::new(gateway.clone());

    if let Some(batch) = cli.postage_batch.as_ref() {
        client = client.with_postage_batch(batch.clone());
    }

    match cli.command {
        Command::Doctor => doctor(&client).await,
        Command::Publish { path } => publish_cmd(&client, &path).await,
        Command::Resolve { name, owner, out } => resolve_cmd(&client, &name, &owner, &out).await,
        Command::Info { name, owner } => info_cmd(&client, &name, &owner).await,
        Command::Install {
            name,
            owner,
            data_dir,
        } => install::run(&client, &name, &owner, data_dir.as_deref()).await,
    }
}

async fn doctor(client: &Client) -> Result<()> {
    let health = client
        .health()
        .await
        .with_context(|| format!("failed to reach antd at {}", client.base()))?;

    println!("antd:           {}", client.base());
    println!("status:         {}", health.status);
    println!("agent:          {}", health.version);
    println!("api version:    {}", health.api_version);

    match client.postage_batch() {
        Some(batch) => println!("postage batch:  {batch}"),
        None => println!("postage batch:  (not configured — uploads will fail)"),
    }

    println!("ok");

    Ok(())
}

async fn publish_cmd(client: &Client, path: &std::path::Path) -> Result<()> {
    let secret = load_signing_key()?;
    let result = publish::run(client, &secret, path).await?;

    println!("name:           {}", result.manifest_name);
    println!("version:        {}", result.manifest_version);
    println!("bzz ref:        {}", result.bzz_ref);
    println!("owner:          0x{}", hex::encode(result.soc.owner_eoa));
    println!("feed index:     {}", result.feed_index);
    println!("soc address:    0x{}", hex::encode(result.soc.address));

    Ok(())
}

async fn resolve_cmd(
    client: &Client,
    name: &str,
    owner_hex: &str,
    out: &std::path::Path,
) -> Result<()> {
    let owner = parse_owner(owner_hex)?;
    let result = resolve::run(client, &owner, name, out).await?;

    println!("name:           {}", result.manifest.name);
    println!("version:        {}", result.manifest.version);
    println!("feed index:     {}", result.feed_index);
    println!("bzz ref:        {}", result.bzz_ref);
    println!("published_at:   {}", result.published_at);
    println!("out:            {}", result.out_dir.display());
    println!("ok");

    Ok(())
}

async fn info_cmd(client: &Client, name: &str, owner_hex: &str) -> Result<()> {
    let owner = parse_owner(owner_hex)?;
    let entries = resolve::list_feed(client, &owner, name).await?;

    if entries.is_empty() {
        println!(
            "no feed entries for {name} under owner 0x{}",
            hex::encode(owner)
        );
        return Ok(());
    }

    println!("index    version    published      bzz_ref");

    for (index, payload) in entries {
        println!(
            "{:<8} {:<10} {:<14} {}",
            index, payload.version, payload.published_at, payload.bzz_ref
        );
    }

    Ok(())
}

/// Read the publisher's signing key from `DAPPSWARM_KEY`. Accepts hex
/// with or without `0x` prefix.
fn load_signing_key() -> Result<[u8; 32]> {
    let raw = std::env::var("DAPPSWARM_KEY")
        .map_err(|_| anyhow!("DAPPSWARM_KEY env var is required for publish"))?;
    let stripped = raw.trim().trim_start_matches("0x").trim_start_matches("0X");

    if stripped.len() != 64 {
        return Err(anyhow!(
            "DAPPSWARM_KEY must be 32 bytes (64 hex chars); got {}",
            stripped.len()
        ));
    }

    let mut out = [0u8; 32];
    hex::decode_to_slice(stripped, &mut out).context("DAPPSWARM_KEY is not valid hex")?;

    Ok(out)
}

fn parse_owner(s: &str) -> Result<[u8; 20]> {
    let stripped = s.trim().trim_start_matches("0x").trim_start_matches("0X");

    if stripped.len() != 40 {
        return Err(anyhow!(
            "owner must be 20 bytes (40 hex chars); got {}",
            stripped.len()
        ));
    }

    let mut out = [0u8; 20];
    hex::decode_to_slice(stripped, &mut out).context("owner is not valid hex")?;

    Ok(out)
}
