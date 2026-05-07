<script lang="ts">
    let { probabilities = [], topology = null} = $props();

    let gridSide = $derived(topology?.grid?.side ?? topology?.grid?.side ?? null);

    let isGrid = $derived(gridSide !== null);

    let columns = $derived(isGrid ? gridSide : Math.ceil(Math.sqrt(probabilities.length)));


    function probToStyle(p: number) {
        const hue2 = 220 -p * 40;
        const sat  = 80 + p * 20;           // 80% → 100%
    const lgt  = 8 + p * 82;            // 8% (near black) → 90% (near white)
    const glow = p > 0.15 ? `0 0 ${Math.round(p * 24)}px hsl(${hue2}, 100%, 70%)` : 'none';
    return `
      background: hsl(${hue2}, ${sat}%, ${lgt}%);
      box-shadow: ${glow};
    `;
    }
</script>

<div class="grid-container" style="--columns: {columns};" aria-label="Quantum walk node state grid">
    {#each probabilities as prob, i (i)}
    <div class="node-block" class:active={prob>0.5} style={probToStyle(prob)}
    title="Node {i} - p={prob.toFixed(3)}"
    aria-label="Node {i}, probability {prob.toFixed(3)}"
    >
    <span class="node-index">{i}</span>    
</div>
{/each}
</div>

<style>
   .grid-container {
    display: grid;
    grid-template-columns: repeat(var(--columns), 1fr);
    gap: 4px;
    padding: 4px;
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
  }

  .node-block {
    aspect-ratio: 1;
    border-radius: 4px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    display: flex;
    align-items: flex-end;
    justify-content: flex-end;
    padding: 2px 3px;
    transition:
      background 80ms ease-out,
      box-shadow 80ms ease-out;
    position: relative;
    cursor: default;
  }

  .node-block.active {
    border-color: rgba(120, 220, 255, 0.4);
  }

  .node-index {
    font-size: 0.45rem;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    color: rgba(255, 255, 255, 0.25);
    line-height: 1;
    user-select: none;
    pointer-events: none;
  } 
</style>