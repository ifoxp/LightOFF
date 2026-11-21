<!-- Status LED: lit = signal colour, off = hollow ring. Always paired with a word by the caller. -->
<script lang="ts">
  let { on }: { on: boolean } = $props();
</script>

<span class="led" class:on aria-hidden="true"></span>

<style>
  .led {
    width: 8px;
    height: 8px;
    flex: none;
    border-radius: 50%;
    background: transparent;
    border: 1.5px solid var(--led-off-ring);
  }
  .led.on {
    background: var(--signal);
    border: 0;
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.18);
  }
</style>
