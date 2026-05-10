<script lang="ts">
  import { shortHex } from '$lib/keccak';

  interface Step {
    label: string;
    formula: string;
    value: string;
  }

  let { steps, active = -1 }: { steps: Step[]; active?: number } = $props();
</script>

<div class="flex flex-col gap-3">
  {#each steps as step, i (step.label)}
    <div
      class="hex-card-glow flex flex-col gap-2 transition-all duration-500"
      class:keccak-step-active={active >= i}
      class:keccak-step-current={active === i}
      style="--i: {i}"
    >
      <div class="flex items-baseline justify-between gap-3">
        <div class="flex items-center gap-2">
          <span
            class="hex-mono inline-flex h-6 min-w-[1.5rem] items-center justify-center rounded-md border border-honey/30 px-1.5 text-xs text-honey"
          >
            {i + 1}
          </span>
          <span class="hex-eyebrow !text-cream">{step.label}</span>
        </div>
        <span class="hex-mono text-xs text-mute">keccak256</span>
      </div>
      <div class="hex-mono text-xs text-cream-mute">{step.formula}</div>
      <div class="hex-hash hex-mono text-sm sm:text-base">{shortHex(step.value, 14, 8)}</div>
      <div class="keccak-bar"></div>
    </div>
  {/each}
</div>

<style>
  .keccak-step-active {
    border-color: rgba(251, 191, 36, 0.35);
    box-shadow:
      0 0 0 1px rgba(251, 191, 36, 0.18) inset,
      0 12px 32px -10px rgba(251, 191, 36, 0.18);
  }
  .keccak-step-current {
    box-shadow:
      0 0 0 1px rgba(251, 191, 36, 0.5) inset,
      0 18px 48px -10px rgba(251, 191, 36, 0.45);
  }
  .keccak-bar {
    height: 2px;
    background: linear-gradient(90deg, transparent, #fbbf24, transparent);
    transform: scaleX(0);
    transform-origin: left;
    transition: transform 600ms cubic-bezier(0.2, 0.8, 0.2, 1);
    transition-delay: calc(var(--i) * 80ms);
  }
  .keccak-step-active .keccak-bar {
    transform: scaleX(1);
  }
</style>
