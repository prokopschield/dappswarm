<script lang="ts">
  let { active = false, files = ['dappnode_package.json', 'docker-compose.yml', 'static/index.html', 'static/_app/...'] }: { active?: boolean; files?: string[] } = $props();

  const leaves = $derived(
    files.map((f, i) => ({
      name: f,
      x: 14 + (i % 4) * 22,
      y: 70 + Math.floor(i / 4) * 18,
      delay: i * 80,
    })),
  );

  const branches = $derived(
    leaves.map((leaf, i) => ({
      x1: 50,
      y1: 30,
      x2: leaf.x,
      y2: leaf.y - 6,
      delay: 200 + i * 60,
    })),
  );
</script>

<svg viewBox="0 0 100 100" class="h-full w-full" aria-hidden="true">
  <defs>
    <radialGradient id="root-glow" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="rgba(251, 191, 36, 0.7)"/>
      <stop offset="100%" stop-color="rgba(251, 191, 36, 0)"/>
    </radialGradient>
  </defs>

  <!-- subtle radial root -->
  <circle cx="50" cy="22" r="14" fill="url(#root-glow)" class:tree-active={active}/>

  <!-- branches -->
  {#each branches as b, i}
    <line
      x1={b.x1}
      y1={b.y1}
      x2={b.x2}
      y2={b.y2}
      stroke="rgba(251, 191, 36, 0.55)"
      stroke-width="0.4"
      stroke-linecap="round"
      class:branch-on={active}
      style="--delay: {b.delay}ms"
    />
  {/each}

  <!-- root chunk -->
  <g transform="translate(50 22)">
    <polygon
      points="0,-7 6,-3.5 6,3.5 0,7 -6,3.5 -6,-3.5"
      fill="rgba(251, 191, 36, 0.16)"
      stroke="rgba(251, 191, 36, 0.9)"
      stroke-width="0.4"
      class:root-on={active}
    />
    <text x="0" y="1" text-anchor="middle" fill="#fef3c7" font-family="JetBrains Mono Variable, monospace" font-size="2.4">manifest</text>
  </g>

  <!-- leaves -->
  {#each leaves as leaf, i}
    <g transform="translate({leaf.x} {leaf.y})" class:leaf-on={active} style="--delay: {leaf.delay}ms">
      <polygon
        points="0,-5 4.3,-2.5 4.3,2.5 0,5 -4.3,2.5 -4.3,-2.5"
        fill="rgba(23, 17, 9, 0.9)"
        stroke="rgba(251, 191, 36, 0.7)"
        stroke-width="0.3"
      />
      <text x="0" y="9" text-anchor="middle" fill="#d6d3cb" font-family="JetBrains Mono Variable, monospace" font-size="1.7">
        {leaf.name.length > 14 ? leaf.name.slice(0, 12) + '…' : leaf.name}
      </text>
    </g>
  {/each}
</svg>

<style>
  .tree-active {
    animation: pulse-glow 3s ease-in-out infinite;
  }
  .root-on {
    animation: root-pop 600ms cubic-bezier(0.2, 0.8, 0.2, 1) backwards;
  }
  .branch-on {
    stroke-dasharray: 90;
    stroke-dashoffset: 90;
    animation: dash-in 700ms cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
    animation-delay: var(--delay);
  }
  .leaf-on {
    opacity: 0;
    animation: leaf-pop 500ms cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
    animation-delay: var(--delay);
  }

  @keyframes pulse-glow {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 1; }
  }
  @keyframes root-pop {
    from { transform: scale(0.4); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
  }
  @keyframes dash-in {
    to { stroke-dashoffset: 0; }
  }
  @keyframes leaf-pop {
    from { opacity: 0; transform: translateY(8px) scale(0.5); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  @media (prefers-reduced-motion: reduce) {
    .branch-on, .leaf-on, .root-on, .tree-active {
      animation: none !important;
      opacity: 1 !important;
      stroke-dashoffset: 0 !important;
    }
  }
</style>
