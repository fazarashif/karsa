<script lang="ts" module>
  import type { AlertPayload } from './bridge';
  import type { Expression } from './Eyes.svelte';

  /** What the collapsed island shows, in priority order (requirements 4.2). */
  export type PillContent =
    | { kind: 'alert'; alert: AlertPayload }
    | { kind: 'due'; title: string; detail: string; color: string }
    | { kind: 'focus'; isBreak: boolean; time: string; minutesLeft: number; paused: boolean; waiting: boolean }
    | { kind: 'media'; title: string; artist: string; art: string }
    | { kind: 'eyes'; expression: Expression };

  export const typeColor = (kind: AlertPayload['kind']) => `var(--${kind})`;
</script>

<script lang="ts">
  import Eyes from './Eyes.svelte';
  import Icon from './Icon.svelte';

  let {
    content,
    notch = false,
    blinkKey,
    onwidth,
    onclick,
  }: {
    content: PillContent;
    /** The slimmer pill of the notch at the top of the screen. */
    notch?: boolean;
    blinkKey: number;
    onwidth: (width: number) => void;
    onclick: () => void;
  } = $props();

  let width = $state(0);
  $effect(() => {
    if (width > 0) onwidth(width);
  });
  const eyesWidth = $derived(notch ? 24 : 28);

  const label = $derived.by(() => {
    switch (content.kind) {
      case 'alert':
        return `Alert: ${content.alert.title}`;
      case 'due':
        return `Karsa, ${content.title} ${content.detail}`;
      case 'focus':
        if (content.waiting) return `Karsa, ${content.isBreak ? 'break' : 'focus'} ready to start`;
        // Whole minutes, so the label (and the accessibility tree) changes once a minute.
        return `Karsa, ${content.isBreak ? 'break' : 'focus'}, ${content.minutesLeft} min left${content.paused ? ', paused' : ''}`;
      case 'media':
        return `Karsa, playing ${content.title}${content.artist ? ` by ${content.artist}` : ''}`;
      default:
        return 'Karsa, nothing due today';
    }
  });
</script>

<button
  class="pill beat"
  class:notch
  class:eyes-only={content.kind === 'eyes'}
  class:with-eyes={content.kind === 'focus'}
  bind:offsetWidth={width}
  {onclick}
  aria-label={label}
>
  {#if content.kind === 'eyes'}
    <Eyes expression={content.expression} width={eyesWidth} {blinkKey} glance />
  {:else if content.kind === 'focus'}
    <Eyes expression={content.waiting ? 'mesem' : content.isBreak ? 'leren' : 'fokus'} width={eyesWidth} />
    {#if content.waiting}
      <span class="label" style:color="var(--focus)">{content.isBreak ? 'Start break' : 'Start focus'}</span>
    {:else if content.isBreak}
      <span class="muted">Break</span>
      <span class="detail" class:muted={content.paused}>{content.time}</span>
    {:else}
      <span class="detail" style:color={content.paused ? 'var(--muted)' : 'var(--focus)'}>{content.time}</span>
    {/if}
  {:else if content.kind === 'media'}
    {#if content.art}
      <img class="art" src={content.art} alt="" />
    {:else}
      <span style:color="var(--accent)"><Icon name="music" size={14} /></span>
    {/if}
    <span class="label ell">{content.title}</span>
  {:else if content.kind === 'alert'}
    <span class="gong" style:color={typeColor(content.alert.kind)} style:--gong-delay="90ms">
      <Icon name="ornaments/kawung-mark" size={10} />
    </span>
    <span class="label ell">{content.alert.title}</span>
  {:else}
    <span style:color={content.color}><Icon name="ornaments/kawung-mark" size={10} /></span>
    <span class="label ell">{content.title}</span>
    <span class="detail" style:color={content.color}>{content.detail}</span>
  {/if}
</button>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    box-sizing: border-box;
    height: var(--pill-h);
    min-width: var(--pill-min-w);
    max-width: var(--pill-max-w);
    padding: 0 16px 0 14px;
    white-space: nowrap;
    font-weight: var(--fw-semibold);
    letter-spacing: var(--tracking-pill);
    font-variant-numeric: tabular-nums;
  }

  .pill.eyes-only {
    width: var(--pill-min-w);
    padding: 0;
  }

  .pill.with-eyes {
    padding-left: 12px;
    gap: 8px;
  }

  .pill.notch {
    height: var(--notch-h);
    min-width: var(--notch-min-w);
    max-width: var(--notch-max-w);
    padding: 0 14px 0 12px;
  }

  .pill.notch.eyes-only {
    width: var(--notch-min-w);
    padding: 0;
  }

  .pill.notch.with-eyes {
    padding-left: 10px;
  }

  .detail {
    flex: none;
  }

  /* Album art, as a small rounded square. */
  .art {
    flex: none;
    width: 18px;
    height: 18px;
    border-radius: 5px;
    object-fit: cover;
  }
</style>
