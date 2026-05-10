<script lang="ts">
  import { deriveFeed, shortHex } from '$lib/keccak';
  import KeccakChain from '$lib/viz/KeccakChain.svelte';

  let name = $state('dappswarm-demo');
  let owner = $state('a07cBC8d1E2fA5b3C9F8e0A1B2c3D4e5f6A7b8C9');
  let index = $state(0);

  const ownerHexClean = $derived(owner.replace(/^0x/i, '').toLowerCase());
  const ownerValid = $derived(/^[0-9a-f]{40}$/.test(ownerHexClean));
  const chain = $derived(
    ownerValid && name.length > 0
      ? deriveFeed(name, index, ownerHexClean)
      : { topic: '0x' + '00'.repeat(32), id: '0x' + '00'.repeat(32), address: '0x' + '00'.repeat(32) },
  );

  const steps = $derived([
    {
      label: 'topic',
      formula: 'keccak256("dappswarm:" || name)',
      value: chain.topic,
    },
    {
      label: `id @ index ${index}`,
      formula: `keccak256(topic || u64_be(${index}))`,
      value: chain.id,
    },
    {
      label: 'SOC address',
      formula: 'keccak256(id || owner_eoa)',
      value: chain.address,
    },
  ]);
</script>

<section class="hex-section" id="try-live">
  <div class="mx-auto max-w-page">
    <div class="reveal mb-12 max-w-prose">
      <p class="hex-eyebrow mb-4">try it live</p>
      <h2 class="hex-title">Compute a feed address.</h2>
      <p class="hex-lede mt-6">
        The chain is fully deterministic; change a byte of the name or the owner and the SOC address moves.
        That's the whole "no registry" story in one input box.
      </p>
    </div>

    <div class="grid gap-8 lg:grid-cols-[1fr_1.2fr]">
      <div class="hex-card-glow flex flex-col gap-5 reveal">
        <label class="flex flex-col gap-2">
          <span class="hex-eyebrow">package name</span>
          <input
            type="text"
            bind:value={name}
            class="hex-input"
            spellcheck="false"
            autocomplete="off"
          />
        </label>

        <label class="flex flex-col gap-2">
          <span class="hex-eyebrow">owner EOA <span class="text-mute">(40 hex chars)</span></span>
          <input
            type="text"
            bind:value={owner}
            class="hex-input"
            class:input-invalid={!ownerValid}
            spellcheck="false"
            autocomplete="off"
          />
        </label>

        <label class="flex flex-col gap-2">
          <span class="hex-eyebrow">feed index</span>
          <input
            type="number"
            min="0"
            max="999"
            bind:value={index}
            class="hex-input w-32"
          />
        </label>

        <div class="mt-2 rounded-xl border border-honey/20 bg-honey/5 p-4">
          <p class="hex-eyebrow mb-2 !text-honey">resolves to</p>
          <p class="hex-mono break-all text-sm text-honey">
            {chain.address}
          </p>
          <p class="mt-2 text-xs text-mute">
            <span class="hex-mono">GET /chunks/{shortHex(chain.address, 10, 6)}</span>
            on any antd / bee that holds this owner's stamp.
          </p>
        </div>
      </div>

      <div class="reveal" style="transition-delay: 120ms;">
        <KeccakChain {steps} active={2} />
      </div>
    </div>
  </div>
</section>

<style>
  .hex-input {
    background: rgba(10, 8, 7, 0.7);
    border: 1px solid rgba(42, 32, 20, 1);
    border-radius: 0.625rem;
    padding: 0.625rem 0.875rem;
    color: #fef3c7;
    font-family: 'JetBrains Mono Variable', monospace;
    font-size: 0.9rem;
    transition: border-color 200ms ease, background 200ms ease;
  }
  .hex-input:focus {
    outline: none;
    border-color: rgba(251, 191, 36, 0.5);
    background: rgba(10, 8, 7, 0.9);
  }
  .input-invalid {
    border-color: rgba(220, 80, 80, 0.5);
  }
</style>
