#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [ ! -d node_modules ]; then
  npm install
fi

npx tsx scripts/refresh-metadata.ts || echo "warning: refresh-metadata failed; using existing src/lib/metadata.ts and src/lib/feedData.ts"

npm run build

cp bundle/dappnode_package.json dist/
cp bundle/docker-compose.yml dist/

echo "demo bundle ready at $(pwd)/dist (size: $(du -sh dist | cut -f1))"
