<script lang="ts">
  import { metadata } from '$lib/metadata';
  import { shortHex } from '$lib/keccak';

  const verified = !!metadata.ref;
  const refLink = verified ? `${metadata.gateway}/bzz/${metadata.ref?.replace(/^0x/, '')}/` : null;
</script>

<div class="hex-pill flex-wrap gap-3 text-xs sm:text-sm">
  {#if verified}
    <span class="flex items-center gap-1.5">
      <span class="relative inline-flex h-2 w-2">
        <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-honey opacity-50"></span>
        <span class="relative inline-flex h-2 w-2 rounded-full bg-honey"></span>
      </span>
      <span>served from</span>
    </span>
    <a href={refLink} target="_blank" rel="noreferrer" class="hex-hash">
      bzz:{shortHex(metadata.ref ?? '', 8, 6)}
    </a>
    {#if metadata.version}
      <span class="text-mute">·</span>
      <span class="hex-mono text-cream">v{metadata.version}</span>
    {/if}
    {#if metadata.index !== null}
      <span class="text-mute">·</span>
      <span class="hex-mono text-cream">feed @ {metadata.index}</span>
    {/if}
    <span class="text-mute">·</span>
    <span class="text-honey">✓ verified</span>
  {:else}
    <span class="flex items-center gap-1.5">
      <span class="inline-flex h-2 w-2 rounded-full bg-mute"></span>
      <span class="text-mute">awaiting first publish</span>
    </span>
    <span class="text-mute">·</span>
    <span class="hex-mono text-cream-mute">{metadata.packageName}</span>
  {/if}
</div>
