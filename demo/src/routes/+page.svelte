<script lang="ts">
  import Hero from '$lib/components/Hero.svelte';
  import WhyCards from '$lib/components/WhyCards.svelte';
  import Architecture from '$lib/components/Architecture.svelte';
  import PublishFlow from '$lib/components/PublishFlow.svelte';
  import ResolveFlow from '$lib/components/ResolveFlow.svelte';
  import SocDeriver from '$lib/components/SocDeriver.svelte';
  import LiveFeed from '$lib/components/LiveFeed.svelte';
  import Terminal from '$lib/components/Terminal.svelte';
  import TryItYourself from '$lib/components/TryItYourself.svelte';
  import Footer from '$lib/components/Footer.svelte';
</script>

<main class="hex-bg min-h-screen">
  <Hero />
  <WhyCards />
  <Architecture />
  <PublishFlow />
  <ResolveFlow />
  <SocDeriver />
  <LiveFeed />
  <Terminal />
  <TryItYourself />
  <Footer />
</main>
