<script lang="ts">
  const layers = [
    {
      eyebrow: 'L4 / CLI',
      title: 'dappswarm doctor / publish / resolve / install / info',
      body: 'Five subcommands. Each is a tiny orchestrator that calls into the layered modules below. The CLI is the surface a Dappnode operator actually touches.',
    },
    {
      eyebrow: 'L3 / orchestrators',
      title: 'publish · resolve · install',
      body: 'Pack the bundle, post to /bzz, find the next feed index, sign and push the SOC. On the resolve side: probe for latest, decode, walk the manifest, run the compose stack.',
    },
    {
      eyebrow: 'L2 / primitives',
      title: 'swarm · soc · feed · bundle',
      body: 'Four modules with sharp edges: a thin reqwest client, a SOC builder, a sequence-indexed feed convention, and a tar-collection bundle model. Each fits in one screen.',
    },
    {
      eyebrow: 'L1 / node',
      title: 'antd (with the SOC patch)',
      body: 'A local Rust Swarm node with a one-route addition: POST /soc/{owner}/{id}. Everything else is bee-compatible: bzz uploads, manifest walks, chunk reads.',
    },
    {
      eyebrow: 'L0 / Swarm',
      title: 'Swarm network',
      body: 'Content-addressed chunks, replicated and propagated by the network. Mantaray manifests give bundles integrity. Postage stamps pay for storage.',
    },
  ];
</script>

<section class="hex-section" id="architecture">
  <div class="mx-auto max-w-page">
    <div class="reveal mb-16 max-w-prose">
      <p class="hex-eyebrow mb-4">architecture</p>
      <h2 class="hex-title">Five layers, four primitives.</h2>
      <p class="hex-lede mt-6">
        The whole system is a thin stack with strict boundaries. You can read it in an afternoon.
      </p>
    </div>

    <div class="grid gap-8 lg:grid-cols-[2fr_3fr] lg:gap-12">
      <!-- diagram column -->
      <div class="lg:sticky lg:top-24 lg:self-start">
        <div class="hex-card-glow flex flex-col gap-3 p-5">
          {#each layers as layer, i (layer.eyebrow)}
            <div
              class="reveal arch-layer flex items-center justify-between rounded-xl border border-ink-border bg-ink-raised/60 px-4 py-3"
              style="transition-delay: {i * 80}ms;"
            >
              <div>
                <p class="hex-mono text-[10px] uppercase tracking-[0.2em] text-honey/80">{layer.eyebrow}</p>
                <p class="text-sm text-cream">{layer.title.split(' · ').join('  ·  ').slice(0, 60)}</p>
              </div>
              <div class="hex-mono text-xs text-mute">L{4 - i}</div>
            </div>
          {/each}
        </div>
        <p class="mt-4 text-xs text-mute">Each layer only depends on the layer below it.</p>
      </div>

      <!-- narration column -->
      <div class="flex flex-col gap-10">
        {#each layers as layer, i (layer.eyebrow)}
          <article class="reveal" style="transition-delay: {(i % 3) * 100}ms;">
            <p class="hex-eyebrow">{layer.eyebrow}</p>
            <h3 class="mt-2 font-display text-2xl font-bold text-cream">{layer.title}</h3>
            <p class="mt-3 text-cream-mute">{layer.body}</p>
          </article>
        {/each}
      </div>
    </div>
  </div>
</section>

<style>
  .arch-layer {
    transition:
      background 400ms ease,
      border-color 400ms ease,
      transform 400ms ease;
  }
  .arch-layer:hover {
    border-color: rgba(251, 191, 36, 0.4);
    background: rgba(251, 191, 36, 0.05);
  }
</style>
