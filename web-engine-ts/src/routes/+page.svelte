<script>
	import EngineControls from "$lib/components/EngineControls.svelte";
	import QuantumGrid from "$lib/components/QuantumGrid.svelte";
    import { engine } from '$lib/engine/engine.svelte';
	const probabilities = $derived(engine.snapshot?.probabilities ?? []);
    const topology      = $derived(engine.current?.topology ?? null);
    //const step          = $derived(engine.current?.step ?? 0);

</script>

<svelte:head>
    <title>QWalk Network Engine</title>
    <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link
    href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@300;400;600&family=JetBrains+Mono:wght@400;700&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<main class="shell">
    <header class="header">
        <div class="header-title">
            <span class="logo-mark">⬡</span>
            <h1>QWalk <span class="subtitle">ENGINE</span></h1>
        </div>
    </header>
    <div class="body">

    <!-- Visualisation panel -->
    <section class="vis-panel">
      <p class="panel-label">QUANTUM WALK — NODE PROBABILITY FIELD</p>

      {#if probabilities.length > 0}
        <QuantumGrid
          probabilities={probabilities}
          topology={topology}
          //step={$step}
        />
      {:else}
        <div class="connecting-hint">
          <span class="pulse">◌</span>
          Waiting for engine…
        </div>
      {/if}

      <!-- Colour legend -->
      <div class="legend">
        <div class="legend-bar"></div>
        <div class="legend-labels">
          <span>p = 0</span>
          <span>p = 1</span>
        </div>
      </div>
    </section>

    <!-- Controls panel -->
    <aside class="sidebar">
      <EngineControls />

      <!-- Info card -->
      <div class="info-card">
        <h3>About</h3>
        <p>
          A discrete-time quantum walk (DTQW) propagates probability amplitudes
          across a graph using a Grover diffusion coin operator.
          Each block represents one node; brightness encodes the probability
          of the walker being found there.
        </p>
        <p>
          The Rust engine steps the walk forward every tick and streams
          snapshots to this page via WebSocket.
        </p>
      </div>
    </aside>

  </div>
</main>

<style>
    .shell {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    padding: 1.5rem 2rem;
    gap: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  /* ── Header ── */
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 1rem;
    border-bottom: 1px solid rgba(255,255,255,0.06);
    padding-bottom: 1rem;
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .logo-mark {
    font-size: 1.6rem;
    color: #7cdcff;
    line-height: 1;
  }

  h1 {
    font-family: 'JetBrains Mono', monospace;
    font-size: 1.1rem;
    font-weight: 700;
    letter-spacing: 0.18em;
    color: #e8eaf0;
  }

  .subtitle {
    color: rgba(255,255,255,0.35);
    font-weight: 400;
  }
</style>