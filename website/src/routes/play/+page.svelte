<script lang="ts">
    import { onMount } from 'svelte';

    let src = "";

    onMount(() => {
        const urlParams = new URLSearchParams(window.location.search);
        const id = urlParams.get('id');
        let lobby = "test";

        src = `/loader.html?name=${id}&lobby=${lobby}`;
    });


</script>
<iframe id="app-frame" title="game iframe" src={src} style="width: 100%; border: none; height: 100vh"></iframe>