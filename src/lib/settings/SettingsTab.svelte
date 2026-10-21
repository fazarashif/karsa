<script lang="ts">
  import { onMount } from 'svelte';
  import { focus } from '../focus.svelte';
  import { formatTime, fromInputs, toDateInput } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import { parseDate, weton } from '../jawa';
  import PickerField from '../PickerField.svelte';
  import { settings } from '../settings.svelte';
  import Stepper from './Stepper.svelte';
  import Switch from './Switch.svelte';

  const s = $derived(settings.current);
  const timer = $derived(focus.state.settings);
  const born = $derived(parseDate(s.birthday));
  const presets = ['Win+Shift+K', 'Ctrl+Alt+K', 'Alt+Shift+K', 'Ctrl+Shift+Space', 'Ctrl+Alt+Space', ''];
  // A shortcut set some other way still shows.
  const hotkeys = $derived(presets.includes(s.captureHotkey) ? presets : [s.captureHotkey, ...presets]);
  const birthdayFormat = new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'long', year: 'numeric' });

  const positions = [
    { label: 'Bottom right', value: 'corner' },
    { label: 'Top', value: 'top' },
  ] as const;

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
      <h3 class="caps">You</h3>
      <div class="block">
        <label class="field wide">
          <span class="label">Name</span>
          <span class="input">
            <input
              value={s.name}
              maxlength="40"
              placeholder="For the greeting"
              aria-label="Name"
              onchange={(e) => settings.update({ name: e.currentTarget.value })}
              onkeydown={(e) => e.key === 'Enter' && e.currentTarget.blur()}
            />
          </span>
        </label>
        <div class="field wide">
          <span class="label">Birthday</span>
          <PickerField
            type="date"
            value={s.birthday}
            display={born ? birthdayFormat.format(born) : ''}
            placeholder="Optional"
            label="Birthday"
            onchange={(value) => settings.update({ birthday: value })}
            onclear={() => settings.update({ birthday: '' })}
          />
        </div>
        <span class="hint">
          {#if born}Your weton is {weton(born)}. Karsa greets you on your birthday and every 35 days, on your weton.
          {:else}Karsa uses it to greet you on your birthday and on your weton day.{/if}
        </span>
      </div>
    </section>

    <section>
      <h3 class="caps">General</h3>
      <div class="block ruled">
        <span class="label">Position</span>
        <div class="chips" role="radiogroup" aria-label="Position">
          {#each positions as position (position.value)}
            <button
              class="chip"
              class:on={s.islandPosition === position.value}
              role="radio"
              aria-checked={s.islandPosition === position.value}
              onclick={() => settings.update({ islandPosition: position.value })}>{position.label}</button
            >
          {/each}
        </div>
        <span class="hint">
          {#if s.islandPosition === 'top'}A slim notch at the top of the screen that opens downwards.
          {:else}Above the taskbar, next to the clock.{/if}
        </span>
      </div>
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
      <h3 class="caps">Now playing</h3>
      <div class="row">
        <span class="grow">
          <span class="label">Show media in the pill</span>
          <span class="hint">Music or video from Spotify, a browser and the like, when nothing else is due. Peek always has the controls.</span>
        </span>
        <Switch
          checked={s.mediaInPill}
          label="Show media in the pill"
          onchange={(v) => settings.update({ mediaInPill: v })}
        />
      </div>
      <div class="row">
        <span class="grow">
          <span class="label">Lyrics</span>
          <span class="hint">Looks up lyrics on lrclib.net. It sends the title, artist, album and length of what is playing, and nothing else.</span>
        </span>
        <Switch checked={s.lyricsEnabled} label="Lyrics" onchange={(v) => settings.update({ lyricsEnabled: v })} />
      </div>
      <div class="row" class:off={!s.lyricsEnabled}>
        <span class="grow">
          <span class="label">Lyric focus</span>
          <span class="hint">Shows the current lyric line in the pill instead of the title. Needs Lyrics.</span>
        </span>
        <Switch
          checked={s.lyricsEnabled && s.lyricFocus}
          label="Lyric focus"
          onchange={(v) => s.lyricsEnabled && settings.update({ lyricFocus: v })}
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
      <h3 class="caps">Quick capture</h3>
      <div class="row">
        <span class="grow">
          <span class="label">Shortcut</span>
          <span class="hint">From any app: type a task or a note, press Enter, and you're back where you were.</span>
        </span>
        <span class="input sm hotkey">
          <select
            value={s.captureHotkey}
            aria-label="Quick capture shortcut"
            onchange={(e) => settings.update({ captureHotkey: e.currentTarget.value })}
          >
            {#each hotkeys as key (key)}
              <option value={key}>{key || 'Off'}</option>
            {/each}
          </select>
          <Icon name="chevron-down" size={12} />
        </span>
      </div>
      {#if s.captureHotkey && !s.captureHotkeyOk}
        <p class="error meta">Another app already uses {s.captureHotkey}. Pick another shortcut.</p>
      {/if}
    </section>

    <section>
      <h3 class="caps">Shelf</h3>
      <div class="row">
        <span class="grow">
          <span class="label">Clipboard history</span>
          <span class="hint">Keeps text you copy for 24 hours, encrypted with your Windows account. Turning it off deletes it.</span>
        </span>
        <Switch
          checked={s.clipboardEnabled}
          label="Clipboard history"
          onchange={(v) => settings.update({ clipboardEnabled: v })}
        />
      </div>
    </section>

    <section>
      <h3 class="caps">End of the day</h3>
      <div class="row">
        <span class="grow">
          <span class="label">Offer to wrap up the day</span>
          <span class="hint">Rampung: a short look back at today and ahead to tomorrow, on weekdays.</span>
        </span>
        <Switch
          checked={s.rampungEnabled}
          label="Offer to wrap up the day"
          onchange={(v) => settings.update({ rampungEnabled: v })}
        />
      </div>
      {#if s.rampungEnabled}
        <div class="row">
          <span class="label grow">At</span>
          <span class="time">
            <PickerField
              type="time"
              value={s.rampungTime}
              display={formatTime(fromInputs(toDateInput(Date.now()), s.rampungTime))}
              placeholder="17:00"
              label="Wrap-up time"
              onchange={(value) => value && settings.update({ rampungTime: value })}
            />
          </span>
        </div>
      {/if}
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

  /* A setting that needs another one turned on first. */
  .row.off {
    opacity: var(--o-disabled);
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

  .block.ruled {
    padding-bottom: 10px;
    border-bottom: 1px solid var(--hairline);
  }

  .field.wide {
    align-self: stretch;
  }

  .time {
    width: 120px;
  }

  .hotkey {
    flex: none;
    width: 150px;
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
