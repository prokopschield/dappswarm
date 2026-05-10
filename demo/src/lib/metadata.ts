// Static configuration for this demo's feed. The dynamic state (current
// bzz ref, latest index, version, history) is fetched at runtime from
// the gateway by feedClient.ts, so the page reflects whatever is live
// rather than a snapshot baked at build time.
//
// `owner` is the EOA the demo's DAPPSWARM_KEY corresponds to. Update
// it once before the first publish.

export interface DemoConfig {
  owner: string;
  packageName: string;
  gateway: string;
}

export const metadata: DemoConfig = {
  owner: '0xfe86589d685ab62d4f2d65955423fd2e2ac20488',
  packageName: 'dappswarm-demo',
  gateway: 'https://api.gateway.ethswarm.org',
};
