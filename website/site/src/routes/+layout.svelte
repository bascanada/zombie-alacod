<script lang="ts">
  	import { Navigation } from '@skeletonlabs/skeleton-svelte';

	import '../app.css';
	let { children } = $props();
</script>

<h1 class="font-metal-mania text-8xl">Hello child of Death</h1>

<div class="card border-surface-100-900 grid w-full grid-rows-[auto_1fr] border-[1px]">
  <!-- Content -->
  <div class="flex items-center justify-center">
	<div style="flex-grow: 1;">
		{@render children()}
	</div>
  </div>
  <!-- Component -->

</div>

<div class="fixed bottom-0 left-0 right-0 z-50 bg-surface-100-800-token border-t border-surface-300-600-token">
   <Navigation.Bar>
    <Navigation.Tile label="Files" href="#/files">Test</Navigation.Tile>
    <Navigation.Tile label="Images" href="#/images">TEst</Navigation.Tile>
    <Navigation.Tile label="Music" href="#/music">Test</Navigation.Tile>
    <Navigation.Tile label="Videos" href="#/videos">Test</Navigation.Tile>
  </Navigation.Bar>
</div>