<script lang="ts">
  import { clock } from '../clock.svelte';
  import { onEscape } from '../escape';
  import { dayFromToday, formatTime } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import { notes, type NoteSummary } from '../notes.svelte';

  let {
    error,
    onopen,
    onnewnote,
    onnewmeeting,
  }: { error: string; onopen: (path: string) => void; onnewnote: () => void; onnewmeeting: () => void } = $props();

  type Filter = 'all' | 'note' | 'meeting';
  let filter = $state<Filter>('all');
  let menuOpen = $state(false);
  let menu = $state<HTMLDivElement>();

  const shown = $derived(notes.list.filter((n) => filter === 'all' || n.kind === filter));

  $effect(() => {
    if (!menuOpen) return;
    const offEscape = onEscape(() => (menuOpen = false));
    const outside = (e: PointerEvent) => {
      const target = e.target as Element;
      if (!menu?.contains(target) && !target.closest('.new')) menuOpen = false;
    };
    window.addEventListener('pointerdown', outside, true);
    return () => {
      offEscape();
      window.removeEventListener('pointerdown', outside, true);
    };
  });

  const weekday = new Intl.DateTimeFormat('en-GB', { weekday: 'short' });
  const dayMonth = new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short' });

  /** "14:00" for a meeting today, "Today" for a note edited today, "Fri" this week, else "12 Oct". */
  function when(note: NoteSummary) {
    void clock.now;
    const today = dayFromToday(0);
    if (note.date >= today && note.date < dayFromToday(1)) return note.kind === 'meeting' ? formatTime(note.date) : 'Today';
    if (note.date >= dayFromToday(-6) && note.date < today) return weekday.format(note.date);
    return dayMonth.format(note.date);
  }

  function pick(action: () => void) {
    menuOpen = false;
    action();
  }
</script>

<div class="view">
  <div class="toolbar">
    <label class="input sm search">
      <Icon name="search" size={16} />
      <input
        value={notes.query}
        oninput={(e) => (notes.query = e.currentTarget.value)}
        placeholder="Search notes"
        aria-label="Search notes"
      />
    </label>
    <div class="new-wrap">
      <button class="btn new" aria-haspopup="menu" aria-expanded={menuOpen} onclick={() => (menuOpen = !menuOpen)}>
        <Icon name="add" size={16} />New<Icon name="chevron-down" size={12} />
      </button>
      {#if menuOpen}
        <div class="menu" role="menu" bind:this={menu}>
          <button class="mi" role="menuitem" onclick={() => pick(onnewnote)}><Icon name="notes" size={16} />New note</button>
          <button class="mi" role="menuitem" onclick={() => pick(onnewmeeting)}>
            <span style:color="var(--meeting)"><Icon name="meeting" size={16} /></span>New meeting note
          </button>
        </div>
      {/if}
    </div>
  </div>

  <div class="chips" role="radiogroup" aria-label="Show">
    {#each [['all', 'All'], ['note', 'Notes'], ['meeting', 'Meetings']] as [id, label] (id)}
      <button class="chip" role="radio" aria-checked={filter === id} class:on={filter === id} onclick={() => (filter = id as Filter)}>
        {label}
      </button>
    {/each}
  </div>

  {#if error}<p class="error meta">{error}</p>{/if}

  {#if shown.length === 0}
    <div class="empty">
      <span style:color="var(--ornament)"><Icon name="notes" size={24} /></span>
      {#if notes.query.trim()}
        <span class="proverb">No matches</span>
        <span class="muted">Nothing in your notes mentions “{notes.query.trim()}”.</span>
      {:else}
        <span class="proverb">No notes yet</span>
        <span class="muted">Start one with New. Notes are Markdown files in Documents\Karsa\Notes.</span>
      {/if}
    </div>
  {:else}
    <div class="scroll">
      {#each shown as note (note.path)}
        <button class="note" onclick={() => onopen(note.path)}>
          <span class="line">
            {#if note.kind === 'meeting'}<span style:color="var(--meeting)"><Icon name="meeting" size={12} /></span>{/if}
            <span class="title ell grow">{note.title}</span>
            <span class="meta">{when(note)}</span>
          </span>
          <span class="line meta" class:indent={note.kind === 'meeting'}>
            {#if note.kind === 'meeting'}
              {#if note.topic}<span class="ell">{note.topic}</span>{/if}
              {#if note.openActions > 0}
                <span class="actions" style:color="var(--focus)">{note.openActions} open</span>
              {:else if note.doneActions > 0}
                <span class="actions" style:color="var(--done)">All done</span>
              {/if}
            {:else}
              <span class="ell">{note.snippet}</span>
            {/if}
          </span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .toolbar {
    display: flex;
    gap: 8px;
  }

  .search {
    flex: 1;
  }

  .new-wrap {
    position: relative;
  }

  .new {
    height: 32px;
    border-radius: 16px;
    padding: 0 12px;
  }

  .menu {
    top: 40px;
    right: 0;
  }

  .chips {
    display: flex;
    gap: 6px;
    margin: 10px 0 4px;
  }

  .error {
    margin: 4px;
    color: var(--danger);
  }

  .note {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 4px;
    border-bottom: 1px solid var(--hairline);
    text-align: left;
    transition: background-color var(--dur-fast) var(--ease-standard);
  }

  .note:hover {
    background: var(--surface);
  }

  .line {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .line.indent {
    padding-left: 20px;
  }

  .title {
    font-weight: var(--fw-semibold);
  }

  .actions {
    flex: none;
  }
</style>
