<script lang="ts">
  export let id = "";
  export let height: string = "64px";
  export let color = '#FF0000';

  let svgContent = '';

  $: if (id) {
    import(`../lib/external-icons/${id}.svg?raw`)
      .then(module => {
        svgContent = module.default;
      })
      .catch(err => {
        console.error(`Failed to load icon ${id}:`, err);
        svgContent = '<!-- SVG failed to load -->';
      });
  }
</script>

<div class="icon-wrapper" style="--size: {height}; --color: {color}">
  {@html svgContent}
</div>

<style>
  .icon-wrapper :global(svg) {
    width: var(--size);
    height: var(--size);
    fill: var(--color);
  }
</style>