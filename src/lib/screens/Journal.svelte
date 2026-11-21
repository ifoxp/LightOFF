<!-- «Журнал»: what the app did and why, newest first. -->
<script lang="ts">
  import { api } from "../api";
  import Icon from "../components/Icon.svelte";
  import Section from "../components/Section.svelte";
  import { store } from "../store.svelte";

  const icon = { ok: "check", info: "info", warn: "warning", error: "error" } as const;
</script>

<Section title="Журнал">
  {#snippet aside()}
    <button class="key" onclick={() => store.run("dir", api.openDataDir)}><Icon name="folder" size={16} />Папка з логами</button>
  {/snippet}
  {#if (store.snap?.journal ?? []).length}
    <ul class="log">
      {#each store.snap?.journal ?? [] as e, i (i)}
        <li class={e.level}>
          <span class="mono ts">{e.ts}</span>
          <Icon name={icon[e.level] ?? "dot"} size={16} />
          <span class="selectable">{e.text}</span>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="muted">Поки що порожньо.</p>
  {/if}
</Section>

<style>
  .log {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }
  .log li {
    display: grid;
    grid-template-columns: 124px 20px 1fr;
    gap: 8px;
    align-items: start;
    padding: 7px 0;
    font-size: 13px;
    border-bottom: 1px solid var(--groove-a);
    box-shadow: 0 1px 0 var(--groove-b);
  }
  .ts {
    font-size: 12px;
    color: var(--engraved);
  }
  .warn,
  .error {
    color: var(--danger-text);
  }
</style>
