<script lang="ts">
  const customAppVersion: string = import.meta.env.VITE_APP_VERSION || "DEV";

  const sanitizeVersion = customAppVersion.split("-")[0];
</script>

<a href="https://github.com/bascanada/zombie-alacod/releases/tag/{sanitizeVersion}"><small class="badge preset-tonal-secondary">{sanitizeVersion}</small></a>