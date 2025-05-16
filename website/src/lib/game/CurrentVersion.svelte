<script lang="ts">
  const customAppVersion: string = import.meta.env.VITE_APP_VERSION || "DEV";

  const versionParts = customAppVersion.split("-");
  const sanitizeVersion = versionParts[0];
  const snapshot = versionParts[1];
</script>

<a href="https://github.com/bascanada/zombie-alacod/releases/tag/{sanitizeVersion}"><small class="badge preset-tonal-secondary">{sanitizeVersion}</small></a>

{#if snapshot}
<a href="https://github.com/bascanada/zombie-alacod/releases/tag/{sanitizeVersion}"><small class="badge preset-tonal-secondary">{snapshot}</small></a>
{/if}
