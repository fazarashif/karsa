<script lang="ts">
  import { onEscape } from '../escape';
  import { newMeeting } from '../meeting';
  import { notes } from '../notes.svelte';
  import { panel } from '../ui.svelte';
  import NotesList from './NotesList.svelte';
  import NoteView from './NoteView.svelte';

  /** `key` remounts the editor for each note opened. */
  type View = { kind: 'list' } | { kind: 'open'; path: string | null; key: number };
  let view = $state<View>({ kind: 'list' });
  let error = $state('');

  const open = (path: string | null) => (view = { kind: 'open', path, key: Date.now() });
  const back = () => (view = { kind: 'list' });

  $effect(() => {
    void notes.refresh();
  });

  $effect(() => {
    if (view.kind === 'open') return onEscape(back);
  });

  // An open note takes the tall editor shape.
  $effect(() => {
    panel.editing = view.kind === 'open';
    return () => (panel.editing = false);
  });

  // Requests from elsewhere: a task's "Open meeting note", or a meeting reminder's alert.
  $effect(() => {
    const request = panel.noteRequest;
    if (!request) return;
    panel.noteRequest = null;
    if ('path' in request) open(request.path);
    else void createMeeting(request.meeting.title, request.meeting.at);
  });

  /** A meeting note is saved right away, so it exists even before anything is typed. */
  async function createMeeting(title: string, at: number) {
    try {
      const saved = await notes.save(null, newMeeting(title, at), null);
      open(saved.path);
    } catch (e) {
      error = String(e);
    }
  }
</script>

{#if view.kind === 'open'}
  {#key view.key}
    <NoteView path={view.path} layout="panel" onback={back} />
  {/key}
{:else}
  <NotesList
    {error}
    onopen={open}
    onnewnote={() => open(null)}
    onnewmeeting={() => createMeeting('Meeting', Date.now())}
  />
{/if}
