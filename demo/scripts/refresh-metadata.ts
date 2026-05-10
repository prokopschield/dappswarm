#!/usr/bin/env tsx
/**
 * Pre-build metadata generator. Walks the demo's own feed via antd /chunks
 * and writes src/lib/metadata.ts + src/lib/feedData.ts. Falls back to
 * placeholder constants when antd is unreachable so the page still renders.
 *
 * Mirrors src/feed.rs: topic = keccak256("dappswarm:" || name), id_n =
 * keccak256(topic || u64_be(index)), addr = keccak256(id_n || owner_eoa).
 *
 * Env:
 *   DAPPSWARM_GATEWAY (default http://127.0.0.1:1633)
 *   DAPPSWARM_DEMO_NAME (default dappswarm-demo)
 *   DAPPSWARM_DEMO_OWNER (40-hex EOA, no 0x prefix; required for live data)
 */

import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import sha3 from 'js-sha3';

const { keccak256 } = sha3;

const TOPIC_PREFIX = 'dappswarm:';
const MAX_PROBE_LOG2 = 32;

interface FeedPayload {
  version: string;
  ref: string;
  published_at: number;
  files?: string[];
}

interface FeedEntry {
  index: number;
  version: string;
  ref: string;
  publishedAt: number;
}

function unhex(s: string): Uint8Array {
  const v = s.toLowerCase().replace(/^0x/, '');
  const out = new Uint8Array(v.length / 2);

  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(v.slice(i * 2, i * 2 + 2), 16);
  }

  return out;
}

function hex(b: Uint8Array | ArrayBuffer): string {
  const arr = b instanceof Uint8Array ? b : new Uint8Array(b);

  return Array.from(arr, (x) => x.toString(16).padStart(2, '0')).join('');
}

function k(input: Uint8Array): Uint8Array {
  return new Uint8Array(keccak256.arrayBuffer(input));
}

function concat(a: Uint8Array, b: Uint8Array): Uint8Array {
  const out = new Uint8Array(a.length + b.length);

  out.set(a, 0);
  out.set(b, a.length);

  return out;
}

function u64BE(n: number): Uint8Array {
  const out = new Uint8Array(8);
  let big = BigInt(n);

  for (let i = 7; i >= 0; i--) {
    out[i] = Number(big & 0xffn);
    big >>= 8n;
  }

  return out;
}

function topic(name: string): Uint8Array {
  return k(new TextEncoder().encode(TOPIC_PREFIX + name));
}

function idAt(topicBytes: Uint8Array, index: number): Uint8Array {
  return k(concat(topicBytes, u64BE(index)));
}

function socAddress(idBytes: Uint8Array, owner: Uint8Array): Uint8Array {
  return k(concat(idBytes, owner));
}

/**
 * Strip the SOC envelope and return the inner CAC payload bytes.
 * Wire format: [id 32][sig 65][span 8][payload].
 */
function payloadFromWire(wire: Uint8Array): Uint8Array {
  const HEAD = 32 + 65 + 8;

  if (wire.length < HEAD) {
    throw new Error(`SOC wire too short: ${wire.length}`);
  }

  return wire.slice(HEAD);
}

async function fetchChunk(gateway: string, address: string): Promise<Uint8Array | null> {
  const url = `${gateway.replace(/\/$/, '')}/chunks/${address}`;

  try {
    const res = await fetch(url);

    if (res.status === 404) {
      return null;
    }

    if (!res.ok) {
      throw new Error(`HTTP ${res.status} for ${url}`);
    }

    return new Uint8Array(await res.arrayBuffer());
  } catch (err) {
    throw new Error(`fetch failed for ${url}: ${(err as Error).message}`);
  }
}

async function probe(
  gateway: string,
  topicBytes: Uint8Array,
  owner: Uint8Array,
  index: number,
): Promise<FeedPayload | null> {
  const id = idAt(topicBytes, index);
  const addr = socAddress(id, owner);
  const wire = await fetchChunk(gateway, hex(addr));

  if (!wire) {
    return null;
  }

  const payload = payloadFromWire(wire);
  const text = new TextDecoder().decode(payload);

  return JSON.parse(text) as FeedPayload;
}

async function findLatest(
  gateway: string,
  topicBytes: Uint8Array,
  owner: Uint8Array,
): Promise<number | null> {
  const zero = await probe(gateway, topicBytes, owner, 0);

  if (!zero) {
    return null;
  }

  let lastHit = 0;
  let step = 1;
  let firstMiss: number | null = null;

  for (let i = 0; i < MAX_PROBE_LOG2; i++) {
    const probeIdx = lastHit + step;
    const hit = await probe(gateway, topicBytes, owner, probeIdx);

    if (hit) {
      lastHit = probeIdx;
      step *= 2;
    } else {
      firstMiss = probeIdx;
      break;
    }
  }

  if (firstMiss === null) {
    return lastHit;
  }

  let lo = lastHit + 1;
  let hi = firstMiss;

  while (lo < hi) {
    const mid = lo + Math.floor((hi - lo) / 2);
    const hit = await probe(gateway, topicBytes, owner, mid);

    if (hit) {
      lastHit = mid;
      lo = mid + 1;
    } else {
      hi = mid;
    }
  }

  return lastHit;
}

async function collectFeed(
  gateway: string,
  topicBytes: Uint8Array,
  owner: Uint8Array,
  latest: number,
): Promise<FeedEntry[]> {
  const out: FeedEntry[] = [];

  for (let i = latest; i >= 0; i--) {
    const payload = await probe(gateway, topicBytes, owner, i);

    if (!payload) {
      continue;
    }

    out.push({
      index: i,
      version: payload.version,
      ref: payload.ref,
      publishedAt: payload.published_at,
    });
  }

  return out;
}

function writeMetadata(meta: {
  ref: string | null;
  index: number | null;
  owner: string | null;
  version: string | null;
  publishedAt: number | null;
  packageName: string;
  gateway: string;
}): void {
  const fields = [
    `  ref: ${meta.ref === null ? 'null' : JSON.stringify(meta.ref)}`,
    `  index: ${meta.index === null ? 'null' : meta.index}`,
    `  owner: ${meta.owner === null ? 'null' : JSON.stringify(meta.owner)}`,
    `  version: ${meta.version === null ? 'null' : JSON.stringify(meta.version)}`,
    `  publishedAt: ${meta.publishedAt === null ? 'null' : meta.publishedAt}`,
    `  packageName: ${JSON.stringify(meta.packageName)}`,
    `  gateway: ${JSON.stringify(meta.gateway)}`,
  ].join(',\n');

  const body = `// AUTO-GENERATED by scripts/refresh-metadata.ts. Edit the script, not this file.

export interface DemoMetadata {
  ref: string | null;
  index: number | null;
  owner: string | null;
  version: string | null;
  publishedAt: number | null;
  packageName: string;
  gateway: string;
}

export const metadata: DemoMetadata = {
${fields},
};
`;

  writeFileSync(resolve('src/lib/metadata.ts'), body);
}

function writeFeed(entries: FeedEntry[]): void {
  const lines = entries
    .map(
      (e) =>
        `  { index: ${e.index}, version: ${JSON.stringify(e.version)}, ref: ${JSON.stringify(e.ref)}, publishedAt: ${e.publishedAt} }`,
    )
    .join(',\n');

  const body = `// AUTO-GENERATED by scripts/refresh-metadata.ts. Edit the script, not this file.

export interface FeedEntry {
  index: number;
  version: string;
  ref: string;
  publishedAt: number;
}

export const feedEntries: FeedEntry[] = [
${lines}${entries.length ? ',' : ''}
];
`;

  writeFileSync(resolve('src/lib/feedData.ts'), body);
}

async function main(): Promise<void> {
  const gateway = process.env.DAPPSWARM_GATEWAY ?? 'http://127.0.0.1:1633';
  const name = process.env.DAPPSWARM_DEMO_NAME ?? 'dappswarm-demo';
  const ownerHex = process.env.DAPPSWARM_DEMO_OWNER;
  const publicGateway = process.env.DAPPSWARM_PUBLIC_GATEWAY ?? 'https://api.gateway.ethswarm.org';

  if (!ownerHex || !/^[0-9a-fA-F]{40}$/.test(ownerHex.replace(/^0x/, ''))) {
    console.warn('refresh-metadata: DAPPSWARM_DEMO_OWNER not set or invalid; writing placeholders.');
    writeMetadata({
      ref: null,
      index: null,
      owner: null,
      version: null,
      publishedAt: null,
      packageName: name,
      gateway: publicGateway,
    });
    writeFeed([]);

    return;
  }

  const owner = unhex(ownerHex);
  const topicBytes = topic(name);

  try {
    const latest = await findLatest(gateway, topicBytes, owner);

    if (latest === null) {
      console.warn(`refresh-metadata: no feed entries found for ${name}; writing placeholders.`);
      writeMetadata({
        ref: null,
        index: null,
        owner: '0x' + ownerHex.replace(/^0x/, '').toLowerCase(),
        version: null,
        publishedAt: null,
        packageName: name,
        gateway: publicGateway,
      });
      writeFeed([]);

      return;
    }

    const entries = await collectFeed(gateway, topicBytes, owner, latest);
    const head = entries[0];

    writeMetadata({
      ref: head.ref,
      index: head.index,
      owner: '0x' + ownerHex.replace(/^0x/, '').toLowerCase(),
      version: head.version,
      publishedAt: head.publishedAt,
      packageName: name,
      gateway: publicGateway,
    });
    writeFeed(entries);

    console.log(`refresh-metadata: wrote ${entries.length} feed entries; head v${head.version} @ ${head.index}.`);
  } catch (err) {
    console.warn(`refresh-metadata: ${(err as Error).message}; writing placeholders.`);
    writeMetadata({
      ref: null,
      index: null,
      owner: '0x' + ownerHex.replace(/^0x/, '').toLowerCase(),
      version: null,
      publishedAt: null,
      packageName: name,
      gateway: publicGateway,
    });
    writeFeed([]);
  }
}

main().catch((err) => {
  console.error('refresh-metadata fatal:', err);
  process.exit(1);
});
