<script lang="ts">
  const cards = [
    {
      eyebrow: 'Dappnodes',
      title: 'Sovereign infrastructure',
      body: 'A Dappnode is a turnkey home server for Ethereum, IPFS, and a hundred other services. Anyone can run one. Packages (manifests plus compose stacks) are how new services land on it.',
      diagram: 'home',
    },
    {
      eyebrow: 'Today',
      title: 'IPFS + ENS',
      body: 'Existing distribution leans on IPFS pinning and ENS lookups. Pinning is best-effort, ENS resolution adds a hop, and updates are a manual choreography of records and CIDs.',
      diagram: 'fragmented',
    },
    {
      eyebrow: 'dappswarm',
      title: 'Swarm + feeds',
      body: 'Bundles live on Swarm as Mantaray manifests. Versions live in single-owner-chunk feeds. The feed address is deterministic from the publisher and package name; no resolution layer.',
      diagram: 'unified',
    },
  ];
</script>

<section class="hex-section" id="why">
  <div class="mx-auto max-w-page">
    <div class="reveal mb-12 max-w-prose">
      <p class="hex-eyebrow mb-4">why this exists</p>
      <h2 class="hex-title">Distribution should be addressable.</h2>
    </div>

    <div class="grid gap-6 md:grid-cols-3">
      {#each cards as card, i (card.eyebrow)}
        <article
          class="hex-card-glow reveal flex flex-col gap-4"
          style="transition-delay: {i * 100}ms;"
        >
          <div class="flex items-center justify-between">
            <p class="hex-eyebrow">{card.eyebrow}</p>
            <div class="diagram diagram-{card.diagram}">
              {#if card.diagram === 'home'}
                <svg viewBox="0 0 32 32" class="h-8 w-8">
                  <path d="M16 4L4 14v14h8v-9h8v9h8V14z" fill="none" stroke="rgba(251,191,36,0.85)" stroke-width="1.4" stroke-linejoin="round"/>
                </svg>
              {:else if card.diagram === 'fragmented'}
                <svg viewBox="0 0 32 32" class="h-8 w-8">
                  <circle cx="9" cy="10" r="3" fill="none" stroke="rgba(251,191,36,0.6)" stroke-width="1.4"/>
                  <circle cx="23" cy="10" r="3" fill="none" stroke="rgba(251,191,36,0.6)" stroke-width="1.4"/>
                  <circle cx="16" cy="22" r="3" fill="none" stroke="rgba(251,191,36,0.6)" stroke-width="1.4"/>
                  <path d="M11 12 L14 20 M21 12 L18 20" stroke="rgba(251,191,36,0.4)" stroke-width="1.2" stroke-dasharray="2 2" fill="none"/>
                </svg>
              {:else}
                <svg viewBox="0 0 32 32" class="h-8 w-8">
                  <polygon points="16,3 27,9.5 27,22.5 16,29 5,22.5 5,9.5" fill="rgba(251,191,36,0.18)" stroke="rgba(251,191,36,0.85)" stroke-width="1.4"/>
                </svg>
              {/if}
            </div>
          </div>
          <h3 class="font-display text-2xl font-bold text-cream">{card.title}</h3>
          <p class="text-cream-mute">{card.body}</p>
        </article>
      {/each}
    </div>
  </div>
</section>
