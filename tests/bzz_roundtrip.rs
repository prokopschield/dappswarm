//! Integration test exercising the `POST /bzz` + `GET /bzz/<ref>/<path>`
//! round-trip against a running `antd` gateway with a configured
//! postage batch.
//!
//! Skipped by default. Run with:
//!
//! ```sh
//! STAMP_BATCH_ID=0x… cargo test --test bzz_roundtrip -- --ignored
//! ```

use dappswarm::bundle;
use dappswarm::swarm::Client;
use tempfile::TempDir;

fn fixture(dir: &std::path::Path) {
    std::fs::write(
        dir.join("dappnode_package.json"),
        r#"{"name": "rt.dnp.dappnode.eth", "version": "0.1.0"}"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("docker-compose.yml"),
        "version: '3'\nservices:\n  web:\n    image: nginx:alpine\n",
    )
    .unwrap();
}

#[tokio::test]
#[ignore = "requires running antd + STAMP_BATCH_ID env var"]
async fn bzz_roundtrip_against_local_antd() {
    let batch =
        std::env::var("STAMP_BATCH_ID").expect("STAMP_BATCH_ID must point to a funded batch");
    let gateway = std::env::var("DAPPSWARM_GATEWAY")
        .unwrap_or_else(|_| dappswarm::swarm::DEFAULT_GATEWAY.to_string());
    let client = Client::new(gateway).with_postage_batch(batch);

    client.health().await.expect("antd reachable");

    let src = TempDir::new().unwrap();
    fixture(src.path());

    let tar_bytes = bundle::pack(src.path()).unwrap();
    let reference = client
        .post_bzz_tar(tar_bytes.clone(), bundle::MANIFEST_FILENAME)
        .await
        .expect("upload bzz tar");

    let manifest_bytes = client
        .get_bzz_path(&reference, bundle::MANIFEST_FILENAME)
        .await
        .expect("fetch manifest");

    let original = std::fs::read(src.path().join(bundle::MANIFEST_FILENAME)).unwrap();
    assert_eq!(manifest_bytes.as_ref(), original.as_slice());
}
