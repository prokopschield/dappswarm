//! `dappswarm install <name>`: resolve, then bring containers up.
//!
//! Pipeline:
//!   1. `resolve::run` into `data_dir` (or a tempdir if not provided).
//!   2. For every `*.tar` / `*.tar.xz` at the bundle root, run
//!      `docker load -i <file>`.
//!   3. `docker compose -f <dir>/docker-compose.yml up -d`.
//!
//! Each shell-out streams its child stdout/stderr through to ours so the
//! demo viewer can see exactly what Docker is doing. Failures surface
//! with the underlying command's exit code; we do not retry.

use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result, anyhow};
use tokio::process::Command;

use crate::bundle::{self, COMPOSE_FILENAME};
use crate::resolve;
use crate::swarm::Client;

pub async fn run(
    client: &Client,
    name: &str,
    owner_hex: &str,
    data_dir: Option<&Path>,
) -> Result<()> {
    let owner = parse_owner(owner_hex)?;

    let (out_dir, _keep_alive) = if let Some(p) = data_dir {
        (p.to_path_buf(), None)
    } else {
        let tmp = tempfile::tempdir().context("creating tempdir for install")?;

        (tmp.path().to_path_buf(), Some(tmp))
    };

    let resolved = resolve::run(client, &owner, name, &out_dir).await?;

    println!(
        "resolved {} v{} → {}",
        resolved.manifest.name,
        resolved.manifest.version,
        out_dir.display()
    );

    let images = bundle::image_tarballs(&out_dir).context("scanning bundle for image tarballs")?;

    for image in &images {
        println!("docker load -i {}", image.display());
        run_streaming("docker", &["load", "-i", &image.to_string_lossy()]).await?;
    }

    if images.is_empty() {
        println!("(no *.tar / *.tar.xz images at bundle root; relying on registry pulls)");
    }

    let compose_path = out_dir.join(COMPOSE_FILENAME);

    println!("docker compose -f {} up -d", compose_path.display());

    run_streaming(
        "docker",
        &["compose", "-f", &compose_path.to_string_lossy(), "up", "-d"],
    )
    .await?;

    println!("ok");

    Ok(())
}

async fn run_streaming(program: &str, args: &[&str]) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("spawning `{program} {}`", args.join(" ")))?;

    let status = child
        .wait()
        .await
        .with_context(|| format!("waiting on `{program} {}`", args.join(" ")))?;

    if !status.success() {
        return Err(anyhow!(
            "`{program} {}` exited with {}",
            args.join(" "),
            status
                .code()
                .map_or_else(|| "signal".into(), |c| c.to_string())
        ));
    }

    Ok(())
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
