<script lang="ts">
  import { clock } from './clock.svelte';
  import FocusTab from './focus/FocusTab.svelte';
  import { formatDate, formatTime, greeting } from './format.svelte';
  import Icon from './Icon.svelte';
  import { tabIn, tabOut } from './motion';
  import NotesTab from './notes/NotesTab.svelte';
  import RemindersTab from './reminders/RemindersTab.svelte';
  import SettingsTab from './settings/SettingsTab.svelte';
  import TodoTab from './todo/TodoTab.svelte';
  import { panel, type Tab } from './ui.svelte';

  const tabs: { id: Tab; label: string; icon?: string }[] = [
    { id: 'todo', label: 'Todo' },
    { id: 'focus', label: 'Focus' },
    { id: 'notes', label: 'Notes' },
    { id: 'reminders', label: 'Reminders' },
    { id: 'settings', label: 'Settings', icon: 'settings' },
  ];

  let tabEls = $state<HTMLButtonElement[]>([]);
  let underline = $state({ x: 0, width: 0 });

  $effect(() => {
    const el = tabEls[tabs.findIndex((t) => t.id === panel.tab)];
    if (el) underline = { x: el.offsetLeft, width: el.offsetWidth };
  });
</script>

<div class="panel" aria-keyshortcuts="Escape">
  <span class="wm"><Icon name="ornaments/aksara-karsa" width={147} height={60} /></span>

  <header class="beat" style="--i: 0">
    <span class="greet"><Icon name="ornaments/kawung-mark" size={8} />{greeting(clock.now)}</span>
    <span class="date-row">
      <span class="date">{formatDate(clock.now)}</span>
      <span class="muted num">{formatTime(clock.now)}</span>
    </span>
  </header>

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
        {:else}
          <NotesTab />
        {/if}
      </div>
    {/key}
  </div>
</div>

<style>
  .panel {
    position: relative;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    width: var(--panel-w);
    height: var(--panel-h);
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
    font: var(--fw-semibold) var(--fs-title-lg) / var(--lh-title-lg) var(--font-text);
  }

  .tabs {
    position: relative;
    display: flex;
    gap: 20px;
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

  /* Settings: an icon at the far end of the tab row. */
  .tab.icon-tab {
    margin-left: auto;
    width: 28px;
    display: grid;
    place-items: center;
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

  .body {
    position: relative;
    flex: 1;
    min-height: 0;
    display: grid;
    padding-top: 12px;
  }

  .pane {
    grid-area: 1 / 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
</style>
