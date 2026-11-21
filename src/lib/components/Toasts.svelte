<!-- Transient messages (bottom right). Errors stay a bit longer; each can be dismissed. -->
<script lang="ts">
  import { store } from "../store.svelte";
  import Icon from "./Icon.svelte";
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each store.toasts as t (t.id)}
    <div class="toast">
      <Icon name={t.tone === "error" ? "warning" : "check"} size={18} />
      <span class="msg">{t.text}</span>
      <button class="key icon-only" aria-label="Закрити повідомлення" onclick={() => store.dismiss(t.id)}>
        <Icon name="close" size={14} />
      </button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 18px;
    bottom: 18px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 20;
    max-width: 440px;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 8px 8px 14px;
    border-radius: 12px;
    background: var(--key-dark);
    color: var(--on-key-dark);
    box-shadow: var(--dark-face), 0 6px 16px rgba(0, 0, 0, 0.25);
    font-size: 13px;
  }
  .msg {
    flex: 1;
    min-width: 0;
  }
  .toast .key {
    height: 26px;
    width: 26px;
  }
</style>
