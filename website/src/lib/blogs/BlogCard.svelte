<script lang="ts">
	import Card from "$lib/layouts/Card.svelte";

  export let id = "id";

  export let title = 'Card Title';
  export let content = 'Card content goes here.';

  export let date = "01/01/01";
  export let version = "1.0.9";

  export let latest = false;

  let footers = [{ text: date, classes: "opacity-60" }, { text: version, classes: "opacity-60" },]
  if (latest) {
    footers.push({ text: "Latest Update", classes: "badge preset-tonal-secondary"})
    console.log("updating");
  }
</script>


<Card
  {title}
  {content}
  {footers}
  href={"/blog/" + id}/>
