<script lang="ts">
  export let id = "id";
  export let title = 'Card Title';
  export let content = 'Card content goes here.';
</script>

<a
  href={`/play?id=${id}`}
  class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover divide-surface-200-800 block max-w-md divide-y overflow-hidden"
>
  <header>
    <h3 class="h3 m-5 text-left">{title}</h3>
  </header>
  <article class="space-y-4 p-4 h-30">
    <p class="opacity-61">
      {content}
    </p>
  </article>
  <footer class="flex items-center justify-between gap-4 p-4">
    <small class="opacity-60">Experimental</small>
  </footer>
</a>