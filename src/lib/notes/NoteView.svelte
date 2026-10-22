<!-- Loads a note and shows the right editor for it: plain or meeting. -->
<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import { inTauri } from '../bridge';
  import Icon from '../Icon.svelte';
  import { isMeeting } from '../meeting';
  import { NoteSession, notes, type NoteFile } from '../notes.svelte';
  import MeetingEditor from './MeetingEditor.svelte';
  import NoteEditor from './NoteEditor.svelte';

  let {
    path,
    project = '',
    layout,
    onback,
    ontitle,
  }: {
    path: string | null;
    /** Where a new note goes ("" for no project). */
    project?: string;
    layout: 'panel' | 'window';
    onback?: () => void;
    ontitle?: (title: string) => void;
  } = $props();

  let file = $state.raw<NoteFile | null>(null);
  let session = $state.raw<NoteSession | null>(null);
  let error = $state('');

  onMount(() => {
    void (async () => {
      try {
        if (path) file = await notes.read(path);
        session = new NoteSession(file?.path ?? null, file?.modified ?? null, project);
      } catch (e) {
        error = String(e);
      }
    })();
    // Unsaved edits are written before the panel view goes away…
    return () => void session?.flush();
  });

  // …and before a note window closes.
  $effect(() => {
    if (layout !== 'window' || !inTauri) return;
    const unlisten = getCurrentWindow().onCloseRequested(async () => {
      await session?.flush();
    });
    return () => void unlisten.then((off) => off());
  });

  async function leave() {
    await session?.flush();
    onback?.();
  }
</script>

{#if error}
  <div class="view">
    {#if layout === 'panel'}
      <div class="bar"><button class="back" onclick={() => onback?.()}><Icon name="chevron-left" size={16} />Notes</button></div>
    {/if}
    <div class="empty">
      <span style:color="var(--ornament)"><Icon name="notes" size={24} /></span>
      <span class="proverb">Note not found</span>
      <span class="muted">{error}</span>
    </div>
  </div>
{:else if session}
  {#if file && isMeeting(file.content)}
    <MeetingEditor {session} initial={file.content} {layout} onback={leave} {ontitle} />
  {:else}
    <NoteEditor {session} initial={file?.content ?? ''} {layout} onback={leave} {ontitle} />
  {/if}
{/if}
