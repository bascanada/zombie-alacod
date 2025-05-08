<script lang="ts">
	import BlogCard from '$lib/blogs/BlogCard.svelte';
	import blogs from '$lib/blogs/content/index.json';
	import CardGrid from '$lib/layouts/CardGrid.svelte';
</script>

<CardGrid items={blogs}>
	{#snippet content(item)}
		<BlogCard id={item.id} title={item.title} content={item.content} date={item.date} version={item.version} />
	{/snippet}
</CardGrid>
