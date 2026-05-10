<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  interface Line {
    kind: 'cmd' | 'out' | 'note';
    text: string;
    delay?: number;
  }

  const script: Line[] = [
    { kind: 'cmd', text: 'dappswarm doctor' },
    { kind: 'out', text: '✓ antd 0.4.2 · API v6.0 · batch 0xabcd…ef01' },
    { kind: 'cmd', text: 'dappswarm publish demo/dist/' },
    { kind: 'out', text: 'tar: 312 files · 487 KiB' },
    { kind: 'out', text: 'POST /bzz · 201 → 0x7f…' },
    { kind: 'out', text: 'feed dappswarm-demo @ index 3 · owner 0xa07c…b8c9' },
    { kind: 'out', text: '✓ published in 814 ms' },
    { kind: 'cmd', text: 'dappswarm install dappswarm-demo --owner 0xa07c…b8c9' },
    { kind: 'out', text: 'resolve · feed @ 3 · ref 0x7f…' },
    { kind: 'out', text: 'docker compose up -d · ✓ dappswarm-demo' },
    { kind: 'cmd', text: 'curl -fsS localhost:8081 | head -1' },
    { kind: 'out', text: '<!doctype html>' },
    { kind: 'note', text: 'you are here ↑' },
  ];

  let visible = $state(0);
  let cursor = $state(true);
  let container: HTMLElement;
  let timers: ReturnType<typeof setTimeout>[] = [];
  let cursorTimer: ReturnType<typeof setInterval> | null = null;

  function start() {
    if (visible > 0) return;

    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    if (reduce) {
      visible = script.length;
      return;
    }

    let acc = 0;

    for (let i = 1; i <= script.length; i++) {
      const line = script[i - 1];
      const dur = line.kind === 'cmd' ? 600 : 240;

      acc += dur;
      timers.push(setTimeout(() => (visible = i), acc));
    }
  }

  onMount(() => {
    cursorTimer = setInterval(() => (cursor = !cursor), 540);
    const obs = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) start();
        }
      },
      { threshold: 0.3 },
    );
    if (container) obs.observe(container);

    return () => obs.disconnect();
  });

  onDestroy(() => {
    timers.forEach(clearTimeout);
    if (cursorTimer) clearInterval(cursorTimer);
  });
</script>

<section class="hex-section" bind:this={container}>
  <div class="mx-auto max-w-page">
    <div class="reveal mb-10 max-w-prose">
      <p class="hex-eyebrow mb-4">in the terminal</p>
      <h2 class="hex-title">Three commands.</h2>
      <p class="hex-lede mt-6">
        That's the whole demo. Doctor, publish, install. The page you're reading is the output of the third one.
      </p>
    </div>

    <div class="reveal mx-auto max-w-3xl overflow-hidden rounded-2xl border border-ink-border bg-black/60 shadow-2xl">
      <div class="flex items-center gap-2 border-b border-ink-border bg-ink-surface/80 px-4 py-3">
        <span class="h-3 w-3 rounded-full bg-red-400/60"></span>
        <span class="h-3 w-3 rounded-full bg-yellow-400/60"></span>
        <span class="h-3 w-3 rounded-full bg-green-400/60"></span>
        <span class="ml-3 hex-mono text-xs text-mute">~ · dappswarm · 80×24</span>
      </div>
      <div class="hex-mono p-5 text-[13px] leading-[1.7] text-cream-mute min-h-[20rem]">
        {#each script as line, i (i)}
          {#if i < visible}
            {#if line.kind === 'cmd'}
              <div class="text-cream">
                <span class="text-honey">▶</span> {line.text}
              </div>
            {:else if line.kind === 'out'}
              <div class="text-cream-mute pl-4">{line.text}</div>
            {:else}
              <div class="pt-2 text-honey/70 italic pl-4">{line.text}</div>
            {/if}
          {/if}
        {/each}
        {#if visible < script.length}
          <div class="text-cream">
            <span class="text-honey">▶</span>
            <span class="inline-block w-2 h-4 align-middle"
              style="background: {cursor ? '#fbbf24' : 'transparent'};"
            ></span>
          </div>
        {/if}
      </div>
    </div>
  </div>
</section>
