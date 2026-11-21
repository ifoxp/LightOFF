<!-- Shell: title bar, navigation keys on the left, the current screen on the right. -->
<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "./lib/components/Icon.svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import Journal from "./lib/screens/Journal.svelte";
  import Schedule from "./lib/screens/Schedule.svelte";
  import SettingsScreen from "./lib/screens/Settings.svelte";
  import Status from "./lib/screens/Status.svelte";
  import { store, type Screen } from "./lib/store.svelte";

  const nav: { id: Screen; label: string; icon: string }[] = [
    { id: "status", label: "Стан", icon: "bolt" },
    { id: "schedule", label: "Графік", icon: "calendar" },
    { id: "journal", label: "Журнал", icon: "list" },
    { id: "settings", label: "Налаштування", icon: "settings" },
  ];

  onMount(() => {
    store.start();
    return () => store.stop();
  });

  // First run: nothing configured yet -> open the settings.
  let routed = false;
  $effect(() => {
    if (!routed && store.snap) {
      routed = true;
      if (!store.snap.has_api) store.screen = "settings";
    }
  });
</script>

<div class="shell">
  <TitleBar />
  <div class="body">
    <nav class="side" aria-label="Розділи">
      {#each nav as n (n.id)}
        <button class="key navkey" aria-pressed={store.screen === n.id} onclick={() => (store.screen = n.id)}>
          <Icon name={n.icon} size={18} />{n.label}
        </button>
      {/each}
      <span class="grow"></span>
      <span class="engraved foot">EcoFlow · ВОЕ</span>
    </nav>
    <main class="screen">
      {#if store.screen === "status"}
        <Status />
      {:else if store.screen === "schedule"}
        <Schedule />
      {:else if store.screen === "journal"}
        <Journal />
      {:else}
        <SettingsScreen />
      {/if}
    </main>
  </div>
  <Toasts />
</div>

<style>
  .shell {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .side {
    width: 196px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px 14px;
    border-right: 1px solid var(--groove-a);
    box-shadow: 1px 0 0 var(--groove-b);
  }
  .navkey {
    justify-content: flex-start;
    height: 40px;
    border-radius: 10px;
    gap: 10px;
    font-size: 13.5px;
  }
  .navkey[aria-pressed="true"] {
    background: var(--chosen-bg);
    color: var(--chosen-fg);
    box-shadow: var(--chosen-shadow);
    transform: none;
  }
  .grow {
    flex: 1;
  }
  .foot {
    font-size: 10px;
    padding: 0 4px;
  }
  .screen {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 6px 28px 28px;
  }
</style>
