<!-- Quick capture, opened by the global hotkey from any app: type, Enter, and focus goes
     straight back to where the user was. Tab switches between a task and a note. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { onEscape } from '../escape';
  import Icon from '../Icon.svelte';
  import { notes } from '../notes.svelte';
  import { settings } from '../settings.svelte';
  import { INBOX_ID, todo } from '../todo.svelte';
  import { panel } from '../ui.svelte';

  type Kind = 'task' | 'note';
  let kind = $state<Kind>('task');
  let text = $state('');
  let saved = $state<string | null>(null);
  let error = $state<string | null>(null);
  let input = $state<HTMLTextAreaElement>();

  onMount(() => {
    input?.focus();
    return onEscape(() => panel.closeAndReturn());
  });

  async function save() {
    const value = text.trim();
    if (!value || saved) return;
    error = null;
    if (kind === 'task') {
      // A task is one line; anything after the first line is dropped.
      await todo.createTask(INBOX_ID, value.split('\n')[0]);
      if (todo.error) {
        error = todo.error;
        return;
      }
      saved = 'Added to Inbox';
    } else {
      try {
        await notes.save(null, value, null);
        saved = 'Saved to Notes';
      } catch (e) {
        error = String(e);
        return;
      }
    }
    setTimeout(() => panel.closeAndReturn(), 700);
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      e.preventDefault();
      kind = kind === 'task' ? 'note' : 'task';
    } else if (e.key === 'Enter' && !(kind === 'note' && e.shiftKey)) {
      e.preventDefault();
      void save();
    }
  }
</script>

<div class="view capture">
  <div class="bar">
    <span class="caps">Quick capture</span>
    <span class="grow"></span>
    {#if settings.current.captureHotkey}<span class="faint key">{settings.current.captureHotkey}</span>{/if}
  </div>

  <div class="seg" role="radiogroup" aria-label="Capture as">
    {#each [['task', 'Task', 'todo'], ['note', 'Note', 'notes']] as [id, label, icon] (id)}
      <button role="radio" aria-checked={kind === id} class:on={kind === id} onclick={() => ((kind = id as Kind), input?.focus())}>
        <Icon name={icon} size={14} />{label}
      </button>
    {/each}
  </div>

  <textarea
    bind:this={input}
    bind:value={text}
    class="box"
    rows={kind === 'note' ? 6 : 2}
    maxlength={kind === 'task' ? 500 : 20000}
    placeholder={kind === 'task' ? 'What needs doing?' : 'Write it down. The first line becomes the title.'}
    aria-label={kind === 'task' ? 'New task' : 'New note'}
    {onkeydown}
  ></textarea>

  {#if saved}
    <p class="saved"><Icon name="check-circle" size={16} />{saved}</p>
  {:else if error}
    <p class="error meta">{error}</p>
  {:else}
    <p class="faint hint">
      Enter to save{kind === 'note' ? ' · Shift+Enter for a new line' : ''} · Tab for {kind === 'task' ? 'a note' : 'a task'} · Esc to close
    </p>
  {/if}
</div>

<style>
  .capture {
    gap: 10px;
  }

  .key {
    font-size: var(--fs-caption);
  }

  .seg {
    display: flex;
    padding: 2px;
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
  }

  .seg button.on {
    background: var(--surface-hover);
    color: var(--fg);
  }

  .box {
    box-sizing: border-box;
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-control);
    background: var(--surface);
    color: var(--fg);
    font: inherit;
    line-height: var(--lh-body);
    resize: none;
    outline: none;
    caret-color: var(--accent);
    user-select: text;
  }

  .box:focus {
    border-color: #d4a84b80;
  }

  .hint,
  .saved,
  .error {
    margin: 0;
    font-size: var(--fs-caption);
  }

  .saved {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--done);
    font-size: var(--fs-body);
  }

  .error {
    color: var(--danger);
  }
</style>
