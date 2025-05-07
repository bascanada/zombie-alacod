<script lang="ts">
  	import { Navigation } from '@skeletonlabs/skeleton-svelte';
    import { House, Gamepad, Settings } from '@lucide/svelte/icons';

  	import '../app.css';
	  let { children } = $props();
</script>

<div class="h-full w-full flex flex-row">
  <div class="h-full z-50 bg-surface-100-800-token border-t border-surface-300-600-token">
    <Navigation.Rail>
        {#snippet header()}
          <Navigation.Tile label="" href="/"><img class="logo" src="/icons/android-chrome-192x192.png" alt="logo" /></Navigation.Tile>
        {/snippet}
        {#snippet tiles()}
          <Navigation.Tile label="Home" href="/"><House /></Navigation.Tile>
          <Navigation.Tile label="Game" href="/game"><Gamepad /></Navigation.Tile>
        {/snippet}

        {#snippet footer()}
          <Navigation.Tile label="Settings" href="/settings"><Settings /></Navigation.Tile>
        {/snippet}
    </Navigation.Rail>
  </div>

  <div class="h-full w-full flex flex-col">
    <div class="card border-surface-100-900 grid w-full grid-rows-[auto_1fr]">
        {@render children()}
    </div>
  </div>

</div>

<style>
  .logo {
    border-radius: 10px;
  }
</style>

