<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { initEngine } from '$lib/engine/engine.svelte';

	onMount(() => {
		return initEngine();
	})

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
{@render children()}


<style>
	:global(*, *::before, *::after) {
		box-sizing: border-box;
		margin: 0;
		padding: 0;
	}

	:global(html, body) {
		height: 100%;
		background: #06060f;
		color: #e8eaf0;
		font-family: 'IBM Plex Sans', 'Segoe UI', system-ui, sans-serif;
		-webkit-font-smoothing: antialiased;
	}

	:global(a) { color: #7cdcff; }
</style>
