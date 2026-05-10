<script lang="ts">
  let { density = 18, opacity = 0.18 }: { density?: number; opacity?: number } = $props();

  const hexes = $derived(Array.from({ length: density }, (_, i) => ({
    x: ((i * 79) % 100) + ((i % 3) - 1) * 6,
    y: ((i * 53) % 100) + ((i % 5) - 2) * 4,
    delay: (i % 9) * 0.4,
    size: 14 + (i % 4) * 6,
  })));
</script>

<div class="pointer-events-none absolute inset-0 overflow-hidden" style="opacity: {opacity}">
  <svg
    class="absolute inset-0 h-full w-full"
    viewBox="0 0 100 100"
    preserveAspectRatio="xMidYMid slice"
    aria-hidden="true"
  >
    {#each hexes as hex (hex.x + ':' + hex.y)}
      <g
        transform="translate({hex.x} {hex.y})"
        style="--d: {hex.delay}s"
        class="hex-float"
      >
        <polygon
          points="0,-{hex.size / 10} {hex.size / 11.5},-{hex.size / 20} {hex.size / 11.5},{hex.size / 20} 0,{hex.size / 10} -{hex.size / 11.5},{hex.size / 20} -{hex.size / 11.5},-{hex.size / 20}"
          fill="none"
          stroke="rgba(251, 191, 36, 0.5)"
          stroke-width="0.12"
        />
      </g>
    {/each}
  </svg>
</div>

<style>
  .hex-float {
    animation: hex-bob 9s ease-in-out infinite;
    animation-delay: var(--d);
    transform-box: fill-box;
    transform-origin: center;
  }
  @keyframes hex-bob {
    0%,
    100% {
      transform: translate(var(--ox, 0), var(--oy, 0)) rotate(0deg);
      opacity: 0.6;
    }
    50% {
      transform: translate(var(--ox, 0), var(--oy, 0)) translateY(-1px) rotate(8deg);
      opacity: 1;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .hex-float {
      animation: none;
    }
  }
</style>
