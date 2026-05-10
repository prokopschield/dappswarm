<script lang="ts">
  import { onMount } from 'svelte';
  import MantarayTree from '$lib/viz/MantarayTree.svelte';

  let stage = $state(-1);
  let started = $state(false);
  let container: HTMLElement;

  onMount(() => {
    const obs = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting && !started) {
            started = true;
            const interval = window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 80 : 1000;

            for (let i = 0; i < 4; i++) {
              setTimeout(() => (stage = i), i * interval);
            }
          }
        }
      },
      { threshold: 0.4 },
    );
    if (container) obs.observe(container);

    return () => obs.disconnect();
  });

  // probe sequence visualisation
  const probeSteps = [
    { idx: 1, hit: true },
    { idx: 2, hit: true },
    { idx: 4, hit: true },
    { idx: 8, hit: false },
    { idx: 6, hit: true },
    { idx: 7, hit: false },
  ];
  const targetIndex = 6;
</script>

<section class="hex-section bg-ink-surface/40" bind:this={container}>
  <div class="mx-auto max-w-page">
    <div class="reveal mb-12 max-w-prose">
      <p class="hex-eyebrow mb-4">flow #2, resolve</p>
      <h2 class="hex-title">Probe. Decode. Walk. Run.</h2>
      <p class="hex-lede mt-6">
        Resolution is symmetric: one feed lookup gives you the latest version, one /bzz walk gives you the bytes,
        one <span class="hex-mono text-honey">docker compose up -d</span> gives you the running service.
      </p>
    </div>

    <div class="grid gap-6 lg:grid-cols-4">
      <!-- 1. Probe -->
      <article class="hex-card-glow flow-step" class:flow-step-active={stage >= 0}>
        <div class="flow-head">
          <span class="flow-num">1</span>
          <h3 class="flow-title">Probe</h3>
        </div>
        <p class="flow-body mb-4">
          Exponential probe finds the upper bound; binary search drills to latest.
        </p>
        <div class="flex flex-wrap gap-1.5">
          {#each probeSteps as p, i (i)}
            <span
              class="probe-cell"
              class:probe-on={stage >= 0}
              class:probe-hit={p.hit}
              style="--delay: {i * 90}ms"
            >
              {p.idx}{p.hit ? '✓' : '✗'}
            </span>
          {/each}
        </div>
        <p class="mt-3 hex-mono text-xs text-honey">→ latest = {targetIndex}</p>
      </article>

      <!-- 2. Decode -->
      <article class="hex-card-glow flow-step" class:flow-step-active={stage >= 1}>
        <div class="flow-head">
          <span class="flow-num">2</span>
          <h3 class="flow-title">Decode</h3>
        </div>
        <p class="flow-body mb-4">
          The SOC payload is a tiny JSON envelope. ≤ 4 KiB by construction.
        </p>
        <pre class="rounded-lg bg-ink-raised/80 p-3 text-[11px] leading-relaxed"><code>{`{
  "version": "0.1.0",
  "ref": "0x7f…",
  "published_at": …,
  "files": [
    "dappnode_package.json",
    "docker-compose.yml",
    "static/index.html",
    …
  ]
}`}</code></pre>
      </article>

      <!-- 3. Walk -->
      <article class="hex-card-glow flow-step" class:flow-step-active={stage >= 2}>
        <div class="flow-head">
          <span class="flow-num">3</span>
          <h3 class="flow-title">Walk</h3>
        </div>
        <p class="flow-body mb-4">
          For each file in <span class="hex-mono text-honey">payload.files</span>:
          <span class="hex-mono text-honey">GET /bzz/{`<ref>`}/{`<path>`}</span>.
        </p>
        <div class="grid h-32 place-items-center">
          <MantarayTree active={stage >= 2} />
        </div>
      </article>

      <!-- 4. Run -->
      <article class="hex-card-glow flow-step" class:flow-step-active={stage >= 3}>
        <div class="flow-head">
          <span class="flow-num">4</span>
          <h3 class="flow-title">Run</h3>
        </div>
        <p class="flow-body mb-4">
          <span class="hex-mono text-honey">docker load</span> on every <span class="hex-mono text-honey">*.tar.xz</span>;
          then <span class="hex-mono text-honey">docker compose up -d</span>.
        </p>
        <div class="hex-card !bg-black/40 !p-3">
          <p class="hex-mono text-[11px] text-honey">▲ dappswarm-demo · running</p>
          <p class="hex-mono mt-1 text-[11px] text-mute">localhost:8081 · 200 OK</p>
        </div>
      </article>
    </div>
  </div>
</section>

<style>
  .flow-step {
    transition: transform 600ms cubic-bezier(0.2, 0.8, 0.2, 1), opacity 600ms cubic-bezier(0.2, 0.8, 0.2, 1), border-color 600ms ease;
    opacity: 0.5;
    transform: translateY(8px);
  }
  .flow-step-active {
    opacity: 1;
    transform: translateY(0);
    border-color: rgba(251, 191, 36, 0.35);
  }
  .flow-head { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; }
  .flow-num {
    display: inline-grid; place-items: center;
    width: 1.75rem; height: 1.75rem;
    border-radius: 0.5rem;
    border: 1px solid rgba(251, 191, 36, 0.3);
    color: #fbbf24;
    font-family: 'JetBrains Mono Variable', monospace;
    font-size: 0.875rem;
  }
  .flow-title {
    font-family: 'Inter Variable', sans-serif;
    font-weight: 700;
    font-size: 1.25rem;
    color: #fef3c7;
  }
  .flow-body { color: #d6d3cb; font-size: 0.95rem; }

  .probe-cell {
    font-family: 'JetBrains Mono Variable', monospace;
    font-size: 0.7rem;
    padding: 0.2rem 0.5rem;
    border-radius: 999px;
    border: 1px solid rgba(168, 162, 158, 0.3);
    color: #a8a29e;
    opacity: 0;
    transform: translateY(4px);
    transition: all 400ms cubic-bezier(0.2, 0.8, 0.2, 1);
    transition-delay: var(--delay);
  }
  .probe-on {
    opacity: 1;
    transform: translateY(0);
  }
  .probe-on.probe-hit {
    border-color: rgba(251, 191, 36, 0.5);
    color: #fbbf24;
  }
</style>
