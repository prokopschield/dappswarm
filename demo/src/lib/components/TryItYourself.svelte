<script lang="ts">
  let copied = $state<number | null>(null);

  async function copy(text: string, idx: number) {
    try {
      await navigator.clipboard.writeText(text);
      copied = idx;
      setTimeout(() => (copied = null), 1400);
    } catch {
      // best-effort; skip
    }
  }

  const steps = [
    {
      title: 'Run antd locally',
      body: 'Clone the patched ant-gateway and run it against a Gnosis RPC and a funded postage batch.',
      code: `git clone https://github.com/prokopschield/ant
cd ant && cargo run -p antd`,
    },
    {
      title: 'Set the env',
      body: 'A throwaway secp256k1 key for feed signing. Do not reuse the postage signer.',
      code: `export DAPPSWARM_GATEWAY=http://127.0.0.1:1633
export STAMP_BATCH_ID=0x…
export DAPPSWARM_KEY=0x…`,
    },
    {
      title: 'Publish + install the demo',
      body: 'The page you are reading is the build output of these two commands.',
      code: `git clone https://github.com/prokopschield/dappswarm
cd dappswarm/demo && npm install && ./scripts/build-bundle.sh
cd .. && cargo run -- publish demo/dist
cargo run -- install dappswarm-demo --owner 0x…`,
    },
  ];
</script>

<section class="hex-section" id="try">
  <div class="mx-auto max-w-page">
    <div class="reveal mb-12 max-w-prose">
      <p class="hex-eyebrow mb-4">try it yourself</p>
      <h2 class="hex-title">Three steps. Same demo.</h2>
    </div>

    <div class="grid gap-6 md:grid-cols-3">
      {#each steps as step, i (step.title)}
        <article class="hex-card-glow reveal flex flex-col gap-4" style="transition-delay: {i * 100}ms;">
          <div class="flex items-baseline gap-3">
            <span class="hex-mono inline-grid h-7 w-7 place-items-center rounded-md border border-honey/30 text-sm text-honey">
              {i + 1}
            </span>
            <h3 class="font-display text-xl font-bold text-cream">{step.title}</h3>
          </div>
          <p class="text-sm text-cream-mute">{step.body}</p>
          <div class="relative">
            <pre class="overflow-x-auto rounded-lg bg-black/60 p-3 text-[11.5px] leading-relaxed text-cream-mute"><code>{step.code}</code></pre>
            <button
              type="button"
              onclick={() => copy(step.code, i)}
              class="absolute right-2 top-2 rounded-md border border-ink-border bg-ink-surface/80 px-2 py-1 text-[10px] uppercase tracking-wider text-mute transition hover:border-honey/40 hover:text-honey"
            >
              {copied === i ? 'copied' : 'copy'}
            </button>
          </div>
        </article>
      {/each}
    </div>
  </div>
</section>
