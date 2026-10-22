<script lang="ts">
  import CalendarTab from './calendar/CalendarTab.svelte';
  import { clock } from './clock.svelte';
  import { SHAPE } from './config';
  import FocusTab from './focus/FocusTab.svelte';
  import Eyes from './Eyes.svelte';
  import { formatDate, formatTime } from './format.svelte';
  import Icon from './Icon.svelte';
  import { greeting, pasaran } from './jawa';
  import { media } from './media.svelte';
  import { mood } from './mood.svelte';
  import { settings } from './settings.svelte';
  import { tabIn, tabOut } from './motion';
  import NotesTab from './notes/NotesTab.svelte';
  import NowPlaying from './nowplaying/NowPlaying.svelte';
  import Capture from './capture/Capture.svelte';
  import Rampung from './rampung/Rampung.svelte';
  import ShelfTab from './shelf/ShelfTab.svelte';
  import RemindersTab from './reminders/RemindersTab.svelte';
  import SettingsTab from './settings/SettingsTab.svelte';
  import TodoTab from './todo/TodoTab.svelte';
  import { panel, type Tab } from './ui.svelte';

  const tabs: { id: Tab; label: string; icon?: string }[] = [
    { id: 'todo', label: 'Todo' },
    { id: 'focus', label: 'Focus' },
    { id: 'notes', label: 'Notes' },
    { id: 'reminders', label: 'Reminders' },
    { id: 'calendar', label: 'Calendar', icon: 'calendar-month' },
    { id: 'shelf', label: 'Shelf', icon: 'shelf' },
    { id: 'settings', label: 'Settings', icon: 'settings' },
  ];

  const shape = $derived(SHAPE[panel.view]);

  let tabEls = $state<HTMLButtonElement[]>([]);
  let underline = $state({ x: 0, width: 0 });

  $effect(() => {
    const el = tabEls[tabs.findIndex((t) => t.id === panel.tab)];
    if (el) underline = { x: el.offsetLeft, width: el.offsetWidth };
  });
</script>

<!-- The panel takes its view's shape at once; the island box morphs to it around the
     content (design v2, 3.1), so the content never reflows during the spring. -->
{#if panel.nowPlaying}
  <NowPlaying onclose={panel.collapse} onback={panel.nowPlayingFrom === 'panel' ? () => (panel.nowPlaying = false) : undefined} />
{:else}
<div class="panel" style:width="{shape.width}px" style:height="{shape.height}px" aria-keyshortcuts="Escape">
  <span class="wm"><Icon name="ornaments/aksara-karsa" width={147} height={60} /></span>

  <header class="beat" style="--i: 0">
    <span class="greet">
      <Icon name="ornaments/kawung-mark" size={8} /><span class="ell grow">{greeting(clock.now, settings.current)}</span>
      <Eyes expression={mood(clock.now)} width={24} glance />
    </span>
    <span class="date-row">
      <span class="date">{formatDate(clock.now)}<span class="pasaran">{pasaran(clock.now)}</span></span>
      {#if media.state.active}
        <button
          class="playing"
          class:paused={!media.state.playing}
          title="Now playing · {media.state.title}"
          onclick={() => panel.showNowPlaying('panel')}
        >
          {#if media.state.appIcon}<img src={media.state.appIcon} alt="" />{:else}<Icon name="music" size={12} />{/if}
          <span class="ell">{media.state.title}</span>
        </button>
      {/if}
      <span class="muted num">{formatTime(clock.now)}</span>
    </span>
  </header>

  {#if panel.capture}
    <div class="body ritual" in:tabIn>
      <div class="pane"><Capture /></div>
    </div>
  {:else if panel.rampung}
    <div class="body ritual" in:tabIn>
      <div class="pane"><Rampung /></div>
    </div>
  {:else}
  <div class="tabs beat" style="--i: 1" role="tablist">
    {#each tabs as t, i (t.id)}
      <button
        bind:this={tabEls[i]}
        class="tab"
        class:on={panel.tab === t.id}
        class:icon-tab={t.icon}
        role="tab"
        aria-selected={panel.tab === t.id}
        aria-label={t.icon ? t.label : undefined}
        title={t.icon ? t.label : undefined}
        onclick={() => (panel.tab = t.id)}
      >
        {#if t.icon}<Icon name={t.icon} size={16} />{:else}{t.label}{/if}
      </button>
    {/each}
    <span
      class="underline"
      class:ready={underline.width > 0}
      style:transform="translateX({underline.x}px) scaleX({underline.width})"
    ></span>
  </div>

  <div class="body beat" style="--i: 2">
    {#key panel.tab}
      <div class="pane" role="tabpanel" in:tabIn out:tabOut>
        {#if panel.tab === 'todo'}
          <TodoTab />
        {:else if panel.tab === 'focus'}
          <FocusTab />
        {:else if panel.tab === 'reminders'}
          <RemindersTab />
        {:else if panel.tab === 'settings'}
          <SettingsTab />
        {:else if panel.tab === 'calendar'}
          <CalendarTab />
        {:else if panel.tab === 'shelf'}
          <ShelfTab />
        {:else}
          <NotesTab />
        {/if}
      </div>
    {/key}
  </div>
  {/if}
</div>
{/if}

<style>
  .panel {
    position: relative;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    padding: 16px 16px 12px;
  }

  .wm {
    position: absolute;
    right: 76px;
    top: 6px;
    color: var(--gading);
    opacity: var(--o-watermark);
    pointer-events: none;
  }

  header {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 4px 12px;
  }

  .greet {
    display: flex;
    align-items: center;
    gap: 6px;
    font: var(--fw-semibold) var(--fs-caption) / var(--lh-caption) var(--font-text);
    letter-spacing: var(--tracking-greeting);
    color: var(--accent);
  }

  .date-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  .date {
    flex: none;
    font: var(--fw-semibold) var(--fs-title-lg) / var(--lh-title-lg) var(--font-text);
  }

  /* What is playing, between the date and the time: opens Now playing. */
  .playing {
    flex: 0 1 auto;
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    max-width: 150px;
    height: 24px;
    margin: 0 8px 0 auto;
    padding: 0 10px 0 6px;
    border-radius: 12px;
    background: var(--surface);
    font: var(--fw-semibold) var(--fs-caption) / 16px var(--font-text);
    color: var(--fg);
    align-self: center;
  }

  .playing:hover {
    background: var(--surface-hover);
  }

  .playing.paused {
    color: var(--muted);
  }

  .playing img {
    flex: none;
    width: 12px;
    height: 12px;
    border-radius: 3px;
  }

  /* Today's Javanese market day, after the date. */
  .pasaran {
    margin-left: 8px;
    font: var(--fw-semibold) var(--fs-caption) / var(--lh-caption) var(--font-text);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
    color: var(--muted);
  }

  .tabs {
    position: relative;
    display: flex;
    gap: var(--tab-gap);
    padding: 0 4px;
    border-bottom: 1px solid var(--hairline);
  }

  .tab {
    height: var(--tab-h);
    font-weight: var(--fw-semibold);
    color: var(--muted);
    border-radius: 6px;
    transition: color var(--dur-fast) var(--ease-standard);
  }

  .tab:hover,
  .tab.on {
    color: var(--fg);
  }

  .tab:focus-visible {
    outline-offset: -2px;
  }

  /* Calendar, Shelf and Settings: icons at the far end of the tab row, 4 px apart. */
  .tab.icon-tab {
    margin-left: auto;
    width: 28px;
    display: grid;
    place-items: center;
  }

  .tab.icon-tab + .tab.icon-tab {
    margin-left: calc(4px - var(--tab-gap));
  }

  /* One 1 px bar scaled to the active tab's width, so moving it only animates transform. */
  .underline {
    position: absolute;
    left: 0;
    bottom: -1px;
    width: 1px;
    height: 2px;
    border-radius: 1px;
    background: var(--accent);
    transform-origin: 0 0;
    opacity: 0;
  }

  .underline.ready {
    opacity: 1;
    transition: transform var(--dur-tab) var(--ease-standard);
  }

  /* One column as wide as the panel: an auto column would grow to fit a long unwrapped
     line, such as a note's title, and push the tab past the panel's edge. */
  .body {
    position: relative;
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    padding-top: 12px;
  }

  /* The ritual takes the tabs' place, separated from the header by the same hairline. */
  .body.ritual {
    border-top: 1px solid var(--hairline);
  }

  .pane {
    grid-area: 1 / 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
</style>
