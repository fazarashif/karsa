<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { noteTitle } from '../meeting';
  import type { NoteSession } from '../notes.svelte';
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

  let text = $state(untrack(() => initial));
  let area = $state<HTMLTextAreaElement>();
  const title = $derived(noteTitle(text));

  $effect(() => ontitle?.(title));

  onMount(() => {
    area?.focus();
    area?.setSelectionRange(text.length, text.length);
  });

  const status = $derived(
    {
      saved: 'Saved',
      saving: 'Saving…',
      unsaved: 'Edited',
      conflict: 'Not saved',
      error: session.error,
    }[session.status],
  );
</script>

<div class="view editor" class:window={layout === 'window'}>
  <NoteBar {session} {layout} {title} {onback} />
  {#if session.status === 'conflict'}
    <ConflictBanner onreload={async () => (text = await session.reload())} onkeep={() => session.keepMine()} />
  {/if}
  <textarea
    class="ed"
    bind:this={area}
    bind:value={text}
    oninput={() => session.edit(text)}
    placeholder="Title on the first line, then anything you like…"
    aria-label="Note"
    spellcheck="true"
  ></textarea>
  <div class="status meta">
    <span class:err={session.status === 'error'}>{status}</span>
    <span class="grow"></span>
    <span>Markdown</span>
  </div>
</div>

<style>
  .ed {
    flex: 1;
    min-height: 0;
    box-sizing: border-box;
    width: 100%;
    padding: 12px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-menu);
    background: var(--surface);
    color: var(--fg);
    font: var(--fs-body) / 20px var(--font-text);
    resize: none;
    outline: none;
    user-select: text;
    caret-color: var(--accent);
    transition: border-color var(--dur-fast) var(--ease-standard);
  }

  .ed:focus {
    border-color: #d4a84b80;
  }

  .ed::placeholder {
    color: var(--faint);
  }

  .window .ed {
    font-size: 14px;
    line-height: 22px;
    padding: 16px;
  }

  .status {
    padding: 10px 4px 0;
  }

  .err {
    color: var(--danger);
  }
</style>
