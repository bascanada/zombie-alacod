<script lang="ts">
    import SvelteMarkdown from '@humanspeak/svelte-markdown'
    import { page } from "$app/state";
	import { onMount } from 'svelte';
    
    let id = page.params.id;

    let source = $state("");

    onMount(async () => {
        const postModule = await import(`../../../lib/blogs/content/${id}.md?raw`);
        source = postModule.default;
    });
</script>



<div class="container mx-auto p-4 max-w-3xl">
  <article class="prose lg:prose-xl p-8 rounded-lg">
    <SvelteMarkdown source={source} />
  </article>

  <div class="mt-8 text-center">
  </div>
</div>



<style lang="postcss">
    @reference "tailwindcss";

    article :global {
        h1 {
            @apply text-6xl text-center m-8;
            font-family: var(--heading-font-family);
        }
    }
</style>