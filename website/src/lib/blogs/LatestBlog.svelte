<script lang="ts">
	import BlogCard from '$lib/blogs/BlogCard.svelte';
    import blogs from '$lib/blogs/content/index.json';
    
    const item = blogs[0];
</script>

{#if item}
    <BlogCard id={item.id} title={item.title} content={item.content} date={item.date} version={item.version} latest={true} />
{/if}