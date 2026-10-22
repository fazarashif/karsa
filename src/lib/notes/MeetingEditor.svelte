<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { formatDate, formatTime } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import { fromLocalDateTime, parseMeeting, serializeMeeting, type ActionItem, type Meeting } from '../meeting';
  import type { NoteSession } from '../notes.svelte';
  import Checkbox from '../todo/Checkbox.svelte';
  import { INBOX_ID, todo } from '../todo.svelte';
  import ConflictBanner from './ConflictBanner.svelte';
  import NoteBar from './NoteBar.svelte';

  let {
    session,
    initial,
    layout,
    onback,
    ontitle,
  }: {
    session: NoteSession;
    initial: string;
    layout: 'panel' | 'window';
    onback?: () => void;
    ontitle?: (title: string) => void;
  } = $props();

  let m = $state<Meeting>(untrack(() => parseMeeting(initial)));
  let intoList = $state(INBOX_ID);
  let addingAttendee = $state(false);
  let attendee = $state('');
  let newAction = $state('');
  let adding = $state(false);
  let titleInput = $state<HTMLInputElement>();

  const open = $derived(m.actions.filter((a) => !a.done).length);
  const toAdd = $derived(m.actions.filter((a) => !a.done && a.taskId === null && a.text.trim()));
  const when = $derived.by(() => {
    const at = m.date ? fromLocalDateTime(m.date) : NaN;
    return Number.isNaN(at) ? 'Pick a time' : `${formatDate(at)} · ${formatTime(at)}`;
  });

  $effect(() => ontitle?.(m.title || 'Meeting'));

  onMount(() => {
    // A fresh meeting note from a reminder: put the cursor in the notes, else the title.
    if (m.title === 'Meeting') titleInput?.select();
  });

  function changed() {
    session.edit(serializeMeeting($state.snapshot(m)));
  }

  /** Esc inside a small inline field cancels that field only, not the whole view. */
  function stopEscape(e: KeyboardEvent, cancel: () => void) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      cancel();
    }
  }

  function addAttendee() {
    const names = attendee.split(',').map((s) => s.trim()).filter(Boolean);
    if (names.length) {
      m.attendees.push(...names);
      changed();
    }
    attendee = '';
    addingAttendee = false;
  }

  function addAction(e: SubmitEvent) {
    e.preventDefault();
    if (!newAction.trim()) return;
    m.actions.push({ text: newAction.trim(), done: false, taskId: null });
    newAction = '';
    changed();
  }

  /** Copies an action item into Todo, linked back to this note, and marks it Added. */
  async function addToTodo(item: ActionItem) {
    await session.flush();
    const before = new Set(todo.tasks.map((t) => t.id));
    await todo.createTask(intoList, item.text, null, session.path);
    const created = todo.tasks.find((t) => !before.has(t.id) && t.title === item.text.trim());
    if (created) {
      item.taskId = created.id;
      changed();
    }
  }

  async function addAll() {
    adding = true;
    for (const item of toAdd) await addToTodo(item);
    adding = false;
  }
</script>

<div class="view editor" class:window={layout === 'window'}>
  <NoteBar {session} {layout} meeting title={m.title || 'Meeting'} {onback} />
  {#if session.status === 'conflict'}
    <ConflictBanner
      onreload={async () => {
        m = parseMeeting(await session.reload());
      }}
      onkeep={() => session.keepMine()}
    />
  {/if}

  <div class="scroll body">
    <input
      class="title"
      bind:this={titleInput}
      bind:value={m.title}
      oninput={changed}
      placeholder="Meeting title"
      aria-label="Meeting title"
      maxlength="200"
    />

    <div class="fields">
      <span class="key">When</span>
      <span class="when">
        <span>{when}</span>
        <input
          class="native"
          type="datetime-local"
          value={m.date}
          aria-label="When"
          onclick={(e) => e.currentTarget.showPicker?.()}
          onchange={(e) => {
            if (e.currentTarget.value) {
              m.date = e.currentTarget.value;
              changed();
            }
          }}
        />
      </span>

      <span class="key">Topic</span>
      <input class="plain" bind:value={m.topic} oninput={changed} placeholder="Add a topic" aria-label="Topic" maxlength="200" />

      <span class="key">Attendees</span>
      <span class="people">
        {#each m.attendees as name, i (i)}
          <span class="person">
            {name}
            <button class="remove" aria-label="Remove {name}" onclick={() => { m.attendees.splice(i, 1); changed(); }}>×</button>
          </span>
        {/each}
        {#if addingAttendee}
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="person person-input"
            bind:value={attendee}
            placeholder="Name"
            aria-label="New attendee"
            autofocus
            onblur={addAttendee}
            onkeydown={(e) => {
              if (e.key === 'Enter') addAttendee();
              stopEscape(e, () => ((attendee = ''), (addingAttendee = false)));
            }}
          />
        {:else}
          <button class="person add" aria-label="Add attendee" onclick={() => (addingAttendee = true)}>+</button>
        {/if}
      </span>
    </div>

    <span class="caps section">Notes</span>
    <textarea
      class="notes"
      bind:value={m.notes}
      oninput={changed}
      placeholder="- What was decided"
      aria-label="Notes"
      rows="2"
    ></textarea>

    <div class="ai-head">
      <span class="caps">Action items{m.actions.length ? ` · ${open} open` : ''}</span>
      <span class="grow"></span>
      {#if toAdd.length}
        <label class="into meta">
          Into
          <select bind:value={intoList} aria-label="Todo list for action items">
            {#each todo.lists as list (list.id)}<option value={list.id}>{list.name}</option>{/each}
          </select>
        </label>
        <button class="btn text tiny" disabled={adding} onclick={addAll}><Icon name="add-to-todo" size={14} />Add all</button>
      {/if}
    </div>

    {#each m.actions as item, i (i)}
      <div class="ai" class:done={item.done}>
        <Checkbox checked={item.done} label="Done: {item.text}" onchange={() => { item.done = !item.done; changed(); }} />
        <input class="plain ai-text" bind:value={item.text} oninput={changed} aria-label="Action item" />
        {#if !item.done}
          {#if item.taskId !== null}
            <span class="added"><Icon name="check-circle" size={12} />Added</span>
          {:else if item.text.trim()}
            <button class="mini" onclick={() => addToTodo(item)}><Icon name="add-to-todo" size={12} />Add to todo</button>
          {/if}
        {/if}
        <button class="ibtn sm x" aria-label="Remove action item" onclick={() => { m.actions.splice(i, 1); changed(); }}>
          <Icon name="close" size={12} />
        </button>
      </div>
    {/each}
    <form class="input sm new-ai" onsubmit={addAction}>
      <Icon name="add" size={16} />
      <input
        bind:value={newAction}
        placeholder="Add an action item…"
        aria-label="New action item"
        maxlength="500"
        onkeydown={(e) => stopEscape(e, () => (newAction = ''))}
      />
    </form>
  </div>

  <div class="status meta">
    <span>{{ saved: 'Saved', saving: 'Saving…', unsaved: 'Edited', conflict: 'Not saved', error: session.error }[session.status]}</span>
    <span class="grow"></span>
    <span>Markdown</span>
  </div>
</div>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  input.plain,
  .title,
  .notes {
    box-sizing: border-box;
    width: 100%;
    border: 0;
    outline: none;
    background: transparent;
    color: var(--fg);
    caret-color: var(--accent);
    user-select: text;
    border-radius: 6px;
  }

  input.plain:hover,
  .title:hover,
  .notes:hover {
    background: var(--surface);
  }

  input.plain:focus,
  .title:focus,
  .notes:focus {
    background: var(--surface);
  }

  input::placeholder,
  textarea::placeholder {
    color: var(--faint);
  }

  .title {
    padding: 2px 4px;
    margin-bottom: 8px;
    font: var(--fw-semibold) var(--fs-title) / var(--lh-title) var(--font-text);
  }

  .fields {
    display: grid;
    grid-template-columns: 76px 1fr;
    row-gap: 6px;
    column-gap: 8px;
    align-items: center;
    padding: 0 4px;
  }

  .key {
    font: var(--fw-semibold) var(--fs-caption) / 18px var(--font-text);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
    color: var(--faint);
  }

  .fields input.plain {
    padding: 2px 4px;
    margin-left: -4px;
  }

  .when {
    position: relative;
    font-variant-numeric: tabular-nums;
  }

  .when .native {
    position: absolute;
    inset: 0;
    opacity: 0;
    width: 100%;
  }

  .people {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .person {
    height: 22px;
    padding: 0 9px;
    border-radius: 11px;
    background: var(--surface);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
  }

  .person .remove {
    display: none;
    color: var(--faint);
  }

  .person:hover .remove {
    display: inline;
  }

  .person.add {
    color: var(--muted);
  }

  .person-input {
    width: 90px;
    border: 0;
    outline: none;
    color: var(--fg);
    user-select: text;
  }

  .section {
    margin: 12px 4px 4px;
  }

  /* Grows with its text (also text loaded or reloaded from the file) up to 12 lines,
     then scrolls inside with the panel's thin scrollbar. It never shrinks to make room:
     the editor scrolls instead. */
  .notes {
    flex: none;
    field-sizing: content;
    min-height: 48px;
    max-height: 248px;
    padding: 4px;
    font: var(--fs-body) / 20px var(--font-text);
    resize: none;
    overflow-y: auto;
  }

  .ai-head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 12px 4px 4px;
  }

  .into select {
    border: 0;
    outline: none;
    background: transparent;
    color: var(--muted);
    font: inherit;
  }

  .tiny {
    height: 24px;
    font-size: var(--fs-caption);
  }

  .ai {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 4px;
    border-bottom: 1px solid var(--hairline);
  }

  .ai-text {
    flex: 1;
    min-width: 0;
    padding: 2px 4px;
  }

  .ai.done .ai-text {
    color: var(--faint);
    text-decoration: line-through;
  }

  .mini {
    flex: none;
    height: 24px;
    padding: 0 8px;
    border-radius: 12px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font: var(--fw-semibold) var(--fs-caption) / 14px var(--font-text);
    color: var(--accent);
    background: var(--focus-tint);
  }

  .mini:hover {
    background: #d4a84b40;
  }

  .added {
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font: var(--fw-semibold) var(--fs-caption) / 14px var(--font-text);
    color: var(--done);
  }

  .ai .x {
    visibility: hidden;
  }

  .ai:hover .x {
    visibility: visible;
  }

  .new-ai {
    margin-top: 8px;
  }

  .status {
    padding: 10px 4px 0;
  }

  .window .title {
    font-size: 20px;
    line-height: 28px;
  }
</style>
