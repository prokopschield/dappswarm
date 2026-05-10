<script lang="ts">
  let { active = false, count = 7 }: { active?: boolean; count?: number } = $props();

  const chunks = $derived(
    Array.from({ length: count }, (_, i) => ({
      id: i,
      delay: i * 110,
    })),
  );
</script>

<svg viewBox="0 0 200 60" class="h-full w-full" aria-hidden="true">
  <defs>
    <linearGradient id="stream-grad" x1="0" x2="1">
      <stop offset="0%" stop-color="rgba(251, 191, 36, 0)"/>
      <stop offset="50%" stop-color="rgba(251, 191, 36, 0.4)"/>
      <stop offset="100%" stop-color="rgba(251, 191, 36, 0)"/>
    </linearGradient>
  </defs>

  <!-- source: bundle dir -->
  <g transform="translate(8 30)">
    <rect x="-5" y="-8" width="10" height="16" rx="1.4" fill="rgba(23, 17, 9, 0.9)" stroke="rgba(251, 191, 36, 0.7)" stroke-width="0.4"/>
    <text x="0" y="-12" text-anchor="middle" fill="#d6d3cb" font-family="JetBrains Mono Variable, monospace" font-size="3">dir</text>
  </g>

  <!-- stream lane -->
  <line x1="20" y1="30" x2="180" y2="30" stroke="url(#stream-grad)" stroke-width="0.6"/>

  <!-- chunks flying through -->
  {#each chunks as c}
    <g style="--delay: {c.delay}ms" class:chunk-flow={active}>
      <polygon
        points="0,-2.6 2.2,-1.3 2.2,1.3 0,2.6 -2.2,1.3 -2.2,-1.3"
        fill="rgba(251, 191, 36, 0.18)"
        stroke="rgba(251, 191, 36, 0.9)"
        stroke-width="0.3"
        transform="translate(20 30)"
      />
    </g>
  {/each}

  <!-- target: antd -->
  <g transform="translate(192 30)">
    <circle r="9" fill="rgba(23, 17, 9, 0.9)" stroke="rgba(251, 191, 36, 0.8)" stroke-width="0.5"/>
    <text x="0" y="1" text-anchor="middle" fill="#fbbf24" font-family="JetBrains Mono Variable, monospace" font-size="3">antd</text>
    <text x="0" y="-12" text-anchor="middle" fill="#d6d3cb" font-family="JetBrains Mono Variable, monospace" font-size="3">/bzz</text>
  </g>
</svg>

<style>
  .chunk-flow {
    animation: chunk-fly 2.4s linear infinite;
    animation-delay: var(--delay);
  }
  @keyframes chunk-fly {
    0%   { transform: translate(0, 0); opacity: 0; }
    10%  { opacity: 1; }
    90%  { opacity: 1; }
    100% { transform: translate(170px, 0); opacity: 0; }
  }
  @media (prefers-reduced-motion: reduce) {
    .chunk-flow { animation: none; }
  }
</style>
