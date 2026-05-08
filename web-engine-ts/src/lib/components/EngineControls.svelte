<script lang=ts>
	import { setSpeed, setTopology } from "$lib/engine/engine.svelte";
	import { toErrorMessage } from "$lib/types/errorMessage";


    let selectedTopology =$state('grid');
    let gridSide = $state(6);
    let ringNodes = $state(20);
    let ringChord = $state(3);
    let tickMs = $state(120);
    let busy = $state(false);
    let message = $state('');

    async function applyTopology() {
        busy = true;
        message = '';
        try {
            let res;
            if (selectedTopology === 'grid') {
                res = await setTopology('grid', { side: Number(gridSide) });
               
            } else {
                res = await setTopology('ring', {nodes: Number(ringNodes), chord_step: Number(ringChord) });
               

            }
            message = res.message;
        } catch (error) {
            message = toErrorMessage(error);
        } finally {
            busy = false;
        }
    }

    async function applySpeed() {
        busy = true;
        message = '';
        try {
            const res = await setSpeed(Number(tickMs));
            message = res.message;
        } catch (error) {
            message = toErrorMessage(error);
        } finally {
            busy = false;
        }
    }
</script>

<aside class="controls">
    <h2 class="controls-title">Engine Controls</h2>

    <section class="control-group">
        <label for="" class="label">Topology</label>
        <div class="radio-row">
            <label for="" class="radio-label">
                <input type="radio" bind:group={selectedTopology} value="grid" >
                Grid
            </label>
            <label for="" class="radio-label">
                <input type="radio" bind:group={selectedTopology} value="ring" >
                Ring
            </label>
        </div>

        {#if selectedTopology === 'grid'}
        <label for="" class="label">Side lenght ({gridSide}x{gridSide} = {gridSide*gridSide} nodes)</label>
        <input type="range" class="slider" min = "2" max = "12" bind:value={gridSide}/>
        {:else}
        <label for="" class="label">Nodes ({ringNodes})</label>
        <input type="range" class="slider" min="3" max="64" bind:value={ringNodes}/>
        <label for="" class="label">Chord step ({ringChord})</label>
        <input type="range" class="slider" min="1" max={Math.floor(ringNodes/2)} bind:value={ringChord}/>
        {/if}

        <button class="btn" onclick={applyTopology} disabled={busy}>
            Apply Topology
        </button>
    </section>

    <section class="control-group">
        <label for="" class="label">Tick speed - {tickMs}ms / step</label>
        <input type="range" class="slider" min="50" max="1000" step="10" bind:value={tickMs} />
        <button class="btn" onclick={applySpeed} disabled={busy}>
            Apply speed
        </button>
    </section>
    {#if message}
        <p class="status-msg">{message}</p>
    {/if}
</aside>

<style>
 .controls {
    background: rgba(255,255,255,0.03);
    border: 1px solid rgba(255,255,255,0.08);
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 220px;
  }

  .controls-title {
    font-size: 0.7rem;
    font-family: 'JetBrains Mono', monospace;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: rgba(255,255,255,0.35);
    margin: 0 0 0.25rem;
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid rgba(255,255,255,0.06);
  }
  .control-group:last-of-type { border-bottom: none; }

  .label {
    font-size: 0.72rem;
    color: rgba(255,255,255,0.5);
    font-family: 'JetBrains Mono', monospace;
  }

  .radio-row {
    display: flex;
    gap: 1rem;
  }
  .radio-label {
    font-size: 0.8rem;
    color: rgba(255,255,255,0.7);
    display: flex;
    align-items: center;
    gap: 0.35rem;
    cursor: pointer;
  }

  .slider {
    width: 100%;
    accent-color: #7cdcff;
    cursor: pointer;
  }

  .btn {
    margin-top: 0.25rem;
    background: rgba(120,220,255,0.1);
    border: 1px solid rgba(120,220,255,0.3);
    color: #7cdcff;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    padding: 0.45rem 0.75rem;
    border-radius: 4px;
    cursor: pointer;
    transition: background 150ms;
  }
  .btn:hover:not(:disabled) { background: rgba(120,220,255,0.18); }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }

  .status-msg {
    font-size: 0.72rem;
    font-family: 'JetBrains Mono', monospace;
    color: #7cdcff;
    margin: 0;
    opacity: 0.75;
  }
</style>