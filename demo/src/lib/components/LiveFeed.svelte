<script lang="ts">
  import { feedStore } from '$lib/feedStore.svelte';
  import { metadata } from '$lib/metadata';
  import { shortHex } from '$lib/keccak';

  function fmtTime(unix: number): string {
    if (!unix) return '?';
    const d = new Date(unix * 1000);

    return d.toISOString().replace('T', ' ').slice(0, 19) + 'Z';
  }
</script>

<section class="hex-section bg-ink-surface/40">
  <div class="mx-auto max-w-page">
    <div class="reveal mb-10 max-w-prose">
      <p class="hex-eyebrow mb-4">this page's feed</p>
      <h2 class="hex-title">You're reading the proof.</h2>
      <p class="hex-lede mt-6">
        Every row below is a real feed entry, fetched live from the gateway when this page loaded. The
        currently-served version is highlighted.
      </p>
    </div>

    <div class="reveal hex-card-glow overflow-hidden p-0">
      <div class="grid grid-cols-[auto_1fr_auto] gap-x-6 gap-y-0 px-6 py-3 text-xs uppercase tracking-[0.2em] text-mute border-b border-ink-border">
        <span>idx</span>
        <span>version · ref</span>
        <span class="text-right">published</span>
      </div>

      {#if feedStore.status === 'loading' || feedStore.status === 'idle'}
        <div class="px-6 py-10 text-center text-mute">
          <p class="hex-mono mb-2">// resolving feed from gateway</p>
        </div>
      {:else if feedStore.status === 'error'}
        <div class="px-6 py-10 text-center text-mute">
          <p class="hex-mono mb-2 text-red-400/80">// gateway unreachable</p>
          <p class="text-sm">{feedStore.error}</p>
        </div>
      {:else if feedStore.entries.length === 0}
        <div class="px-6 py-10 text-center text-mute">
          <p class="hex-mono mb-2">// awaiting first publish</p>
          <p class="text-sm">
            Run <span class="hex-mono text-honey">make demo-publish</span> to populate this page's feed.
          </p>
        </div>
      {:else}
        {#each feedStore.entries as entry, i (entry.index)}
          <a
            href="{metadata.gateway}/bzz/{entry.ref.replace(/^0x/, '')}/"
            target="_blank"
            rel="noreferrer"
            class="feed-row grid grid-cols-[auto_1fr_auto] items-center gap-x-6 px-6 py-4 transition-colors"
            class:current={i === 0}
          >
            <span class="hex-mono text-sm text-honey">@{entry.index}</span>
            <span class="flex flex-col gap-0.5">
              <span class="flex items-baseline gap-3">
                <span class="font-display font-bold text-cream">v{entry.version}</span>
                {#if i === 0}
                  <span class="hex-mono text-[10px] uppercase tracking-[0.2em] text-honey">currently served</span>
                {/if}
              </span>
              <span class="hex-hash text-xs">{shortHex(entry.ref, 12, 8)}</span>
            </span>
            <span class="hex-mono text-xs text-mute text-right">{fmtTime(entry.publishedAt)}</span>
          </a>
        {/each}
      {/if}
    </div>

    {#if metadata.owner}
      <p class="mt-6 text-xs text-mute">
        owner <span class="hex-hash">{shortHex(metadata.owner, 8, 6)}</span> ·
        package <span class="hex-mono text-cream-mute">{metadata.packageName}</span>
      </p>
    {/if}
  </div>
</section>

<style>
  .feed-row {
    border-bottom: 1px solid rgba(42, 32, 20, 0.6);
  }
  .feed-row:last-child { border-bottom: 0; }
  .feed-row:hover {
    background: rgba(251, 191, 36, 0.04);
  }
  .feed-row.current {
    background: rgba(251, 191, 36, 0.06);
    box-shadow: inset 3px 0 0 #fbbf24;
  }
</style>
