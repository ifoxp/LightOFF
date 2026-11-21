<!-- Row of keys acting as a radio group; the chosen key stays pressed. -->
<script lang="ts" generics="T extends string">
  import Icon from "./Icon.svelte";

  let {
    value,
    options,
    label,
    disabled = false,
    size = "normal",
    onchange,
  }: {
    value: T;
    options: { value: T; label: string; hint?: string; icon?: string }[];
    label: string;
    disabled?: boolean;
    size?: "small" | "normal";
    onchange: (v: T) => void;
  } = $props();

  let buttons: HTMLButtonElement[] = $state([]);

  function onkeydown(e: KeyboardEvent, i: number) {
    let next = -1;
    if (e.key === "ArrowRight" || e.key === "ArrowDown") next = (i + 1) % options.length;
    if (e.key === "ArrowLeft" || e.key === "ArrowUp") next = (i - 1 + options.length) % options.length;
    if (next >= 0) {
      e.preventDefault();
      onchange(options[next].value);
      buttons[next]?.focus();
    }
  }
</script>

<div class="seg well" class:small={size === "small"} role="radiogroup" aria-label={label}>
  {#each options as o, i (o.value)}
    <button
      bind:this={buttons[i]}
      type="button"
      role="radio"
      class="key"
      aria-checked={o.value === value}
      tabindex={o.value === value ? 0 : -1}
      title={o.hint}
      {disabled}
      onclick={() => o.value !== value && onchange(o.value)}
      onkeydown={(e) => onkeydown(e, i)}
    >
      {#if o.icon}<Icon name={o.icon} size={16} />{/if}{o.label}
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    gap: 5px;
    padding: 4px;
  }
  .seg .key {
    height: 30px;
    padding: 0 12px;
    font-size: 12.5px;
  }
  .seg.small .key {
    height: 28px;
    padding: 0 9px;
    font-size: 12px;
  }
  .seg .key[aria-checked="true"] {
    background: var(--chosen-bg);
    color: var(--chosen-fg);
    box-shadow: var(--chosen-shadow);
  }
</style>
