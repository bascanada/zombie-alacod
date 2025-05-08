<script lang="ts">
  	import { Navigation } from '@skeletonlabs/skeleton-svelte';
    import { House, Settings, ToiletIcon, Gamepad2 } from '@lucide/svelte/icons';
    import { Toaster } from '@skeletonlabs/skeleton-svelte';
    import { toaster } from '$lib/toaster';

  	import '../app.css';
	  import CurrentVersion from '$lib/game/CurrentVersion.svelte';

	  let { children } = $props();
</script>

<div class="h-full w-full flex flex-row">
  <div class="h-full z-50 bg-surface-100-800-token border-t border-surface-300-600-token">
    <Navigation.Rail>
        {#snippet header()}
          <Navigation.Tile label="" href="/"><img class="logo" src="/icons/android-chrome-192x192.png" alt="logo" /></Navigation.Tile>
          <CurrentVersion></CurrentVersion>
        {/snippet}
        {#snippet tiles()}
          <Navigation.Tile label="Home" href="/"><House /></Navigation.Tile>
          <Navigation.Tile label="Blog" href="/blog"><House /></Navigation.Tile>
          <Navigation.Tile label="Tools" href="/tools"><ToiletIcon /></Navigation.Tile>
        {/snippet}

        {#snippet footer()}
          <Navigation.Tile label="Settings" href="/settings"><Settings /></Navigation.Tile>
        {/snippet}
    </Navigation.Rail>
  </div>

  {@render children()}

</div>

<Toaster {toaster}></Toaster>

<style>
  .logo {
    border-radius: 10px;
  }
</style>

