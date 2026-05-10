// Reactive feed state shared across components. Populated once on
// mount; the page re-renders as soon as the gateway responds.

import { loadFeed, type FeedEntry } from './feedClient';
import { metadata } from './metadata';

type Status = 'idle' | 'loading' | 'ready' | 'empty' | 'error';

export const feedStore = $state<{
  status: Status;
  entries: FeedEntry[];
  latest: FeedEntry | null;
  error: string | null;
}>({
  status: 'idle',
  entries: [],
  latest: null,
  error: null,
});

let started = false;

export function startFeedLoad(): void {
  if (started || typeof window === 'undefined') {
    return;
  }

  started = true;
  feedStore.status = 'loading';

  loadFeed(metadata.gateway, metadata.owner, metadata.packageName)
    .then(({ entries, latest }) => {
      feedStore.entries = entries;
      feedStore.latest = latest;
      feedStore.status = entries.length === 0 ? 'empty' : 'ready';
    })
    .catch((err: unknown) => {
      feedStore.error = err instanceof Error ? err.message : String(err);
      feedStore.status = 'error';
    });
}
