#!/usr/bin/env bash
# Fetch a small public Dappnode package from github.com/dappnode and
# convert it into a `dappswarm`-shaped publishable bundle directory.
#
# The DNP format on disk is essentially what dappswarm expects:
# `dappnode_package.json`, `docker-compose.yml`, and image tarballs
# saved with `docker save | xz`. Most public DNPs build their image
# from a Dockerfile in the repo, so we still need to `docker build`
# locally before saving.
#
# Usage:
#   ./scripts/fetch-real-dnp.sh <repo-name> [tag]
#
# Example:
#   ./scripts/fetch-real-dnp.sh DAppNodePackage-helloworld v0.1.0
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <github-repo-name> [tag]"
  exit 1
fi

REPO="$1"
TAG="${2:-main}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/fixtures/$REPO"
URL="https://github.com/dappnode/$REPO.git"

if [[ -d "$DEST" ]]; then
  echo "$DEST already exists; refusing to clobber"
  exit 1
fi

git clone --depth 1 --branch "$TAG" "$URL" "$DEST"

if [[ ! -f "$DEST/dappnode_package.json" ]]; then
  echo "no dappnode_package.json at repo root — not a DNP layout"
  exit 1
fi

NAME="$(jq -r '.name'    "$DEST/dappnode_package.json")"
VERSION="$(jq -r '.version' "$DEST/dappnode_package.json")"
IMAGE_TAG="${NAME}:${VERSION}"
TAR="$DEST/${NAME}_${VERSION}.tar"

echo "==> docker build $IMAGE_TAG"
docker build -t "$IMAGE_TAG" "$DEST"

echo "==> docker save $IMAGE_TAG -> $TAR"
docker save "$IMAGE_TAG" -o "$TAR"

echo "==> xz -f -T0 $TAR"
xz -f -T0 "$TAR"

echo "ok: $TAR.xz"
ls -lh "$DEST"
