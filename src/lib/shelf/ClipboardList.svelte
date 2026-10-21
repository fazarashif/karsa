<script lang="ts">
  import { onMount } from 'svelte';
  import { clipboard, type ClipItem } from '../clipboard.svelte';
  import { clock } from '../clock.svelte';
  import { onEscape } from '../escape';
  import { formatTime } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import { settings } from '../settings.svelte';

  let copiedId = $state<number | null>(null);
  let menuOpen = $state(false);
  let confirmClear = $state(false);

  const s = $derived(settings.current);

  onMount(() => clipboard.watch());

  $effect(() => {
    if (!menuOpen) return;
    return onEscape(() => (menuOpen = false));
  });

  function when(at: number, now: Date) {
    const minutes = Math.floor((now.getTime() - at) / 60_000);
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes} min ago`;
    const sameDay = new Date(at).toDateString() === now.toDateString();
    return sameDay ? formatTime(at) : `Yesterday ${formatTime(at)}`;
  }

  async function copy(item: ClipItem) {
    await clipboard.copy(item.id);
    if (clipboard.error) return;
    copiedId = item.id;
    setTimeout(() => copiedId === item.id && (copiedId = null), 1200);
  }

  async function clear() {
    await clipboard.clear();
    menuOpen = false;
    confirmClear = false;
  }
</script>

{#if !s.clipboardEnabled}
  <div class="empty">
    <span style:color="var(--ornament)"><Icon name="copy" size={24} /></span>
    <span class="proverb">Clipboard history is off</span>
    <span class="muted">Turn it on to keep what you copy for 24 hours.</span>
    <button class="btn sm" onclick={() => settings.update({ clipboardEnabled: true })}>Turn on</button>
  </div>
{:else}
  <div class="toolbar">
    <label class="input sm grow">
      <Icon name="search" size={14} />
      <input value={clipboard.query} oninput={(e) => (clipboard.query = e.currentTarget.value)} placeholder="Search copied text" aria-label="Search copied text" />
    </label>
    <button
      class="ibtn"
      aria-label={s.clipboardPaused ? 'Resume recording' : 'Pause recording'}
      title={s.clipboardPaused ? 'Resume recording' : 'Pause recording'}
      onclick={() => settings.update({ clipboardPaused: !s.clipboardPaused })}
    >
      <Icon name={s.clipboardPaused ? 'play' : 'pause'} size={14} />
    </button>
    <span class="menu-anchor">
      <button class="ibtn" aria-label="More" onclick={() => ((menuOpen = !menuOpen), (confirmClear = false))}><Icon name="more" size={16} /></button>
      {#if menuOpen}
        <div class="menu" role="menu">
          {#if confirmClear}
            <button class="mi danger" role="menuitem" onclick={clear}><Icon name="delete" size={16} />Confirm: clear all</button>
          {:else}
            <button class="mi" role="menuitem" onclick={() => (confirmClear = true)}><Icon name="delete" size={16} />Clear all</button>
          {/if}
        </div>
      {/if}
    </span>
  </div>

  {#if s.clipboardPaused}
    <p class="paused meta"><Icon name="pause" size={12} />Paused. New copies are not kept.</p>
  {/if}
  {#if clipboard.error}<p class="error meta">{clipboard.error}</p>{/if}

  {#if clipboard.items.length === 0}
    <div class="empty">
      <span style:color="var(--ornament)"><Icon name="copy" size={24} /></span>
      {#if clipboard.query.trim()}
        <span class="proverb">No matches</span>
      {:else}
        <span class="proverb">Nothing copied yet</span>
        <span class="muted">Text you copy shows here for 24 hours.</span>
        <span class="faint small">Copies that password managers mark as private are never kept.</span>
      {/if}
    </div>
  {:else}
    <div class="scroll list">
      {#each clipboard.items as item (item.id)}
        <div class="item">
          <button class="body" onclick={() => copy(item)} title="Copy again">
            <span class="text">{item.text}</span>
            <span class="meta">
              {#if copiedId === item.id}
                <span class="copied"><Icon name="check" size={12} />Copied</span>
              {:else}
                <span>{when(item.copiedAt, clock.now)}</span>
              {/if}
            </span>
          </button>
          <button class="ibtn sm remove" aria-label="Remove" onclick={() => clipboard.remove(item.id)}><Icon name="close" size={12} /></button>
        </div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 8px;
  }

  .menu-anchor {
    position: relative;
  }

  .menu {
    right: 0;
    top: 32px;
  }

  .mi.danger {
    color: var(--danger);
  }

  .paused,
  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0 4px 8px;
  }

  .error {
    color: var(--danger);
  }

  .item {
    position: relative;
    display: flex;
    align-items: flex-start;
    border-bottom: 1px solid var(--hairline);
  }

  .body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 28px 8px 4px;
    text-align: left;
    border-radius: 8px;
  }

  .body:hover {
    background: var(--surface);
  }

  .text {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    overflow-wrap: anywhere;
    white-space: pre-line;
  }

  .copied {
    color: var(--done);
  }

  .remove {
    position: absolute;
    right: 4px;
    top: 8px;
    opacity: 0;
  }

  .item:hover .remove,
  .remove:focus-visible {
    opacity: 1;
  }

  .small {
    font-size: var(--fs-caption);
  }
</style>
