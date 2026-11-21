<!-- «Налаштування»: EcoFlow keys, VOE address, automation timing, behaviour. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, errText } from "../api";
  import Icon from "../components/Icon.svelte";
  import Section from "../components/Section.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Switch from "../components/Switch.svelte";
  import { store } from "../store.svelte";
  import { loadTheme, saveTheme, type ThemeChoice } from "../theme";
  import type { Settings } from "../types";

  let s = $state<Settings | null>(null);
  let saved = $state("");
  let autostart = $state(false);
  let showSecret = $state(false);
  let testMsg = $state<{ ok: boolean; text: string } | null>(null);
  let theme = $state<ThemeChoice>(loadTheme());

  // Reserve modes: automatic value or a fixed number.
  let batAuto = $state(true);
  let batValue = $state(13);
  let gridAuto = $state(true);
  let gridValue = $state(95);

  function load(v: Settings) {
    s = v;
    batAuto = v.battery_reserve == null;
    batValue = v.battery_reserve ?? 13;
    gridAuto = v.grid_reserve == null;
    gridValue = v.grid_reserve ?? 95;
    saved = JSON.stringify(collect());
  }

  function collect(): Settings | null {
    if (!s) return null;
    return {
      ...s,
      lead_minutes: Number(s.lead_minutes),
      back_minutes: Number(s.back_minutes),
      block_minutes: Number(s.block_minutes),
      schedule_refresh_minutes: Number(s.schedule_refresh_minutes),
      battery_reserve: batAuto ? null : Number(batValue),
      grid_reserve: gridAuto ? null : Number(gridValue),
    };
  }

  const dirty = $derived(s ? JSON.stringify(collect()) !== saved : false);

  onMount(async () => {
    try {
      load(await api.getSettings());
      autostart = await api.autostartStatus();
    } catch (e) {
      store.toast(errText(e), "error");
    }
  });

  async function save() {
    const v = collect();
    if (!v) return;
    try {
      await api.saveSettings(v);
      saved = JSON.stringify(v);
      store.toast("Налаштування збережено");
      await store.refresh();
    } catch (e) {
      store.toast(errText(e), "error");
    }
  }

  async function test() {
    const v = collect();
    if (!v) return;
    testMsg = null;
    try {
      testMsg = { ok: true, text: await api.testConnection(v) };
    } catch (e) {
      testMsg = { ok: false, text: errText(e) };
    }
  }

  async function importLegacy() {
    const v = await store.run("import", api.importLegacy);
    if (v && s) {
      s = { ...s, access_key: v.access_key, secret_key: v.secret_key, sn: v.sn, city: v.city, street: v.street, house: v.house };
      store.toast("Дані імпортовано. Перевірте й натисніть «Зберегти».");
    }
  }

  async function toggleAutostart(v: boolean) {
    await store.run("autostart", () => api.setAutostart(v));
    autostart = await api.autostartStatus();
  }
</script>

{#if s}
  <Section title="EcoFlow Developer API">
    {#snippet aside()}
      <button class="key" onclick={importLegacy}><Icon name="download" size={16} />Імпорт зі старого LightOFF</button>
    {/snippet}
    <div class="form">
      <label><span>Access Key</span><input class="input mono" bind:value={s.access_key} spellcheck="false" autocomplete="off" /></label>
      <label>
        <span>Secret Key</span>
        <span class="row">
          <input class="input mono" type={showSecret ? "text" : "password"} bind:value={s.secret_key} spellcheck="false" autocomplete="off" />
          <button class="key" onclick={() => (showSecret = !showSecret)}>{showSecret ? "Сховати" : "Показати"}</button>
        </span>
      </label>
      <label><span>SN пристрою</span><input class="input mono" bind:value={s.sn} spellcheck="false" autocomplete="off" /></label>
      <label>
        <span>Сервер API</span>
        <select class="input" bind:value={s.api_host}>
          <option value="https://api-e.ecoflow.com">Європа (api-e.ecoflow.com)</option>
          <option value="https://api.ecoflow.com">Загальний (api.ecoflow.com)</option>
          <option value="https://api-a.ecoflow.com">Америка (api-a.ecoflow.com)</option>
        </select>
      </label>
    </div>
    <div class="row gap">
      <button class="key big" onclick={test}><Icon name="refresh" size={16} />Перевірити з'єднання</button>
      {#if testMsg}<span class:err={!testMsg.ok} class="small">{testMsg.text}</span>{/if}
    </div>
  </Section>

  <Section title="Адреса на сайті ВОЕ">
    <div class="form three">
      <label><span>Місто / село</span><input class="input" bind:value={s.city} placeholder="Вінниця" /></label>
      <label><span>Вулиця</span><input class="input" bind:value={s.street} placeholder="Соборна" /></label>
      <label><span>Будинок</span><input class="input" bind:value={s.house} placeholder="1" /></label>
    </div>
    <div class="line">
      <span>
        Оновлювати графік у фоні
        <small class="muted">кожні N хвилин, 0 — вимкнено. Якщо сайт попросить перевірку «я не робот», прийде сповіщення.</small>
      </span>
      <input class="input mono num" type="number" min="0" max="1440" bind:value={s.schedule_refresh_minutes} />
    </div>
  </Section>

  <Section title="Автоматика">
    <div class="line">
      <span>«Від станції» за стільки хвилин до відключення</span>
      <input class="input mono num" type="number" min="1" max="60" bind:value={s.lead_minutes} />
    </div>
    <div class="line">
      <span>«Від мережі» через стільки хвилин після початку блоку</span>
      <input class="input mono num" type="number" min="1" max="120" bind:value={s.back_minutes} />
    </div>
    <div class="line">
      <span>
        Довжина блоку, хвилин
        <small class="muted">довгі відключення діляться на блоки, і перемикання повторюється на початку кожного (4 год → двічі, 6 год → тричі)</small>
      </span>
      <input class="input mono num" type="number" min="30" max="720" step="30" bind:value={s.block_minutes} />
    </div>
    <div class="line">
      <span>Враховувати «можливі» відключення</span>
      <Switch checked={s.include_maybe} label="Враховувати можливі відключення" onchange={(v) => s && (s.include_maybe = v)} />
    </div>
    <div class="line">
      <span>
        Резерв «від станції»
        <small class="muted">«Авто» = ліміт розряду + 3 % (зараз це 13 %)</small>
      </span>
      <span class="row">
        <Segmented label="Резерв від станції" size="small" value={batAuto ? "auto" : "fixed"} options={[{ value: "auto", label: "Авто" }, { value: "fixed", label: "Своє" }]} onchange={(v) => (batAuto = v === "auto")} />
        <input class="input mono num" type="number" min="0" max="100" bind:value={batValue} disabled={batAuto} />
      </span>
    </div>
    <div class="line">
      <span>
        Резерв «від мережі»
        <small class="muted">«Авто» = поточний заряд (батарея просто тримає заряд). Більше число — STREAM дозаряджатиметься з мережі</small>
      </span>
      <span class="row">
        <Segmented label="Резерв від мережі" size="small" value={gridAuto ? "auto" : "fixed"} options={[{ value: "auto", label: "Авто" }, { value: "fixed", label: "Своє" }]} onchange={(v) => (gridAuto = v === "auto")} />
        <input class="input mono num" type="number" min="0" max="100" bind:value={gridValue} disabled={gridAuto} />
      </span>
    </div>
  </Section>

  <Section title="Програма">
    <div class="line">
      <span>Сповіщення Windows перед відключенням</span>
      <span class="row">
        <button class="key" onclick={() => store.run("notif", api.testNotification)}><Icon name="bell" size={16} />Перевірити</button>
        <Switch checked={s.notifications} label="Сповіщення" onchange={(v) => s && (s.notifications = v)} />
      </span>
    </div>
    <div class="line">
      <span>Запускати з Windows (згорнутою в трей)</span>
      <Switch checked={autostart} label="Автозапуск" onchange={toggleAutostart} />
    </div>
    <div class="line">
      <span>Тема</span>
      <Segmented
        label="Тема"
        size="small"
        value={theme}
        options={[
          { value: "system", label: "Як у системі" },
          { value: "light", label: "Світла" },
          { value: "dark", label: "Темна" },
        ]}
        onchange={(v) => {
          theme = v;
          saveTheme(v);
        }}
      />
    </div>
    <div class="line">
      <span>Дані, налаштування й логи</span>
      <span class="row">
        <button class="key" onclick={() => store.run("dir", api.openDataDir)}><Icon name="folder" size={16} />Відкрити папку</button>
        <button class="key danger" onclick={() => api.quit()}><Icon name="power" size={16} />Вийти з програми</button>
      </span>
    </div>
  </Section>

  <div class="savebar" class:show={dirty}>
    <span>Є незбережені зміни</span>
    <button class="key" onclick={async () => load(await api.getSettings())}>Скасувати</button>
    <button class="key primary" onclick={save}><Icon name="check" size={18} />Зберегти</button>
  </div>
{/if}

<style>
  .small {
    font-size: 12.5px;
  }
  .err {
    color: var(--danger-text);
  }
  .form {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px 16px;
    margin-bottom: 14px;
  }
  .form.three {
    grid-template-columns: 1.2fr 1.5fr 0.6fr;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12.5px;
    color: var(--text-2);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .row .input {
    flex: 1;
  }
  .gap {
    gap: 12px;
  }
  .line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
    padding: 8px 0;
    font-size: 13.5px;
  }
  .line > span:first-child {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-width: 62%;
  }
  .line small {
    font-size: 12px;
  }
  .num {
    width: 84px;
    flex: none !important;
  }
  .savebar {
    position: sticky;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
    padding: 12px 0 2px;
    background: var(--body);
    visibility: hidden;
  }
  .savebar.show {
    visibility: visible;
  }
  .savebar span {
    flex: 1;
    color: var(--text-2);
    font-size: 13px;
  }
</style>
