<!-- The root of an "Open full" note window: one note, full size, in its own window. -->
<script lang="ts" module>
  import tileSvg from './assets/ornaments/kawung-tile.svg?raw';

  // The kawung tile as a CSS background, in gading (the SVG itself uses currentColor).
  const tile = `url("data:image/svg+xml,${encodeURIComponent(tileSvg.replaceAll('currentColor', '#EFE6D2'))}")`;
</script>

<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import { core, inTauri } from './lib/bridge';
  import { prefs } from './lib/format.svelte';
  import NoteView from './lib/notes/NoteView.svelte';
  import { todo } from './lib/todo.svelte';

  let { path }: { path: string } = $props();

  onMount(() => {
    document.documentElement.classList.add('note-window');
    core.uses12HourClock().then((v) => (prefs.hour12 = v));
    todo.load();
  });

  let lastTitle = '';
  function ontitle(title: string) {
    if (!inTauri || title === lastTitle) return;
    lastTitle = title;
    void getCurrentWindow().setTitle(`${title} — Karsa`);
  }
</script>

<main class="page">
  <div class="tile" style:background-image={tile} aria-hidden="true"></div>
  <div class="content">
    <NoteView {path} layout="window" {ontitle} />
  </div>
</main>

<style>
  :global(html.note-window),
  :global(html.note-window body) {
    background: var(--island-bg);
  }

  .page {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
  }

  /* Kawung watermark across the top, at the design's 6% ceiling. */
  .tile {
    position: absolute;
    inset: 0 0 auto;
    height: 56px;
    background-size: 24px 24px;
    opacity: var(--o-watermark);
    pointer-events: none;
  }

  .content {
    position: relative;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    width: 100%;
    max-width: 760px;
    box-sizing: border-box;
    margin: 0 auto;
    padding: 16px 24px 16px;
  }
</style>
