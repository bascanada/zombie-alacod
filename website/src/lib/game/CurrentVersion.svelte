<script lang="ts">
  const customAppVersion: string = import.meta.env.VITE_APP_VERSION || "DEV";
</script>

<small class="opacity-60">{customAppVersion}</small>