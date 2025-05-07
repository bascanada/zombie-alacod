<script lang="ts">
    import GameCard from '$lib/game/GameCard.svelte';
    import applications from '$lib/game/applications.json';
    
    const items = applications;
</script>


<div>
    <h1 class="font-metal-mania text-8xl text-center m-8">Zombie alacod</h1>
</div>

{#if items && items.length > 0}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 m-8">
    {#each items as item (item.id)}
        <div><GameCard id={item.id} title={item.name} content={item.content} /></div>
    {/each}
    </div>

{/if}
