<!-- «Графік»: today and tomorrow in half-hours. Refresh from the VOE site or paint cells by hand. -->
<script lang="ts">
  import { api } from "../api";
  import Icon from "../components/Icon.svelte";
  import Section from "../components/Section.svelte";
  import Segmented from "../components/Segmented.svelte";
  import { dayLabel, hm, isToday } from "../format";
  import { store } from "../store.svelte";
  import type { Cell } from "../types";

  const snap = $derived(store.snap);
  let brush = $state<Cell>("off");
  let drag = $state<{ date: string; from: number; to: number } | null>(null);

  const hours = Array.from({ length: 24 }, (_, h) => String(h).padStart(2, "0"));

  function nowIndex(date: string): number {
    if (!isToday(date + "T00:00")) return -1;
    const d = new Date();
    return d.getHours() * 2 + (d.getMinutes() >= 30 ? 1 : 0);
  }

  function inDrag(date: string, i: number) {
    return drag && drag.date === date && i >= Math.min(drag.from, drag.to) && i <= Math.max(drag.from, drag.to);
  }

  function down(date: string, i: number, e: PointerEvent) {
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
    drag = { date, from: i, to: i };
  }

  function enter(date: string, i: number) {
    if (drag && drag.date === date) drag = { ...drag, to: i };
  }

  async function up() {
    if (!drag) return;
    const d = drag;
    await store.run("cells", () => api.setCells(d.date, d.from, d.to, brush));
    drag = null;
  }

  function span(i: number) {
    const m = (n: number) => `${String(Math.floor(n / 2)).padStart(2, "0")}:${n % 2 ? "30" : "00"}`;
    return `${m(i)}–${m(i + 1)}`;
  }

  const stageText: Record<string, string> = {
    loading: "відкриваю сайт…",
    challenge: "сайт просить перевірку «я не робот» — пройдіть її у вікні",
    searching: "шукаю адресу…",
    manual: "знайдіть адресу у вікні сайту й натисніть «Показати»",
    no_form: "на сторінці немає форми пошуку",
  };
</script>

<svelte:window onpointerup={up} />

<Section title="Графік відключень">
  {#snippet aside()}
    {#if snap?.queue}<span class="tag">черга {snap.queue}</span>{/if}
    <button class="key big" onclick={() => store.run("voe", api.openVoe)} disabled={!!snap?.voe_open && !!store.busy}>
      <Icon name="external" size={16} />Оновити з сайту ВОЕ
    </button>
  {/snippet}

  <p class="muted small meta">
    {#if snap?.schedule_updated}
      {snap.schedule_source === "voe" ? "З сайту ВОЕ" : "Змінено вручну"} · {snap.schedule_updated}
    {:else}
      Графіка ще немає: оновіть з сайту ВОЕ або позначте години вручну.
    {/if}
    {#if snap?.voe_open}
      <span class="tag live">
        ВОЕ{snap.voe_visible ? "" : " (у фоні)"}:
        {snap.voe_stage === "challenge" && !snap.voe_visible
          ? "сайт просить перевірку — натисніть «Оновити з сайту ВОЕ»"
          : (stageText[snap.voe_stage ?? "loading"] ?? snap.voe_stage)}
      </span>
    {/if}
  </p>

  {#each snap?.days ?? [] as day (day.date)}
    {@const now = nowIndex(day.date)}
    <div class="day">
      <div class="dayhead">
        <span class="dname">{dayLabel(day.date)}</span>
        <button class="link-button" onclick={() => store.run("clear", () => api.clearDay(day.date))}>Очистити</button>
      </div>
      <div class="hours mono" aria-hidden="true">
        {#each hours as h (h)}<span>{h}</span>{/each}
      </div>
      <div class="cells well" role="grid" aria-label={dayLabel(day.date)}>
        {#each day.cells as c, i (i)}
          <button
            class="cell {c}"
            class:now={i === now}
            class:half={i % 2 === 1}
            class:drag={inDrag(day.date, i)}
            title="{span(i)}: {c === 'off' ? 'відключення' : c === 'maybe' ? 'можливе відключення' : 'світло є'}"
            aria-label={span(i)}
            onpointerdown={(e) => down(day.date, i, e)}
            onpointerenter={() => enter(day.date, i)}
          ></button>
        {/each}
      </div>
    </div>
  {/each}

  <div class="tools">
    <span class="muted small">Пензель:</span>
    <Segmented
      label="Що позначати"
      size="small"
      value={brush}
      options={[
        { value: "off", label: "Відключення" },
        { value: "maybe", label: "Можливе" },
        { value: "on", label: "Світло є" },
      ]}
      onchange={(v) => (brush = v)}
    />
    <span class="muted small">Клацніть або протягніть по клітинках, щоб змінити графік.</span>
  </div>
  <div class="legend small">
    <span><i class="sw on"></i>світло є</span>
    <span><i class="sw off"></i>відключення</span>
    <span><i class="sw maybe"></i>можливе відключення</span>
  </div>
</Section>

<Section title="Заплановані перемикання">
  {#if (snap?.plan ?? []).length}
    <ul class="plan">
      {#each snap?.plan ?? [] as p (p.key)}
        <li class:past={p.past}>
          <span class="mono t">{isToday(p.at) ? hm(p.at) : `${p.at.slice(8, 10)}.${p.at.slice(5, 7)} ${hm(p.at)}`}</span>
          <Icon name={p.to === "battery" ? "battery" : "plug"} size={16} />
          <span class="what">{p.to === "battery" ? "Від станції" : "Від мережі"}</span>
          <span class="muted small">блок {hm(p.block)} · вікно {p.window}</span>
          {#if p.done}<span class="tag">виконано</span>{:else if p.past}<span class="tag">пропущено</span>{/if}
        </li>
      {/each}
    </ul>
  {:else}
    <p class="muted small">Перемикань немає: у графіку немає відключень на найближчу добу.</p>
  {/if}
  {#if !snap?.automation}
    <p class="small warn">Автоматику вимкнено — перемикання не виконуватимуться.</p>
  {/if}
</Section>

<style>
  .small {
    font-size: 12px;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    margin-bottom: 14px;
  }
  .live {
    border-color: var(--signal);
  }
  .day {
    margin-bottom: 16px;
  }
  .dayhead {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }
  .dname {
    font-weight: 600;
  }
  .hours {
    display: grid;
    grid-template-columns: repeat(24, 1fr);
    font-size: 10px;
    color: var(--engraved);
    padding: 0 5px 3px;
  }
  .cells {
    display: grid;
    grid-template-columns: repeat(48, 1fr);
    gap: 0;
    padding: 5px;
    touch-action: none;
  }
  .cell {
    height: 34px;
    border: 0;
    padding: 0;
    margin: 0;
    cursor: crosshair;
    background: var(--key);
    border-left: 1px solid var(--groove);
  }
  .cell.half {
    border-left-color: transparent;
    margin-right: 2px;
  }
  .cell:first-child {
    border-radius: 5px 0 0 5px;
  }
  .cell:last-child {
    border-radius: 0 5px 5px 0;
  }
  .cell.off {
    background: var(--key-dark);
  }
  .cell.maybe {
    background: repeating-linear-gradient(135deg, var(--signal) 0 4px, var(--key) 4px 8px);
  }
  .cell.now {
    box-shadow: inset 0 -4px 0 var(--signal);
  }
  .cell.drag {
    outline: 2px solid var(--signal);
    outline-offset: -2px;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    margin-top: 6px;
  }
  .legend {
    display: flex;
    gap: 18px;
    margin-top: 10px;
    color: var(--text-2);
  }
  .legend span {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .sw {
    width: 14px;
    height: 14px;
    border-radius: 3px;
    display: inline-block;
    box-shadow: var(--well-shadow);
  }
  .sw.on {
    background: var(--key);
  }
  .sw.off {
    background: var(--key-dark);
  }
  .sw.maybe {
    background: repeating-linear-gradient(135deg, var(--signal) 0 3px, var(--key) 3px 6px);
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
  .plan li.past {
    opacity: 0.55;
  }
  .t {
    width: 92px;
    font-weight: 500;
  }
  .what {
    width: 104px;
  }
  .warn {
    margin-top: 10px;
    color: var(--danger-text);
  }
</style>
