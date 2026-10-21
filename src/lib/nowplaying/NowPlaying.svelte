<!-- Now playing (design v2, 4.9 and 6.2): wide and short, with the lyrics below when they
     are open. Controls the app does not support are left out, not greyed out. -->
<script lang="ts">
  import Eyes from '../Eyes.svelte';
  import Icon from '../Icon.svelte';
  import { lyrics } from '../lyrics.svelte';
  import { formatPlayTime, media, positionAt, sourceLabel } from '../media.svelte';
  import { onEscape } from '../escape';
  import { settings } from '../settings.svelte';
  import Switch from '../settings/Switch.svelte';
  import { panel } from '../ui.svelte';

  let { onclose, onback }: { onclose: () => void; onback?: () => void } = $props();

  // Opened from the panel: Esc goes back to the tabs instead of closing the island.
  $effect(() => {
    if (onback) return onEscape(onback);
  });

  const s = $derived(media.state);

  // The seek bar and times move once a second, only while playing and shown.
  let now = $state(Date.now());
  $effect(() => {
    if (!s.playing) return;
    now = Date.now();
    const t = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(t);
  });
  /** While dragging the seek bar: the position under the pointer. */
  let scrub = $state<number | null>(null);
  const position = $derived(scrub ?? positionAt(s, now));
  const fraction = $derived(s.durationMs > 0 ? position / s.durationMs : 0);

  let bar = $state<HTMLDivElement>();
  function scrubTo(e: PointerEvent) {
    if (!bar) return;
    const r = bar.getBoundingClientRect();
    scrub = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * s.durationMs;
  }
  function onSeekDown(e: PointerEvent) {
    if (!s.canSeek) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    scrubTo(e);
  }
  function onSeekUp() {
    if (scrub !== null) media.control({ kind: 'seek', ms: Math.round(scrub) });
    scrub = null;
  }

  // Sessions: with two or more apps playing, the source row picks between them.
  let switcher = $state(false);
  $effect(() => {
    if (!switcher) return;
    return onEscape(() => (switcher = false));
  });

  // Volume: click mutes, the wheel moves it by 5%, hovering shows the level.
  function onWheel(e: WheelEvent) {
    if (s.volume === null) return;
    e.preventDefault();
    const level = Math.max(0, Math.min(1, s.volume + (e.deltaY < 0 ? 0.05 : -0.05)));
    media.control({ kind: 'volume', level: Math.round(level * 100) / 100 });
  }
  const volumeIcon = $derived(s.muted || s.volume === 0 ? 'volume-mute' : (s.volume ?? 1) < 0.5 ? 'volume-low' : 'volume');
  const nextRepeat = { off: 'all', all: 'one', one: 'off' } as const;
  const repeatLabel = { off: 'Repeat', all: 'Repeat one', one: 'Repeat off' } as const;

  // Lyrics are looked up once the area is open (and only when the user turned them on).
  $effect(() => {
    if (panel.lyricsOpen && settings.current.lyricsEnabled && s.active) void lyrics.load();
  });

  function onKeydown(e: KeyboardEvent) {
    const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;
    if (typing || e.ctrlKey || e.altKey) return;
    if (e.key === ' ') media.control({ kind: 'toggle' });
    else if (e.key === 'ArrowLeft' && e.shiftKey) media.control({ kind: 'previous' });
    else if (e.key === 'ArrowRight' && e.shiftKey) media.control({ kind: 'next' });
    else if (e.key === 'ArrowLeft' && s.canSeek) media.control({ kind: 'seek', ms: Math.max(0, position - 5000) });
    else if (e.key === 'ArrowRight' && s.canSeek) media.control({ kind: 'seek', ms: Math.min(s.durationMs, position + 5000) });
    else if (e.key.toLowerCase() === 'm' && s.volume !== null) media.control({ kind: 'mute', on: !s.muted });
    else return;
    e.preventDefault();
  }

  const offset = $derived(lyrics.offsetMs === 0 ? '0 s' : `${lyrics.offsetMs > 0 ? '+' : '−'}${Math.abs(lyrics.offsetMs) / 1000} s`);
</script>

<svelte:window onkeydown={onKeydown} />

<div class="np" class:with-lyrics={panel.lyricsOpen}>
  <div class="source beat" style="--i: 0">
    {#if onback}
      <button class="ibtn back" aria-label="Back" title="Back (Esc)" onclick={onback}><Icon name="chevron-left" size={16} /></button>
    {/if}
    <button
      class="who"
      class:many={s.sessions.length > 1}
      disabled={s.sessions.length < 2}
      aria-haspopup={s.sessions.length > 1 ? 'menu' : undefined}
      aria-expanded={switcher}
      onclick={() => (switcher = !switcher)}
    >
      {#if s.appIcon}<img class="app-icon" src={s.appIcon} alt="" />{:else}<Icon name="source-app" size={16} />{/if}
      <span class="ell">{sourceLabel(s)}</span>
      {#if s.sessions.length > 1}
        <span class="count num">{s.sessions.length}</span>
        <Icon name={switcher ? 'chevron-up' : 'chevron-down'} size={12} />
      {/if}
    </button>
    {#if switcher}
      <div class="menu sessions" role="menu">
        {#each s.sessions as session (session.id)}
          <button
            class="mi"
            role="menuitemradio"
            aria-checked={session.id === s.id}
            onclick={() => {
              media.control({ kind: 'select', id: session.id });
              switcher = false;
            }}
          >
            <span class="ell grow">{session.app} · <span class="muted">{session.title || 'Nothing playing'}</span></span>
            <span class="faint">{session.playing ? 'Playing' : 'Paused'}</span>
            {#if session.id === s.id}<Icon name="check" size={12} />{/if}
          </button>
        {/each}
      </div>
    {/if}
    <span class="grow"></span>
    <button
      class="ibtn"
      class:on={panel.lyricsOpen}
      aria-label="Lyrics"
      title="Lyrics"
      aria-pressed={panel.lyricsOpen}
      onclick={() => (panel.lyricsOpen = !panel.lyricsOpen)}
    ><Icon name="lyrics" size={16} /></button>
    <button class="ibtn" aria-label="Close" title="Close" onclick={onclose}><Icon name="close" size={16} /></button>
  </div>

  <div class="main beat" style="--i: 1">
    {#key s.art}
      {#if s.art}
        <img class="art" src={s.art} alt="" />
      {:else}
        <span class="art placeholder"><Icon name="music" size={24} /></span>
      {/if}
    {/key}
    <span class="text">
      <span class="title ell">{s.title || 'Nothing playing'}</span>
      <span class="by ell muted">{[s.artist, s.album].filter(Boolean).join(' · ')}</span>
    </span>
  </div>

  {#if s.durationMs > 0}
    <div class="seek beat" style="--i: 2">
      <div
        class="bar"
        class:live={s.canSeek}
        class:scrubbing={scrub !== null}
        bind:this={bar}
        role="slider"
        tabindex="-1"
        aria-label="Position"
        aria-valuemin={0}
        aria-valuemax={Math.round(s.durationMs / 1000)}
        aria-valuenow={Math.round(position / 1000)}
        onpointerdown={onSeekDown}
        onpointermove={(e) => scrub !== null && scrubTo(e)}
        onpointerup={onSeekUp}
        onpointercancel={() => (scrub = null)}
      >
        <span class="track"><i style:transform="scaleX({fraction})"></i></span>
        <span class="knob" style:left="{fraction * 100}%"></span>
      </div>
      <span class="times num muted"><span>{formatPlayTime(position)}</span><span>−{formatPlayTime(s.durationMs - position)}</span></span>
    </div>
  {/if}

  <div class="controls beat" style="--i: 3">
    {#if s.canShuffle}
      <button class="ibtn ctl" class:on={s.shuffle} aria-label="Shuffle" title="Shuffle" aria-pressed={s.shuffle}
        onclick={() => media.control({ kind: 'shuffle', on: !s.shuffle })}><Icon name="shuffle" size={18} /></button>
    {/if}
    <button class="ibtn ctl" aria-label="Previous" title="Previous" disabled={!s.canPrevious}
      onclick={() => media.control({ kind: 'previous' })}><Icon name="previous" size={18} /></button>
    <button class="play" aria-label={s.playing ? 'Pause' : 'Play'} onclick={() => media.control({ kind: 'toggle' })}>
      <Icon name={s.playing ? 'pause' : 'play'} size={18} />
    </button>
    <button class="ibtn ctl" aria-label="Next" title="Next" disabled={!s.canNext}
      onclick={() => media.control({ kind: 'next' })}><Icon name="next" size={18} /></button>
    {#if s.canRepeat}
      <button class="ibtn ctl" class:on={s.repeat !== 'off'} aria-label={repeatLabel[s.repeat]} title={repeatLabel[s.repeat]}
        onclick={() => media.control({ kind: 'repeat', mode: nextRepeat[s.repeat] })}><Icon name={s.repeat === 'one' ? 'repeat-one' : 'repeat'} size={18} /></button>
    {/if}
    {#if s.volume !== null}
      <span class="volume">
        <button class="ibtn ctl" aria-label={s.muted ? 'Unmute' : 'Mute'} title="{s.muted ? 'Unmute' : 'Mute'} · scroll for volume"
          onclick={() => media.control({ kind: 'mute', on: !s.muted })} onwheel={onWheel}><Icon name={volumeIcon} size={18} /></button>
        <span class="level num">{s.muted ? 'Muted' : `${Math.round((s.volume ?? 0) * 100)}%`}</span>
      </span>
    {/if}
  </div>

  {#if panel.lyricsOpen}
    <div class="lyrics">
      {#if !settings.current.lyricsEnabled}
        <div class="state">
          <span class="caps faint">Lyrics are off</span>
          <span class="note faint">Turning them on sends the title, artist, album and length of what is playing to lrclib.net. Nothing else leaves your PC.</span>
          <button class="btn sm" onclick={() => settings.update({ lyricsEnabled: true })}>Turn on lyrics</button>
        </div>
      {:else}
        <div class="lyrics-bar">
          <span class="caps faint">{lyrics.status === 'plain' ? 'Lyrics · not timed' : 'Lyrics · lrclib.net'}</span>
          <span class="grow"></span>
          {#if lyrics.status === 'timed'}
            <button class="nudge" aria-label="Earlier" title="Lyrics earlier" onclick={() => lyrics.nudge(-500)}>−</button>
            <span class="offset num faint">Timing {offset}</span>
            <button class="nudge" aria-label="Later" title="Lyrics later" onclick={() => lyrics.nudge(500)}>+</button>
          {/if}
          <span class="focus-label faint">Lyric focus</span>
          <Switch checked={settings.current.lyricFocus} label="Lyric focus" onchange={(v) => settings.update({ lyricFocus: v })} />
        </div>
        {#if lyrics.status === 'timed'}
          <div class="lines" aria-live="off">
            <div class="list" style:transform="translateY({-Math.max(0, lyrics.current - 1) * 24}px)">
              {#each lyrics.lines as [, text], i (i)}
                <span class="line ell" class:sung={i < lyrics.current} class:now={i === lyrics.current}>{text || '♪'}</span>
              {/each}
            </div>
          </div>
        {:else if lyrics.status === 'plain'}
          <div class="plain scroll">{lyrics.plain}</div>
        {:else if lyrics.status === 'loading'}
          <div class="state">
            <Eyes expression="melek" width={28} head="small" />
            <span class="muted">Looking for lyrics…</span>
            <span class="skeleton"><i style:width="70%"></i><i style:width="55%"></i><i style:width="62%"></i></span>
          </div>
        {:else}
          <div class="state">
            <span style:color="var(--ornament)"><Icon name="ornaments/kawung-mark" size={16} /></span>
            <span class="muted">
              {lyrics.status === 'instrumental' ? 'An instrumental, no words to show' : lyrics.status === 'error' ? lyrics.message : 'No lyrics found for this song'}
            </span>
            {#if lyrics.status === 'missing'}<span class="note faint">Karsa asked once and will not ask again for this track.</span>{/if}
            {#if lyrics.status !== 'instrumental'}<button class="btn sm" onclick={() => lyrics.load(true)}>Search again</button>{/if}
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .np {
    position: relative;
    box-sizing: border-box;
    width: 520px;
    height: 212px;
    padding: 16px;
    display: flex;
    flex-direction: column;
  }

  .np.with-lyrics {
    height: 420px;
  }

  .source {
    position: relative;
    display: flex;
    align-items: center;
    gap: 2px;
    height: 24px;
  }

  .back {
    margin-right: 2px;
  }

  .who {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    max-width: 360px;
    height: 24px;
    padding: 0 6px 0 2px;
    border-radius: 8px;
    font: var(--fw-semibold) 13px/18px var(--font-text);
    color: var(--muted);
  }

  .who.many:hover {
    background: var(--surface);
  }

  .who:disabled {
    opacity: 1;
    cursor: default;
  }

  .app-icon {
    flex: none;
    width: 16px;
    height: 16px;
    border-radius: var(--radius-art-sm);
  }

  .count {
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    box-sizing: border-box;
    border-radius: 9px;
    background: var(--surface-hover);
    font-size: var(--fs-caption);
    line-height: 18px;
    color: var(--fg);
  }

  .sessions {
    left: 0;
    top: 28px;
    min-width: 280px;
    z-index: 2;
  }

  .sessions .mi {
    gap: 8px;
  }

  .grow {
    flex: 1;
  }

  .ibtn.on {
    color: var(--accent);
  }

  .main {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-top: 10px;
  }

  .art {
    flex: none;
    width: 56px;
    height: 56px;
    border-radius: var(--radius-art);
    object-fit: cover;
  }

  .art.placeholder {
    display: grid;
    place-items: center;
    background: #3a2a1e;
    color: #c0855299;
  }

  .text {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .title {
    font: var(--fw-semibold) 15px/20px var(--font-text);
  }

  .seek {
    margin-top: 10px;
  }

  .bar {
    position: relative;
    height: var(--seek-hit);
    display: flex;
    align-items: center;
  }

  .bar.live {
    cursor: pointer;
  }

  .track {
    position: relative;
    flex: 1;
    height: var(--seek-h);
    border-radius: 3px;
    background: var(--track);
    overflow: hidden;
    transition: height var(--dur-fast) var(--ease-standard);
  }

  .track i {
    position: absolute;
    inset: 0;
    background: var(--gading);
    transform-origin: 0 0;
  }

  .knob {
    position: absolute;
    top: 50%;
    width: 12px;
    height: 12px;
    margin: -6px 0 0 -6px;
    border-radius: 50%;
    background: var(--gading);
    opacity: 0;
  }

  .bar.live:hover .track,
  .bar.scrubbing .track {
    height: 6px;
  }

  .bar.live:hover .knob,
  .bar.scrubbing .knob {
    opacity: 1;
  }

  .times {
    display: flex;
    justify-content: space-between;
    margin-top: 2px;
    font-size: var(--fs-caption);
    line-height: 14px;
  }

  .controls {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 20px;
    margin-top: 4px;
  }

  .ctl {
    width: 32px;
    height: 32px;
    color: var(--fg);
  }

  .ctl.on {
    color: var(--accent);
  }

  .play {
    display: grid;
    place-items: center;
    width: var(--round-button-lg);
    height: var(--round-button-lg);
    border-radius: 50%;
    background: var(--gading);
    color: #000;
    transition: background var(--dur-fast) var(--ease-standard), transform var(--dur-fast) var(--ease-standard);
  }

  .play:hover {
    background: #f7f0e1;
  }

  .play:active {
    background: #dcd2bc;
    transform: scale(0.95);
  }

  .volume {
    position: absolute;
    right: 0;
    display: flex;
    align-items: center;
  }

  /* The level shows while hovering the volume button. */
  .level {
    position: absolute;
    right: 36px;
    padding: 0 6px;
    border-radius: 9px;
    background: var(--surface-hover);
    font-size: var(--fs-caption);
    line-height: 18px;
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--dur-fast) var(--ease-standard);
  }

  .volume:hover .level {
    opacity: 1;
  }

  .lyrics {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    margin-top: 14px;
    padding-top: 10px;
    border-top: 1px solid var(--hairline);
  }

  .lyrics-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 24px;
  }

  .nudge {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--surface);
    color: var(--fg);
  }

  .nudge:hover {
    background: var(--surface-hover);
  }

  .offset {
    min-width: 72px;
    text-align: center;
    font-size: var(--fs-caption);
  }

  .focus-label {
    margin-left: 10px;
    font-size: var(--fs-caption);
  }

  .lines {
    position: relative;
    flex: 1;
    min-height: 0;
    margin-top: 6px;
    overflow: hidden;
  }

  .list {
    display: flex;
    flex-direction: column;
    transition: transform 220ms var(--ease-out);
  }

  .line {
    height: 24px;
    font-size: 13px;
    line-height: 24px;
    color: var(--muted);
    transition: color 220ms var(--ease-out);
  }

  .line.sung {
    color: var(--faint);
  }

  .line.now {
    font: var(--fw-semibold) 15px/24px var(--font-text);
    color: var(--gading);
  }

  .plain {
    margin-top: 6px;
    white-space: pre-wrap;
    line-height: 20px;
    color: var(--muted);
    user-select: text;
  }

  .state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    text-align: center;
  }

  .note {
    max-width: 380px;
    font-size: var(--fs-caption);
    line-height: var(--lh-caption);
  }

  .skeleton {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    width: 240px;
    margin-top: 4px;
  }

  .skeleton i {
    height: 8px;
    border-radius: 4px;
    background: var(--surface-hover);
  }

  @media (prefers-reduced-motion: reduce) {
    .list,
    .line {
      transition: none;
    }
  }
</style>
