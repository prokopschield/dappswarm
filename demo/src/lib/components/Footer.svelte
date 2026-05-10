<script lang="ts">
  import { metadata } from '$lib/metadata';
</script>

<footer class="border-t border-ink-border bg-ink-surface/40">
  <div class="mx-auto max-w-page px-6 py-12">
    <div class="flex flex-wrap items-end justify-between gap-8">
      <div class="flex flex-col gap-2">
        <p class="font-display text-2xl font-bold text-cream tracking-tightest">
          <span class="text-honey">dapp</span>swarm
        </p>
        <p class="max-w-md text-sm text-cream-mute">
          Built with dappswarm, and you're reading the proof.
          {#if metadata.ref}
            This page is served from
            <a class="hex-link hex-mono" href="{metadata.gateway}/bzz/{metadata.ref.replace(/^0x/, '')}/" target="_blank" rel="noreferrer">
              bzz:{metadata.ref.slice(0, 10)}…
            </a>
          {/if}
        </p>
      </div>

      <nav class="flex flex-wrap items-center gap-x-6 gap-y-2 text-sm">
        <a href="https://github.com/prokopschield/dappswarm" target="_blank" rel="noreferrer" class="hex-link">repo</a>
        <a href="https://github.com/prokopschield/ant" target="_blank" rel="noreferrer" class="hex-link">antd</a>
        <a href="https://www.ethswarm.org/" target="_blank" rel="noreferrer" class="hex-link">swarm</a>
        <a href="https://dappnode.com/" target="_blank" rel="noreferrer" class="hex-link">dappnode</a>
        <a href="https://ethprague.com/" target="_blank" rel="noreferrer" class="hex-link">ethprague</a>
      </nav>
    </div>

    <div class="mt-10 flex flex-wrap items-center justify-between gap-4 border-t border-ink-border pt-6 text-xs text-mute">
      <p>GPL-3.0-or-later. Hackathon entry; not audited.</p>
      <p class="hex-mono">⬡ {metadata.packageName}</p>
    </div>
  </div>
</footer>
