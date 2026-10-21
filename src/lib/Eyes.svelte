<script lang="ts" module>
  /** Mripat expressions; the names are internal and never shown. */
  export type Expression =
    | 'melek'
    | 'kedhep'
    | 'fokus'
    | 'leren'
    | 'mesem'
    | 'merem'
    | 'bungah'
    | 'mrengut'
    | 'nyanyi'
    | 'kaget'
    | 'seneng'
    | 'gumun';

  /** Eyes only; a small head with the plain blangkon; a large head with the kawung
      blangkon and brows (design v2, 2.1). */
  export type Head = 'eyes' | 'small' | 'large';

  /** Each expression's brows, shown on the large head only. */
  const BROWS: Record<Expression, string> = {
    melek: 'neutral',
    kedhep: 'neutral',
    fokus: 'focus',
    leren: 'soft',
    mesem: 'soft',
    merem: 'soft',
    bungah: 'raised',
    mrengut: 'frown',
    nyanyi: 'raised',
    kaget: 'raised',
    seneng: 'soft',
    gumun: 'raised',
  };

  /** Frame height and the eyes' top, in units of the 28-wide grid. */
  const FRAME: Record<Head, { height: number; eyesTop: number }> = {
    eyes: { height: 14, eyesTop: 0 },
    small: { height: 21, eyesTop: 7 },
    large: { height: 26, eyesTop: 12 },
  };
</script>

<script lang="ts">
  import Icon from './Icon.svelte';
  import { crossfade } from './motion';
  import { pointer } from './pointer.svelte';
  import { reaction } from './reactions.svelte';

  let {
    expression = 'melek',
    width = 28,
    head = 'eyes',
    blinkKey = 0,
    glance = false,
  }: { expression?: Expression; width?: number; head?: Head; blinkKey?: number; glance?: boolean } = $props();

  let el = $state<HTMLSpanElement>();
  const unit = $derived(width / 28);
  const frame = $derived(FRAME[head]);

  // Kedhep: blink once each time blinkKey changes (on hover), and once when a reaction
  // ends and the usual face returns. Never on a timer.
  let blinking = $state(false);
  let lastEnded = reaction.ended;
  $effect(() => {
    const ended = reaction.ended;
    if (blinkKey === 0 && ended === lastEnded) return;
    lastEnded = ended;
    blinking = false;
    requestAnimationFrame(() => (blinking = true));
  });

  // The pupils lean toward the cursor while it moves over the island, at most ~1.3 px.
  const look = $derived.by(() => {
    const p = pointer.position;
    if (!glance || !p || !el) return { x: 0, y: 0 };
    const r = el.getBoundingClientRect();
    const dx = p.x - (r.left + r.width / 2);
    const dy = p.y - (r.top + r.height / 2);
    const distance = Math.hypot(dx, dy) || 1;
    const reach = Math.min(1, distance / 90);
    return { x: (dx / distance) * reach * 1.3, y: (dy / distance) * reach * 0.9 };
  });
</script>

<span class="head" data-expression={expression} style:width="{width}px" style:height="{frame.height * unit}px">
  {#if head !== 'eyes'}
    <span class="layer" style:top="0" style:height="{11 * unit}px">
      <Icon name="mascot/blangkon-{head === 'large' ? 'kawung' : 'plain'}" {width} height={11 * unit} />
    </span>
  {/if}
  {#if head === 'large'}
    {#key BROWS[expression]}
      <span class="layer" style:top="{9 * unit}px" style:height="{4 * unit}px" in:crossfade out:crossfade>
        <Icon name="mascot/brows-{BROWS[expression]}" {width} height={4 * unit} />
      </span>
    {/key}
  {/if}
  <span
    bind:this={el}
    class="layer eyes"
    class:blinking
    style:top="{frame.eyesTop * unit}px"
    style:height="{14 * unit}px"
    style:--gx="{look.x.toFixed(2)}px"
    style:--gy="{look.y.toFixed(2)}px"
    onanimationend={() => (blinking = false)}
  >
    {#key expression}
      <span class="face" in:crossfade out:crossfade>
        <Icon name="mascot/eyes-{expression}" {width} height={14 * unit} />
      </span>
    {/key}
  </span>
</span>

<style>
  .head {
    position: relative;
    display: inline-block;
    flex: none;
  }

  .layer {
    position: absolute;
    left: 0;
    width: 100%;
  }

  .face {
    position: absolute;
    inset: 0;
  }

  /* Pupils and their highlights. */
  .eyes :global([fill='#3B2414']),
  .eyes :global([fill='#fff']) {
    transform: translate(var(--gx), var(--gy));
    transition: transform var(--dur-fast) var(--ease-standard);
  }

  /* Only the eyes blink; the blangkon and brows stay put. */
  .blinking {
    animation: kedhep var(--dur-blink) var(--ease-standard);
  }

  @keyframes kedhep {
    50% {
      transform: scaleY(0.18);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .blinking {
      animation: none;
    }

    .eyes :global([fill='#3B2414']),
    .eyes :global([fill='#fff']) {
      transform: none;
    }
  }
</style>
