<!-- Custom title bar: drag region, STREAM state, minimize / hide to tray. -->
<script lang="ts">
  import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
  import { inTauri } from "../api";
  import { sourceLabel } from "../format";
  import { store } from "../store.svelte";
  import Icon from "./Icon.svelte";
  import Led from "./Led.svelte";

  const win: Window | null = inTauri ? getCurrentWindow() : null;

  const st = $derived(store.snap?.status ?? null);
  const subtitle = $derived(
    st ? `${Math.round(st.soc)} % · ${sourceLabel[st.source]}` : store.snap?.status_error ? "EcoFlow недоступний" : "Читаю стан…",
  );
  const auto = $derived(store.snap?.automation ?? false);
</script>

<header class="bar" data-tauri-drag-region>
  <span class="brand">
    <span class="logo"><Icon name="bolt" size={15} /></span>
    <span class="name">LightOFF</span>
    <span class="ver mono">0.1</span>
  </span>
  <span class="sep" aria-hidden="true"></span>
  <span class="sub">{subtitle}</span>
  <span class="spacer"></span>
  <span class="state"><Led on={auto} />{auto ? "Автоматика працює" : "Автоматику вимкнено"}</span>
  <span class="controls">
    <button class="key wc" aria-label="Згорнути" title="Згорнути" onclick={() => win?.minimize()}>
      <Icon name="minimize" size={16} />
    </button>
    <button class="key wc" aria-label="Сховати в трей" title="Сховати в трей (автоматика продовжить роботу)" onclick={() => win?.close()}>
      <Icon name="to_tray" size={16} />
    </button>
  </span>
</header>

<style>
  .bar {
    height: 44px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 6px 0 14px;
    background: var(--titlebar);
    border-bottom: 1px solid var(--titlebar-edge);
    box-shadow: 0 1px 0 var(--titlebar-hl);
  }
  .brand,
  .sep,
  .sub,
  .spacer,
  .state {
    pointer-events: none;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .logo {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    background: var(--key-dark);
    color: var(--signal);
    box-shadow: var(--dark-face), var(--dark-drop);
  }
  .name {
    font: 600 14px Geologica, sans-serif;
  }
  .ver {
    font-size: 11px;
    font-weight: 500;
    color: var(--engraved);
    text-shadow: var(--engrave-shadow);
  }
  .sep {
    width: 1px;
    height: 18px;
    background: var(--titlebar-edge);
    box-shadow: 1px 0 0 var(--titlebar-hl);
  }
  .sub {
    font-size: 13px;
    color: var(--engraved);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .spacer {
    flex: 1;
  }
  .state {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-right: 10px;
    font-size: 12.5px;
    color: var(--text-2);
    white-space: nowrap;
  }
  .controls {
    display: flex;
    gap: 6px;
  }
  .wc {
    width: 44px;
    height: 30px;
    padding: 0;
  }
</style>
