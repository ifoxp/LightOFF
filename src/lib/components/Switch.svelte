<!-- Toggle switch (role=switch). Hit area 44×36, physical track with a key-like knob. -->
<script lang="ts">
  let {
    checked,
    label,
    disabled = false,
    onchange,
  }: { checked: boolean; label: string; disabled?: boolean; onchange: (v: boolean) => void } = $props();
</script>

<button
  type="button"
  role="switch"
  class="switch"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={() => onchange(!checked)}
>
  <span class="track" class:on={checked}><span class="knob"></span></span>
</button>

<style>
  .switch {
    width: 52px;
    height: 36px;
    flex: none;
    border: 0;
    padding: 0;
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-radius: 8px;
  }
  .switch:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
  .track {
    width: 44px;
    height: 24px;
    padding: 3px;
    border-radius: 7px;
    display: flex;
    justify-content: flex-start;
    background: var(--switch-off);
    box-shadow: var(--well-shadow);
    transition: background 120ms ease;
  }
  .track.on {
    justify-content: flex-end;
    background: var(--switch-on);
    box-shadow: inset 0 2px 3px rgba(0, 0, 0, 0.45);
  }
  .knob {
    width: 18px;
    height: 18px;
    border-radius: 5px;
    background: var(--key);
    box-shadow: var(--key-face), 0 1px 1px rgba(0, 0, 0, 0.3);
  }
</style>
