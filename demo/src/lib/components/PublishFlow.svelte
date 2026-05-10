<script lang="ts">
  import { onMount } from 'svelte';
  import KeccakChain from '$lib/viz/KeccakChain.svelte';
  import MantarayTree from '$lib/viz/MantarayTree.svelte';
  import ChunkStream from '$lib/viz/ChunkStream.svelte';
  import { deriveFeed } from '$lib/keccak';

  const SAMPLE_NAME = 'dappswarm-demo';
  const SAMPLE_OWNER = 'a07cBC8d1E2fA5b3C9F8e0A1B2c3D4e5f6A7b8C9';
  const SAMPLE_INDEX = 0;

  const chain = deriveFeed(SAMPLE_NAME, SAMPLE_INDEX, SAMPLE_OWNER);

  let stage = $state(-1);
  let started = $state(false);
  let container: HTMLElement;

  onMount(() => {
    const obs = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting && !started) {
            started = true;
            const interval = window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 80 : 1100;

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

  const keccakSteps = [
    {
      label: 'topic',
      formula: 'keccak256("dappswarm:" || name)',
      value: chain.topic,
    },
    {
      label: 'id @ index 0',
      formula: 'keccak256(topic || u64_be(0))',
      value: chain.id,
    },
    {
      label: 'SOC address',
      formula: 'keccak256(id || owner_eoa)',
      value: chain.address,
    },
  ];
</script>

<section class="hex-section" id="how-it-works" bind:this={container}>
  <div class="mx-auto max-w-page">
    <div class="reveal mb-12 max-w-prose">
      <p class="hex-eyebrow mb-4">flow #1, publish</p>
      <h2 class="hex-title">Pack. Upload. Index. Sign.</h2>
      <p class="hex-lede mt-6">
        Four stages, end-to-end, in well under a second. The output is a 32-byte bzz reference and a feed entry
        the entire network can find from <span class="hex-mono text-honey">(owner, name)</span> alone.
      </p>
    </div>

    <div class="grid gap-8 lg:grid-cols-2">
      <!-- left: three stage cards -->
      <div class="flex flex-col gap-5">
        <article
          class="hex-card-glow flow-step"
          class:flow-step-active={stage >= 0}
        >
          <div class="flow-head">
            <span class="flow-num">1</span>
            <h3 class="flow-title">Pack the bundle</h3>
          </div>
          <p class="flow-body">
            <span class="hex-mono text-honey">bundle::pack</span> emits an uncompressed tar of the directory.
            Swarm chunks dedupe automatically; recompressing here would defeat that.
          </p>
          <div class="mt-4 h-16">
            <ChunkStream active={stage >= 0} count={6} />
          </div>
        </article>

        <article
          class="hex-card-glow flow-step"
          class:flow-step-active={stage >= 1}
        >
          <div class="flow-head">
            <span class="flow-num">2</span>
            <h3 class="flow-title">Upload to /bzz</h3>
          </div>
          <p class="flow-body">
            POST as a tar collection with <span class="hex-mono text-honey">swarm-collection: true</span>.
            antd splits it into chunks and assembles a Mantaray manifest. The response is the root reference.
          </p>
          <div class="mt-4 grid h-32 place-items-center">
            <MantarayTree active={stage >= 1} />
          </div>
        </article>

        <article
          class="hex-card-glow flow-step"
          class:flow-step-active={stage >= 3}
        >
          <div class="flow-head">
            <span class="flow-num">4</span>
            <h3 class="flow-title">Sign + push the SOC</h3>
          </div>
          <p class="flow-body">
            secp256k1 signature over <span class="hex-mono text-honey">keccak256(id || inner_addr)</span>.
            POST to <span class="hex-mono text-honey">/soc/{`{owner}/{id}`}</span>. The feed now points at the
            new bundle.
          </p>
          <div class="mt-4 flex flex-wrap items-center gap-2 text-xs">
            <span class="hex-pill"><span class="hex-mono text-honey">⬡</span> envelope</span>
            <span class="hex-pill hex-mono text-honey">id || sig || span || data</span>
            <span class="hex-pill"><span class="hex-mono text-honey">⤳</span> antd</span>
          </div>
        </article>
      </div>

      <!-- right: keccak chain (stage 3) -->
      <div class="lg:sticky lg:top-24 lg:self-start">
        <article class="hex-card-glow flow-step" class:flow-step-active={stage >= 2}>
          <div class="flow-head">
            <span class="flow-num">3</span>
            <h3 class="flow-title">Index: the keccak chain</h3>
          </div>
          <p class="flow-body mb-5">
            The feed address is fully deterministic: it falls out of three keccak rounds over the package name,
            the index, and the owner's EOA. No registry asked.
          </p>
          <KeccakChain steps={keccakSteps} active={stage - 1} />
        </article>
      </div>
    </div>
  </div>
</section>

<style>
  .flow-step {
    transition:
      transform 600ms cubic-bezier(0.2, 0.8, 0.2, 1),
      opacity 600ms cubic-bezier(0.2, 0.8, 0.2, 1),
      border-color 600ms ease;
    opacity: 0.55;
    transform: translateY(8px);
  }
  .flow-step-active {
    opacity: 1;
    transform: translateY(0);
    border-color: rgba(251, 191, 36, 0.35);
    box-shadow:
      0 0 0 1px rgba(251, 191, 36, 0.18) inset,
      0 18px 40px -10px rgba(251, 191, 36, 0.18);
  }
  .flow-head {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }
  .flow-num {
    display: inline-grid;
    place-items: center;
    width: 1.75rem;
    height: 1.75rem;
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
  .flow-body {
    color: #d6d3cb;
  }
</style>
