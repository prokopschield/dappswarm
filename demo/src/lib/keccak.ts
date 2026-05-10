import sha3 from 'js-sha3';

const { keccak256 } = sha3;

export const FEED_TOPIC_PREFIX = 'dappswarm:';

const enc = new TextEncoder();

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

function hex(b: Uint8Array): string {
  return Array.from(b, (x) => x.toString(16).padStart(2, '0')).join('');
}

function unhex(s: string): Uint8Array {
  const v = s.toLowerCase().replace(/^0x/, '');
  const out = new Uint8Array(v.length / 2);

  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(v.slice(i * 2, i * 2 + 2), 16);
  }

  return out;
}

function k(input: Uint8Array): Uint8Array {
  return new Uint8Array(keccak256.arrayBuffer(input));
}

export function topic(name: string): Uint8Array {
  return k(enc.encode(FEED_TOPIC_PREFIX + name));
}

export function idAt(topicBytes: Uint8Array, index: number): Uint8Array {
  return k(concat(topicBytes, u64BE(index)));
}

export function socAddress(idBytes: Uint8Array, ownerEoa: Uint8Array): Uint8Array {
  return k(concat(idBytes, ownerEoa));
}

export function deriveFeed(
  name: string,
  index: number,
  ownerHex: string,
): {
  topic: string;
  id: string;
  address: string;
} {
  const t = topic(name);
  const i = idAt(t, index);
  const owner = unhex(ownerHex);
  const a = socAddress(i, owner);

  return {
    topic: '0x' + hex(t),
    id: '0x' + hex(i),
    address: '0x' + hex(a),
  };
}

export function shortHex(s: string, head = 6, tail = 4): string {
  if (!s) {
    return '';
  }

  const stripped = s.startsWith('0x') ? s.slice(2) : s;

  if (stripped.length <= head + tail + 2) {
    return s.startsWith('0x') ? s : '0x' + s;
  }

  return `0x${stripped.slice(0, head)}…${stripped.slice(-tail)}`;
}

export { hex, unhex };
