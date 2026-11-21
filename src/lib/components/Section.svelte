<!-- A titled group separated from the next one by a groove (no shadowed cards). -->
<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, aside, children }: { title: string; aside?: Snippet; children: Snippet } = $props();
</script>

<section class="section">
  <div class="head">
    <h2 class="engraved">{title}</h2>
    {#if aside}<div class="aside">{@render aside()}</div>{/if}
  </div>
  {@render children()}
</section>

<style>
  .section {
    padding: 18px 0 20px;
    border-bottom: 1px solid var(--groove-a);
    box-shadow: 0 1px 0 var(--groove-b);
  }
  .section:last-child {
    border-bottom: 0;
    box-shadow: none;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
    min-height: 30px;
  }
  h2 {
    font-weight: 500;
  }
  .aside {
    display: flex;
    gap: 8px;
    align-items: center;
  }
</style>
