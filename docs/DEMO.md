# Demo script — 90 seconds

Live segment runs the tiny `hello-dnp` fixture; recorded segment shows
the same flow against a real public DNP from `github.com/dappnode`.

## Pre-stage (off camera)

- `antd` running on `127.0.0.1:1633`, postage batch funded, wallet
  configured. `dappswarm doctor` reports `ok`.
- `STAMP_BATCH_ID`, `DAPPSWARM_KEY` exported in the demo shell.
- `nginx:alpine` already pulled into the local Docker cache so the
  first install doesn't depend on Docker Hub round-trip latency.
- Real DNP recorded segment captured per `M8` in PLAN.md (bzz ref,
  owner, version) and ready to roll.

## Live narration (≈ 60 s)

1. **Frame** (5 s):
   > "Dappnode packages today live in registries — IPFS, GitHub, image
   > hubs. dappswarm puts them on Swarm with mutable feeds for
   > versions. End-to-end, censorship-resistant, no IPFS hop."

2. **Doctor** (5 s):
   ```sh
   dappswarm doctor
   ```
   Show antd reachable + postage batch attached.

3. **Publish** (15 s):
   ```sh
   dappswarm publish fixtures/hello-dnp
   ```
   Read out the printed `bzz ref`, `owner`, `feed index 0`. Note the
   feed address is **deterministic** from `(owner, package_name)`, so
   anyone with those two strings finds the latest version.

4. **Install** (20 s):
   ```sh
   export DAPPSWARM_OWNER=0x…   # from the publish output
   dappswarm install hello.dnp.dappnode.eth --owner $DAPPSWARM_OWNER
   ```
   Watch `docker compose up -d` stream through. Then:
   ```sh
   curl -fsS http://localhost:8080
   ```
   The page reads "Hello from Swarm!".

5. **Update** (15 s):
   Change the `index.html` body, bump the manifest, republish:
   ```sh
   sed -i 's/Hello from Swarm!/Hello — version 2!/' fixtures/hello-dnp/index.html
   sed -i 's/"version": "0.1.1"/"version": "0.1.2"/' fixtures/hello-dnp/dappnode_package.json
   dappswarm publish fixtures/hello-dnp           # → next feed index
   dappswarm install hello.dnp.dappnode.eth --owner $DAPPSWARM_OWNER
   curl -fsS http://localhost:8080                 # → updated body
   ```

## Recorded segment (≈ 25 s)

> "And the same flow works for a real-world DNP."

Roll capture: `dappswarm install <real-name> --owner <addr>` against a
public Dappnode package fetched from `github.com/dappnode` via
`scripts/fetch-real-dnp.sh`. Show containers up, hit the service.

## Closer (≈ 5 s)

> "Three changes from baseline Bee: SOC writes via a tiny ant-gateway
> patch, sequence-indexed feeds for versions, and a one-shot install
> command. Everything else is bee-compatible — bzz uploads, manifest
> walk, chunk reads."

## Failure modes (be ready to recover from)

- **Postage exhausted mid-publish**: swap `STAMP_BATCH_ID` to the
  backup batch and rerun. `dappswarm doctor` shows the configured one.
- **antd cold**: keep it warm; if it dies, `cargo run -p antd` and
  give it ~10 s before retrying.
- **Hotel Wi-Fi**: the tiny image is on the demo machine already; the
  real DNP segment is recorded for exactly this reason.
- **Container collision**: `docker rm -f dappswarm-hello` before the
  install if a stale container is still running.
