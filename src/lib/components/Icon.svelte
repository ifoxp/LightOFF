<!-- One stroke icon from lib/icons.ts. Decorative unless `label` is set. -->
<script lang="ts">
  import { iconPaths } from "../icons";

  let { name, size = 20, label }: { name: string; size?: number; label?: string } = $props();
  const paths = $derived(iconPaths(name));
</script>

<svg
  class="i"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  role={label ? "img" : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : "true"}
>
  {#each paths as d, i (i)}<path {d} />{/each}
</svg>
