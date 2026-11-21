<!-- «Стан»: live STREAM readings, manual switching, automation and the next planned switches. -->
<script lang="ts">
  import { api } from "../api";
  import Icon from "../components/Icon.svelte";
  import Section from "../components/Section.svelte";
  import Switch from "../components/Switch.svelte";
  import { hm, isToday, sourceLabel, watts } from "../format";
  import { store } from "../store.svelte";

  const snap = $derived(store.snap);
  const st = $derived(snap?.status ?? null);
  const upcoming = $derived((snap?.plan ?? []).filter((p) => !p.past).slice(0, 6));
  let reserveInput = $state(50);

  function when(iso: string) {
    return isToday(iso) ? hm(iso) : `${iso.slice(8, 10)}.${iso.slice(5, 7)} ${hm(iso)}`;
  }
</script>

<Section title="EcoFlow STREAM">
  {#snippet aside()}
    {#if st}<span class="muted small">оновлено {st.read_at}</span>{/if}
    <button class="key icon-only" title="Оновити зараз" aria-label="Оновити зараз" onclick={() => store.run("status", api.refreshStatus)}>
      <Icon name="refresh" size={16} />
    </button>
  {/snippet}

  {#if !snap?.has_api}
    <div class="well note">
      <Icon name="info" size={18} />Вкажіть ключі EcoFlow і SN у «Налаштуваннях».
    </div>
  {:else if snap?.status_error && !st}
    <div class="well note err"><Icon name="warning" size={18} />{snap.status_error}</div>
  {/if}

  {#if st}
    <div class="now">
      <div class="charge">
        <span class="engraved">Заряд</span>
        <span class="soc mono">{Math.round(st.soc)}<small>%</small></span>
        <div class="meter well"><span style:width="{Math.min(100, st.soc)}%"></span></div>
      </div>
      <div class="mode recess">
        <span class="engraved">Дім живиться</span>
        <span class="mode-name">
          <Icon name={st.source === "battery" ? "battery" : st.source === "grid" ? "plug" : "info"} size={22} />
          {sourceLabel[st.source]}
        </span>
        <span class="muted small">
          Резерв {st.reserve} % · ліміти {st.min_dsg}–{st.max_chg} %{st.self_powered ? "" : " · не Self-powered"}
        </span>
      </div>
    </div>

    <div class="readings">
      <div class="r"><Icon name="battery" size={18} /><span class="engraved">Батарея</span><b class="mono">{watts(st.battery_w)}</b><span class="muted small">{st.battery_w > 5 ? "заряджається" : st.battery_w < -5 ? "віддає" : "спокій"}</span></div>
      <div class="r"><Icon name="home" size={18} /><span class="engraved">Дім</span><b class="mono">{watts(st.load_w)}</b></div>
      <div class="r"><Icon name="grid" size={18} /><span class="engraved">З мережі</span><b class="mono">{watts(st.grid_w)}</b></div>
      <div class="r"><Icon name="sun" size={18} /><span class="engraved">Сонце</span><b class="mono">{watts(st.pv_w)}</b></div>
    </div>
  {/if}
</Section>

<Section title="Перемкнути вручну">
  <div class="switches">
    <button
      class="key primary"
      disabled={!snap?.has_api || !!store.busy}
      onclick={() => store.run("battery", () => api.setSource("battery"), (r) => `Від станції: резерв ${r} %`)}
    >
      <Icon name="battery" size={20} />Від станції
    </button>
    <button
      class="key big dark"
      disabled={!snap?.has_api || !!store.busy}
      onclick={() => store.run("grid", () => api.setSource("grid"), (r) => `Від мережі: резерв ${r} %`)}
    >
      <Icon name="plug" size={18} />Від мережі
    </button>
    <span class="grow"></span>
    <label class="reserve">
      <span class="muted small">Свій резерв, %</span>
      <input class="input mono" type="number" min="0" max="100" bind:value={reserveInput} />
    </label>
    <button
      class="key big"
      disabled={!snap?.has_api || !!store.busy}
      onclick={() => store.run("reserve", () => api.setReserve(Number(reserveInput)), (r) => `Резерв ${r} %`)}>Встановити</button
    >
  </div>
  <p class="muted small hint">
    Перемикання йде через резерв живлення в режимі Self-powered. STREAM реагує за 1–2 хвилини.
    <button class="link-button" disabled={!snap?.has_api} onclick={() => store.run("ai", api.setAiMode, () => "Увімкнено AI Mode")}>Повернути AI Mode</button>
  </p>
</Section>

<Section title="Автоматика за графіком">
  {#snippet aside()}
    <Switch
      checked={snap?.automation ?? false}
      label="Автоматика за графіком"
      onchange={(v) => store.run("auto", () => api.setAutomation(v))}
    />
  {/snippet}

  <div class="sched-now">
    {#if snap}
      {#if snap.outage_now}
        <span class="tag">За графіком зараз відключення</span>
        {#if snap.next_change}<span class="muted">до {snap.next_change}</span>{/if}
      {:else if snap.next_change}
        <span class="tag">Світло є</span><span class="muted">наступне відключення о {snap.next_change}</span>
      {:else}
        <span class="tag">Світло є</span><span class="muted">відключень у графіку немає</span>
      {/if}
    {/if}
  </div>

  {#if upcoming.length}
    <ul class="plan">
      {#each upcoming as p (p.key)}
        <li class:off={!snap?.automation}>
          <span class="mono t">{when(p.at)}</span>
          <Icon name={p.to === "battery" ? "battery" : "plug"} size={16} />
          <span>{p.to === "battery" ? "Від станції" : "Від мережі"}</span>
          <span class="muted small">блок {hm(p.block)} · вікно {p.window}</span>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="muted small">Немає запланованих перемикань. Графік — у розділі «Графік».</p>
  {/if}
</Section>

<style>
  .small {
    font-size: 12px;
  }
  .note {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 12px 14px;
    margin-bottom: 14px;
    font-size: 13px;
  }
  .note.err {
    color: var(--danger-text);
  }
  .now {
    display: grid;
    grid-template-columns: 1fr 1.2fr;
    gap: 18px;
    align-items: stretch;
  }
  .charge {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .soc {
    font-size: 54px;
    font-weight: 500;
    line-height: 1;
  }
  .soc small {
    font-size: 22px;
    margin-left: 4px;
    color: var(--engraved);
  }
  .meter {
    height: 12px;
    padding: 2px;
    margin-top: 4px;
  }
  .meter span {
    display: block;
    height: 100%;
    border-radius: 5px;
    background: var(--signal);
  }
  .mode {
    display: flex;
    flex-direction: column;
    gap: 8px;
    justify-content: center;
    padding: 16px 18px;
  }
  .mode-name {
    display: flex;
    align-items: center;
    gap: 10px;
    font: 600 22px Geologica, sans-serif;
  }
  .readings {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 10px;
    margin-top: 18px;
  }
  .r {
    display: grid;
    grid-template-columns: auto 1fr;
    column-gap: 8px;
    row-gap: 2px;
    align-items: center;
  }
  .r b {
    grid-column: 1 / -1;
    font-size: 18px;
    font-weight: 500;
  }
  .r .small {
    grid-column: 1 / -1;
  }
  .switches {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .grow {
    flex: 1;
  }
  .reserve {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .reserve .input {
    width: 76px;
  }
  .hint {
    margin-top: 12px;
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .sched-now {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-bottom: 12px;
    font-size: 13px;
  }
  .plan {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .plan li {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13.5px;
  }
  .plan li.off {
    opacity: 0.5;
  }
  .t {
    width: 92px;
    font-weight: 500;
  }
</style>
