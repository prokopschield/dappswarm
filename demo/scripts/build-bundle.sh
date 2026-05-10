#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [ ! -d node_modules ]; then
  npm install
fi

npm run build

cp bundle/dappnode_package.json dist/
cp bundle/docker-compose.yml dist/

echo "demo bundle ready at $(pwd)/dist (size: $(du -sh dist | cut -f1))"
