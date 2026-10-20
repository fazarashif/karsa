<script lang="ts">
  import { onMount } from 'svelte';
  import { focus } from '../focus.svelte';
  import Icon from '../Icon.svelte';
  import { settings } from '../settings.svelte';
  import Stepper from './Stepper.svelte';
  import Switch from './Switch.svelte';

  const s = $derived(settings.current);
  const timer = $derived(focus.state.settings);

  const leads = [
    { label: 'At time', minutes: 0 },
    { label: '5m', minutes: 5 },
    { label: '10m', minutes: 10 },
    { label: '15m', minutes: 15 },
    { label: '30m', minutes: 30 },
  ];

  // Start with Windows can change outside Karsa (Task Manager's Startup apps).
  onMount(() => void settings.load());
</script>

<div class="view">
  <div class="scroll">
    <section>
      <h3 class="caps">General</h3>
      <div class="row">
        <span class="label grow">Start with Windows</span>
        <Switch checked={s.autostart} label="Start with Windows" onchange={(v) => settings.update({ autostart: v })} />
      </div>
      <div class="row">
        <span class="grow">
          <span class="label">Hide during fullscreen</span>
          <span class="hint">Games, videos and presentations keep the whole screen. Alerts wait until they end.</span>
        </span>
        <Switch
          checked={s.hideInFullscreen}
          label="Hide during fullscreen"
          onchange={(v) => settings.update({ hideInFullscreen: v })}
        />
      </div>
    </section>

    <section>
      <h3 class="caps">Focus</h3>
      <div class="row">
        <span class="label grow">Focus</span>
        <Stepper label="Focus length" value={timer.focusMin} min={5} max={120} step={5} unit="min"
          onchange={(v) => focus.updateSettings({ focusMin: v })} />
      </div>
      <div class="row">
        <span class="label grow">Short break</span>
        <Stepper label="Short break length" value={timer.shortBreakMin} min={1} max={30} unit="min"
          onchange={(v) => focus.updateSettings({ shortBreakMin: v })} />
      </div>
      <div class="row">
        <span class="label grow">Long break</span>
        <Stepper label="Long break length" value={timer.longBreakMin} min={5} max={60} step={5} unit="min"
          onchange={(v) => focus.updateSettings({ longBreakMin: v })} />
      </div>
      <div class="row">
        <span class="label grow">Long break after</span>
        <Stepper label="Sessions before a long break" value={timer.sessionsPerCycle} min={2} max={8}
          unit={timer.sessionsPerCycle === 1 ? 'session' : 'sessions'}
          onchange={(v) => focus.updateSettings({ sessionsPerCycle: v })} />
      </div>
      <div class="row">
        <span class="grow">
          <span class="label">Start the next phase by itself</span>
          <span class="hint">Otherwise Karsa waits for you after each focus session and break.</span>
        </span>
        <Switch
          checked={timer.autoStart}
          label="Start the next phase by itself"
          onchange={(v) => focus.updateSettings({ autoStart: v })}
        />
      </div>
    </section>

    <section>
      <h3 class="caps">Reminders</h3>
      <div class="block">
        <span class="label">Remind me</span>
        <div class="chips" role="radiogroup" aria-label="Default lead time">
          {#each leads as lead (lead.minutes)}
            <button
              class="chip"
              class:on={s.reminderLeadMin === lead.minutes}
              role="radio"
              aria-checked={s.reminderLeadMin === lead.minutes}
              onclick={() => settings.update({ reminderLeadMin: lead.minutes })}>{lead.label}</button
            >
          {/each}
        </div>
        <span class="hint">The starting point for new reminders and for todo deadlines with a time.</span>
      </div>
    </section>

    <section>
      <h3 class="caps">Notes</h3>
      <div class="block">
        <span class="label">Folder</span>
        <span class="path" title={s.notesFolder}>{s.notesFolder}</span>
        <div class="actions">
          <button class="btn sm" onclick={settings.chooseNotesFolder}>Change…</button>
          <button class="btn sm" onclick={settings.openNotesFolder}>Open</button>
          {#if s.notesFolderCustom}
            <button class="btn text" onclick={settings.resetNotesFolder}>Use default</button>
          {/if}
        </div>
        <span class="hint">Pick a OneDrive folder to back notes up. Notes already saved stay where they are.</span>
      </div>
    </section>

    {#if settings.error}<p class="error meta">{settings.error}</p>{/if}

    <footer class="about faint">
      <Icon name="ornaments/kawung-mark" size={8} />Karsa {s.version}
    </footer>
  </div>
</div>

<style>
  section {
    padding-bottom: 12px;
  }

  h3 {
    margin: 4px 4px 2px;
  }

  .row {
    box-sizing: border-box;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 44px;
    padding: 6px 4px;
    border-bottom: 1px solid var(--hairline);
  }

  .row:last-child {
    border-bottom: 0;
  }

  .row .grow {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .block {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding: 6px 4px;
  }

  .hint {
    font-size: var(--fs-caption);
    line-height: var(--lh-caption);
    color: var(--faint);
  }

  .chips {
    display: flex;
    gap: 6px;
  }

  .path {
    max-width: 100%;
    font-size: var(--fs-caption);
    line-height: var(--lh-caption);
    color: var(--muted);
    overflow-wrap: anywhere;
    user-select: text;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .error {
    margin: 0 4px 12px;
    color: var(--danger);
  }

  .about {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 4px 0 8px;
    font-size: var(--fs-caption);
    color: var(--faint);
  }

  .about :global(.icon) {
    color: var(--accent);
  }
</style>
