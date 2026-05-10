<script lang="ts">
  import HexGrid from '$lib/viz/HexGrid.svelte';
  import SelfCertBadge from './SelfCertBadge.svelte';
</script>

<section class="relative isolate overflow-hidden hex-bg">
  <HexGrid density={32} opacity={0.25} />
  <div class="hex-grain absolute inset-0"></div>

  <div class="relative mx-auto flex min-h-[88vh] max-w-page flex-col justify-center px-6 py-24 sm:py-32">
    <div class="flex flex-col items-start gap-8">
      <div class="hex-pill animate-rise">
        <span class="hex-mono text-honey">⬡</span>
        <span class="text-cream-mute">ETHPrague Swarm Hackathon</span>
        <span class="text-mute">·</span>
        <span>Dappnode Packages on Swarm</span>
      </div>

      <h1
        class="font-display text-[clamp(3rem,11vw,9rem)] font-black leading-[0.9] tracking-tightest text-cream"
        style="animation: rise 800ms cubic-bezier(0.2, 0.8, 0.2, 1) 80ms both;"
      >
        <span class="text-honey">dapp</span><span class="text-cream">swarm</span>
      </h1>

      <p
        class="hex-lede max-w-2xl text-xl sm:text-2xl"
        style="animation: rise 800ms cubic-bezier(0.2, 0.8, 0.2, 1) 200ms both;"
      >
        Dappnode packages on
        <a href="https://www.ethswarm.org/" target="_blank" rel="noreferrer" class="hex-link">Swarm</a>,
        published as Mantaray bundles, versioned through sequence-indexed feeds, addressed
        deterministically from <span class="hex-mono text-honey">(owner, name)</span>. No registry. No IPFS hop. One
        command to install.
      </p>

      <div
        class="flex flex-wrap items-center gap-3"
        style="animation: rise 800ms cubic-bezier(0.2, 0.8, 0.2, 1) 320ms both;"
      >
        <a href="#how-it-works" class="hex-button-primary">
          How it works
          <span aria-hidden="true">↓</span>
        </a>
        <a
          href="https://github.com/prokopschield/dappswarm"
          target="_blank"
          rel="noreferrer"
          class="hex-button-ghost"
        >
          View source
          <span aria-hidden="true">↗</span>
        </a>
      </div>

      <div style="animation: rise 800ms cubic-bezier(0.2, 0.8, 0.2, 1) 460ms both;">
        <SelfCertBadge />
      </div>
    </div>
  </div>

  <!-- bottom fade -->
  <div
    class="pointer-events-none absolute inset-x-0 bottom-0 h-32 bg-gradient-to-b from-transparent to-ink"
    aria-hidden="true"
  ></div>
</section>
