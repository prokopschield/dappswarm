# dappswarm

Publish and install [Dappnode](https://dappnode.com/) packages over
[Swarm](https://www.ethswarm.org/), with versions tracked through
sequence-indexed Swarm Feeds.

ETHPrague Hackathon entry for the **Dappnode Packages on Swarm** bounty.

```
dappswarm publish ./fixtures/hello-dnp/
# → bzz: 0x… | feed hello.dnp.dappnode.eth @ index 0 | owner 0x…

dappswarm install hello.dnp.dappnode.eth --owner 0x…
# → resolve, unpack, docker load, docker compose up -d
```

## What it is

`dappswarm` packs a Dappnode package directory (`dappnode_package.json`,
`docker-compose.yml`, optional image `*.tar.xz` archives) into a single
Mantaray-manifest upload on Swarm and stamps each release into a Swarm
Feed indexed by package name. Resolvers find the latest release with one
feed lookup, fetch each declared file, and bring the containers up, with
no centralised registry and no IPFS hop.

The list of bundle-relative file paths is embedded in the feed payload
itself, so resolve works against any bee-compatible gateway: no
manifest-listing endpoint required.

The storage backend is a local
[`antd`](https://github.com/prokopschield/ant) (Rust Swarm node), patched
to expose `POST /soc/{owner}/{id}` so feed updates can be pushed without
running a separate Bee.

## Components

| Crate / module | Job |
| --- | --- |
| `swarm`   | Bee-shaped HTTP client over `antd` (`/health`, `/bzz`, `/chunks`, `/soc`). |
| `soc`     | Build a signed Single-Owner-Chunk envelope around a payload. |
| `feed`    | Topic = `keccak256("dappswarm:" \|\| name)`; entries written at `index = 0, 1, 2, …`; lookup by exponential probe + binary search. |
| `bundle`  | DNP layout: pack/unpack uncompressed tar, validate manifest. |
| `publish` | manifest → tar → `POST /bzz` → next index → sign + `POST /soc`. |
| `resolve` | `find_latest` SOC → decode JSON → walk manifest → write files. |
| `install` | Resolve + `docker load *.tar.xz` + `docker compose up -d`. |

## Prereqs

1. `antd` from `prokopschield/ant` (with the SOC patch applied) running
   on `127.0.0.1:1633`. `cargo run -p antd` after configuring its
   wallet + postage env.
2. A funded postage batch on Gnosis. Export `STAMP_BATCH_ID`.
3. A throwaway secp256k1 key for feed signing. Export `DAPPSWARM_KEY`
   (32-byte hex). **Do not** reuse the postage signer.

```sh
export DAPPSWARM_GATEWAY=http://127.0.0.1:1633   # optional; default
export STAMP_BATCH_ID=0x…                        # funded batch
export DAPPSWARM_KEY=0x…                         # feed signer
```

## Commands

```
dappswarm doctor
  Probe antd; print version, API version, configured batch.

dappswarm publish <dir>
  Pack <dir>, upload bundle, sign + push next feed entry.
  Reads DAPPSWARM_KEY from the env. Prints bzz ref + owner + index.

dappswarm resolve <name> --owner 0x… --out <dir>
  Find latest feed entry, walk the manifest, write files into <dir>.

dappswarm install <name> --owner 0x… [--data-dir <dir>]
  Resolve + docker load every *.tar(.xz) at the bundle root +
  docker compose -f <dir>/docker-compose.yml up -d.

dappswarm info <name> --owner 0x…
  List every published version, newest first.
```

## Demo

The bundled `fixtures/hello-dnp` is a 3-file bundle that pulls
`nginx:alpine` from Docker Hub and bind-mounts the fixture's
`index.html` into it; no image tarball required.

```sh
make doctor                 # antd reachable, batch configured
make publish                # → prints owner address; export it
export DAPPSWARM_OWNER=0x…  # use the address printed above
make install                # resolve → docker compose up -d

curl -fsS http://localhost:8080
# Hello from Swarm!
```

To bundle a Docker image into the package itself, drop a `*.tar.xz`
(produced by `docker save | xz`) into the bundle root and reference its
loaded image name in `docker-compose.yml`. `dappswarm install` will run
`docker load` on every archive at the bundle root before bringing the
compose project up.

### Demo site

`demo/` is a SvelteKit app that explains dappswarm and is itself
deployed via dappswarm. It lives behind nginx on port `8081` once
installed.

```sh
make demo-build             # compile svelte → demo/dist/ (~550 KiB)
make demo-publish           # publishes demo/dist/; prints owner + index
export DAPPSWARM_OWNER=0x…
make demo-install           # resolve → docker compose up -d on :8081

curl -fsS http://localhost:8081 | head -1
# <!doctype html>
```

The page reads its own feed at build time, so when served from a
freshly-published bundle the footer prints the live `bzz:` ref, the
"This page's feed" section lists every prior version, and the
"Try it live" widget recomputes the SOC address client-side. See
`demo/README` (the page itself) for development notes.

## Trust model

The publisher's secp256k1 EOA is the trust anchor. Resolvers must
supply `--owner 0x…` to look up a feed; the Swarm SOC validation
guarantees the recovered signer matches that owner. ENS-mediated owner
discovery is future work.

## Layout assumptions

- `dappnode_package.json` and `docker-compose.yml` are required.
- Any `*.tar` / `*.tar.xz` at the bundle root is treated as a Docker
  image archive and `docker load`ed.
- The IPFS-shaped `image.hash` field in the manifest is ignored;
  Swarm's Mantaray manifest gives us bundle-level integrity for free.

## Troubleshooting

| Symptom | Likely cause |
| --- | --- |
| `doctor`: `failed to reach antd` | antd not running on the configured gateway. |
| `publish`: 503 with "uploads not configured" | `antd` started without `--postage-batch` / `--wallet-key`. |
| `publish`: 4xx on `POST /soc` | Bad signature recovery; check that the same `DAPPSWARM_KEY` produced the owner you expect. |
| `install`: `docker compose: command not found` | Compose v2 plugin missing. Install or fall back to `docker-compose`. |
