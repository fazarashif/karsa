<!-- The Shelf: things to hand over to another app later. Copied text from the last 24 hours,
     and files dropped on the island. -->
<script lang="ts">
  import Icon from '../Icon.svelte';
  import { panel } from '../ui.svelte';
  import ClipboardList from './ClipboardList.svelte';
  import FilesList from './FilesList.svelte';
</script>

<div class="view">
  <div class="seg" role="tablist" aria-label="Shelf">
    <button role="tab" aria-selected={panel.shelf === 'clipboard'} class:on={panel.shelf === 'clipboard'} onclick={() => (panel.shelf = 'clipboard')}>
      <Icon name="copy" size={14} />Clipboard
    </button>
    <button role="tab" aria-selected={panel.shelf === 'files'} class:on={panel.shelf === 'files'} onclick={() => (panel.shelf = 'files')}>
      <Icon name="file" size={14} />Files
    </button>
  </div>
  {#if panel.shelf === 'clipboard'}
    <ClipboardList />
  {:else}
    <FilesList />
  {/if}
</div>

<style>
  .seg {
    display: flex;
    padding: 2px;
    margin-bottom: 10px;
    border-radius: var(--radius-control);
    background: var(--surface);
  }

  .seg button {
    flex: 1;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-radius: 8px;
    font-weight: var(--fw-semibold);
    color: var(--muted);
    transition: background-color var(--dur-fast) var(--ease-standard), color var(--dur-fast) var(--ease-standard);
  }

  .seg button:hover {
    color: var(--fg);
  }

  .seg button.on {
    background: var(--surface-hover);
    color: var(--fg);
  }
</style>
