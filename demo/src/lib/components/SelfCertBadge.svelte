<script lang="ts">
  import { metadata } from '$lib/metadata';
  import { feedStore } from '$lib/feedStore.svelte';
  import { shortHex } from '$lib/keccak';
</script>

<div class="hex-pill flex-wrap gap-3 text-xs sm:text-sm">
  {#if feedStore.status === 'ready' && feedStore.latest}
    <span class="flex items-center gap-1.5">
      <span class="relative inline-flex h-2 w-2">
        <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-honey opacity-50"></span>
        <span class="relative inline-flex h-2 w-2 rounded-full bg-honey"></span>
      </span>
      <span>latest</span>
    </span>
    <a
      href="{metadata.gateway}/bzz/{feedStore.latest.ref.replace(/^0x/, '')}/"
      target="_blank"
      rel="noreferrer"
      class="hex-hash"
    >
      bzz:{shortHex(feedStore.latest.ref, 8, 6)}
    </a>
    <span class="text-mute">·</span>
    <span class="hex-mono text-cream">v{feedStore.latest.version}</span>
    <span class="text-mute">·</span>
    <span class="hex-mono text-cream">feed @ {feedStore.latest.index}</span>
    <span class="text-mute">·</span>
    <span class="text-honey">✓ live</span>
  {:else if feedStore.status === 'loading'}
    <span class="flex items-center gap-1.5">
      <span class="inline-flex h-2 w-2 animate-pulse rounded-full bg-honey/60"></span>
      <span class="text-mute">resolving feed</span>
    </span>
    <span class="text-mute">·</span>
    <span class="hex-mono text-cream-mute">{metadata.packageName}</span>
  {:else if feedStore.status === 'error'}
    <span class="flex items-center gap-1.5">
      <span class="inline-flex h-2 w-2 rounded-full bg-red-400/80"></span>
      <span class="text-mute">gateway unreachable</span>
    </span>
    <span class="text-mute">·</span>
    <span class="hex-mono text-cream-mute">{metadata.packageName}</span>
  {:else}
    <span class="flex items-center gap-1.5">
      <span class="inline-flex h-2 w-2 rounded-full bg-mute"></span>
      <span class="text-mute">awaiting first publish</span>
    </span>
    <span class="text-mute">·</span>
    <span class="hex-mono text-cream-mute">{metadata.packageName}</span>
  {/if}
</div>
