/**
 * Esc closes the innermost open thing first (a menu, a detail view), and only closes
 * the panel when nothing else claims it. Components register while they are open.
 */
const handlers: (() => void)[] = [];

export function onEscape(handler: () => void): () => void {
  handlers.push(handler);
  return () => {
    const i = handlers.lastIndexOf(handler);
    if (i >= 0) handlers.splice(i, 1);
  };
}

/** Returns true when an open view handled Esc. */
export function handleEscape(): boolean {
  const handler = handlers.at(-1);
  handler?.();
  return handler !== undefined;
}
