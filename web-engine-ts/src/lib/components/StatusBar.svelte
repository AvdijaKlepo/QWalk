<script lang="ts">
    let { connected = false, step = 0, nodeCount = 0, tickMs = 0, reconnectCount = 0 } = $props();
</script>

<div class="status-bar">
    <span class="indicator" class:online={connected} class:offline={!connected}>
        {connected ? '● LIVE' : reconnectCount > 0 ? `↺ RECONNECTING (${reconnectCount})` : '○ CONNECTING'}
    </span>

    <span class="divider">|</span>
    <span class="metric">STEP <strong>{step.toLocaleString()}</strong></span>

    <span class="divider">|</span>
    <span class="metric">NODES <strong>{nodeCount}</strong></span>

    <span class="divicer">|</span>
    <span class="metric">TICK <strong>{tickMs}ms</strong></span>
</div>

<style>
    .status-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    color: rgba(255,255,255,0.35);
    padding: 0.5rem 0;
  }

  .indicator {
    font-weight: 700;
    letter-spacing: 0.12em;
  }
  .indicator.online  { color: #4dffb0; }
  .indicator.offline { color: #ff6b6b; }

  .divider { opacity: 0.2; }

  .metric strong {
    color: rgba(255,255,255,0.65);
    font-weight: 600;
  }
</style>