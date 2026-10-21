<script lang="ts">
  import { onMount } from 'svelte';
  import { clock } from '../clock.svelte';
  import { onEscape } from '../escape';
  import { formatTime } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import { formatSize, shelf, type ShelfItem } from '../shelf.svelte';

  let copiedId = $state<number | null>(null);
  let menuOpen = $state(false);
  let confirmClear = $state(false);
  /** "2 files added", shown briefly after Paste or Add files. */
  let notice = $state('');
  let addedTimer: ReturnType<typeof setTimeout> | undefined;

  function report(count: number) {
    if (!count) return;
    notice = count === 1 ? '1 file added' : `${count} files added`;
    clearTimeout(addedTimer);
    addedTimer = setTimeout(() => (notice = ''), 2500);
  }

  const paste = async () => report(await shelf.paste());
  const add = async () => report(await shelf.pick());

  // Ctrl+V pastes files while the Files view is open, unless a text field has the focus.
  function onKeydown(e: KeyboardEvent) {
    const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;
    if (e.ctrlKey && e.key.toLowerCase() === 'v' && !typing) {
      e.preventDefault();
      void paste();
    }
  }

  onMount(() => void shelf.load());

  $effect(() => {
    if (!menuOpen) return;
    return onEscape(() => (menuOpen = false));
  });

  function added(at: number, now: Date) {
    const minutes = Math.floor((now.getTime() - at) / 60_000);
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes} min ago`;
    if (new Date(at).toDateString() === now.toDateString()) return formatTime(at);
    return new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short' }).format(at);
  }

  async function copy(item: ShelfItem) {
    await shelf.copy([item.id]);
    if (shelf.error) return;
    copiedId = item.id;
    setTimeout(() => copiedId === item.id && (copiedId = null), 1200);
  }

  // Dragging a row hands the file to Windows' own drag, so it can be dropped in any app.
  function dragStart(e: DragEvent, item: ShelfItem) {
    e.preventDefault();
    if (item.exists) void shelf.drag([item.id]);
  }

  async function clear() {
    await shelf.clear();
    menuOpen = false;
    confirmClear = false;
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="toolbar">
  <button class="btn sm" onclick={add}><Icon name="add" size={14} />Add files…</button>
  <button class="btn sm" onclick={paste} title="Paste files copied in Explorer (Ctrl+V)"><Icon name="copy" size={14} />Paste</button>
  <span class="grow"></span>
  {#if shelf.items.length}
    <span class="menu-anchor">
      <button class="ibtn" aria-label="More" onclick={() => ((menuOpen = !menuOpen), (confirmClear = false))}><Icon name="more" size={16} /></button>
      {#if menuOpen}
        <div class="menu" role="menu">
          {#if confirmClear}
            <button class="mi danger" role="menuitem" onclick={clear}><Icon name="delete" size={16} />Confirm: clear shelf</button>
          {:else}
            <button class="mi" role="menuitem" onclick={() => (confirmClear = true)}><Icon name="delete" size={16} />Clear shelf</button>
          {/if}
        </div>
      {/if}
    </span>
  {/if}
</div>

{#if shelf.error}<p class="error meta">{shelf.error}</p>{:else if notice}<p class="added meta"><Icon name="check" size={12} />{notice}</p>{/if}

{#if shelf.items.length === 0}
  <div class="empty">
    <span style:color="var(--ornament)"><Icon name="shelf" size={24} /></span>
    <span class="proverb">The shelf is empty</span>
    <span class="muted">Drag files onto the island, paste files copied in Explorer, or use Add files. Karsa keeps a link, not a copy.</span>
  </div>
{:else}
  <div class="scroll list">
    {#each shelf.items as item (item.id)}
      <div class="item" class:missing={!item.exists}>
        <button
          class="body"
          draggable={item.exists}
          ondragstart={(e) => dragStart(e, item)}
          onclick={() => item.exists && shelf.open(item.id)}
          title={item.path}
        >
          <span class="kind"><Icon name={item.folder ? 'folder' : 'file'} size={16} /></span>
          <span class="text">
            <span class="name ell">{item.name}</span>
            <span class="meta">
              {#if !item.exists}
                <span class="urgent">Moved or deleted</span>
              {:else if copiedId === item.id}
                <span class="copied"><Icon name="check" size={12} />Copied. Paste it anywhere</span>
              {:else}
                {#if item.size !== null}<span>{formatSize(item.size)}</span>{/if}
                <span>{added(item.addedAt, clock.now)}</span>
              {/if}
            </span>
          </span>
        </button>
        <span class="actions">
          {#if item.exists}
            <button class="ibtn sm" aria-label="Copy" title="Copy" onclick={() => copy(item)}><Icon name="copy" size={12} /></button>
            <button class="ibtn sm" aria-label="Show in folder" title="Show in folder" onclick={() => shelf.reveal(item.id)}><Icon name="folder" size={12} /></button>
          {/if}
          <button class="ibtn sm" aria-label="Remove from shelf" title="Remove from shelf" onclick={() => shelf.remove(item.id)}><Icon name="close" size={12} /></button>
        </span>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 28px;
    margin-bottom: 8px;
  }

  .added {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0 4px 8px;
    color: var(--done);
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

  .error {
    margin: 0 4px 8px;
    color: var(--danger);
  }

  .item {
    position: relative;
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--hairline);
  }

  .body {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 4px;
    text-align: left;
    border-radius: 8px;
  }

  .body:hover {
    background: var(--surface);
  }

  .kind {
    flex: none;
    color: var(--muted);
  }

  .missing .kind,
  .missing .name {
    opacity: var(--o-disabled);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .copied {
    color: var(--done);
  }

  .actions {
    position: absolute;
    right: 4px;
    display: flex;
    gap: 2px;
    padding-left: 12px;
    background: linear-gradient(90deg, transparent, var(--island-bg) 12px);
    opacity: 0;
  }

  .item:hover .actions,
  .actions:focus-within {
    opacity: 1;
  }
</style>
