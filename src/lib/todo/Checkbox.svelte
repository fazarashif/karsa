<script lang="ts">
  import Icon from '../Icon.svelte';

  let {
    checked,
    label,
    size = 16,
    onchange,
  }: { checked: boolean; label: string; size?: 14 | 16; onchange: () => void } = $props();
</script>

<button
  class="cb"
  class:on={checked}
  role="checkbox"
  aria-checked={checked}
  aria-label={label}
  style:width="{size}px"
  style:height="{size}px"
  onclick={onchange}
>
  {#if checked}<Icon name="check" size={size - 6} />{/if}
</button>

<style>
  .cb {
    flex: none;
    box-sizing: border-box;
    display: grid;
    place-items: center;
    border: 1.5px solid var(--checkbox-border);
    border-radius: var(--radius-checkbox);
    color: var(--on-accent);
    transition:
      border-color var(--dur-fast) var(--ease-standard),
      background-color var(--dur-fast) var(--ease-standard),
      transform var(--dur-fast) var(--ease-standard);
  }

  .cb:hover {
    border-color: var(--fg);
  }

  .cb:active {
    background: var(--surface);
    transform: scale(0.9);
  }

  .cb.on {
    background: var(--done);
    border-color: var(--done);
  }

  .cb.on:hover {
    background: #b5c07a;
    border-color: #b5c07a;
  }
</style>
