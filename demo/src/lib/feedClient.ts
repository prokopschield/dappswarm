// Runtime feed client: probes the demo's own SOC feed via a Swarm
// gateway, decodes payloads, and returns the version history.
// Mirrors src/feed.rs::find_latest in the rust CLI: probe 0, then
// exponential probe (1, 2, 4, 8, ...) until a miss, then binary
// search across the gap.

import { topic, idAt, socAddress, hex, unhex } from './keccak';

export interface FeedPayload {
  version: string;
  ref: string;
  published_at: number;
  files?: string[];
}

export interface FeedEntry {
  index: number;
  version: string;
  ref: string;
  publishedAt: number;
}

const SOC_HEAD_LEN = 32 + 65 + 8;
const MAX_PROBE_LOG2 = 32;

async function fetchPayload(
  gateway: string,
  ownerBytes: Uint8Array,
  topicBytes: Uint8Array,
  index: number,
  signal?: AbortSignal,
): Promise<FeedPayload | null> {
  const id = idAt(topicBytes, index);
  const addr = socAddress(id, ownerBytes);
  const url = `${gateway.replace(/\/$/, '')}/chunks/${hex(addr)}`;
  const res = await fetch(url, { signal, cache: 'no-store' });

  if (res.status === 404) {
    return null;
  }

  if (!res.ok) {
    throw new Error(`gateway returned ${res.status} for index ${index}`);
  }

  const wire = new Uint8Array(await res.arrayBuffer());

  if (wire.length < SOC_HEAD_LEN) {
    throw new Error(`SOC at index ${index} is too short: ${wire.length} bytes`);
  }

  const payload = wire.slice(SOC_HEAD_LEN);
  const text = new TextDecoder().decode(payload);

  return JSON.parse(text) as FeedPayload;
}

async function findLatest(
  gateway: string,
  ownerBytes: Uint8Array,
  topicBytes: Uint8Array,
  signal?: AbortSignal,
): Promise<number | null> {
  const zero = await fetchPayload(gateway, ownerBytes, topicBytes, 0, signal);

  if (!zero) {
    return null;
  }

  let lastHit = 0;
  let step = 1;
  let firstMiss: number | null = null;

  for (let i = 0; i < MAX_PROBE_LOG2; i++) {
    const probeIdx = lastHit + step;
    const hit = await fetchPayload(gateway, ownerBytes, topicBytes, probeIdx, signal);

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
    const hit = await fetchPayload(gateway, ownerBytes, topicBytes, mid, signal);

    if (hit) {
      lastHit = mid;
      lo = mid + 1;
    } else {
      hi = mid;
    }
  }

  return lastHit;
}

export async function loadFeed(
  gateway: string,
  ownerHex: string,
  packageName: string,
  signal?: AbortSignal,
): Promise<{ entries: FeedEntry[]; latest: FeedEntry | null }> {
  const stripped = ownerHex.replace(/^0x/i, '').toLowerCase();

  if (!/^[0-9a-f]{40}$/.test(stripped) || /^0+$/.test(stripped)) {
    return { entries: [], latest: null };
  }

  const ownerBytes = unhex(stripped);
  const topicBytes = topic(packageName);
  const latestIndex = await findLatest(gateway, ownerBytes, topicBytes, signal);

  if (latestIndex === null) {
    return { entries: [], latest: null };
  }

  const indices = Array.from({ length: latestIndex + 1 }, (_, i) => latestIndex - i);
  const payloads = await Promise.all(
    indices.map((i) => fetchPayload(gateway, ownerBytes, topicBytes, i, signal).then((p) => ({ index: i, payload: p }))),
  );

  const entries: FeedEntry[] = [];

  for (const r of payloads) {
    if (!r.payload) {
      continue;
    }

    entries.push({
      index: r.index,
      version: r.payload.version,
      ref: r.payload.ref,
      publishedAt: r.payload.published_at,
    });
  }

  return { entries, latest: entries[0] ?? null };
}
