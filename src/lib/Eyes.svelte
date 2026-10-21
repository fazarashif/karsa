<script lang="ts" module>
  /** Mripat expressions; the names are internal and never shown. */
  export type Expression = 'melek' | 'fokus' | 'leren' | 'mesem' | 'merem' | 'bungah' | 'mrengut';
</script>

<script lang="ts">
  import Icon from './Icon.svelte';
  import { crossfade } from './motion';
  import { pointer } from './pointer.svelte';

  let {
    expression = 'melek',
    width = 28,
    blinkKey = 0,
    glance = false,
  }: { expression?: Expression; width?: number; blinkKey?: number; glance?: boolean } = $props();

  let el = $state<HTMLSpanElement>();

  // Kedhep: blink once each time blinkKey changes (on hover). Never on a timer.
  let blinking = $state(false);
  $effect(() => {
    if (blinkKey === 0) return;
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

<span
  bind:this={el}
  class="eyes"
  class:blinking
  style:width="{width}px"
  style:height="{width / 2}px"
  style:--gx="{look.x.toFixed(2)}px"
  style:--gy="{look.y.toFixed(2)}px"
  onanimationend={() => (blinking = false)}
>
  {#key expression}
    <span class="face" in:crossfade out:crossfade>
      <Icon name="eyes/eyes-{expression}" {width} height={width / 2} />
    </span>
  {/key}
</span>

<style>
  .eyes {
    position: relative;
    display: inline-block;
    flex: none;
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

  .blinking {
    animation: kedhep var(--dur-blink) var(--ease-standard);
  }

  @keyframes kedhep {
    50% {
      transform: scaleY(0.18);
    }
  }
</style>
