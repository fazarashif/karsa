/**
 * Meeting notes in Markdown: a small YAML header, a "## Notes" section and a
 * "## Action items" checklist. An item already copied to Todo carries a hidden
 * `<!-- karsa-task:ID -->` marker, so it is never added twice. Any other sections and
 * header keys a user writes (in Karsa or another editor, e.g. Obsidian `tags`) are kept
 * as they are.
 */

export type ActionItem = { text: string; done: boolean; taskId: number | null };

export type Meeting = {
  title: string;
  /** Local "YYYY-MM-DDTHH:MM". */
  date: string;
  topic: string;
  attendees: string[];
  notes: string;
  actions: ActionItem[];
  /** Sections other than Notes and Action items, verbatim. */
  rest: string;
  /** Header lines for keys Karsa does not use, verbatim. */
  extra: string[];
};

const TASK_MARK = /\s*<!--\s*karsa-task:(\d+)\s*-->\s*$/;
const ITEM = /^\s*[-*]\s+\[( |x|X)\]\s?(.*)$/;

const KNOWN_KEYS = ['type', 'title', 'date', 'topic', 'attendees'];

function splitFrontMatter(text: string): { fields: Record<string, string>; extra: string[]; body: string } {
  const lines = text.split(/\r?\n/);
  if (lines[0] !== '---') return { fields: {}, extra: [], body: text };
  const end = lines.indexOf('---', 1);
  if (end < 0) return { fields: {}, extra: [], body: text };
  const fields: Record<string, string> = {};
  const extra: string[] = [];
  const attendeeItems: string[] = [];
  let key = '';
  for (const line of lines.slice(1, end)) {
    const kv = /^([^\s:#-][^:]*):(.*)$/.exec(line);
    if (kv) {
      key = kv[1].trim();
      fields[key] = kv[2].trim();
    }
    if (!KNOWN_KEYS.includes(key)) {
      extra.push(line);
    } else if (!kv && key === 'attendees') {
      // A YAML block list ("attendees:" then "  - Alex" lines), as other editors write it.
      const item = /^\s*-\s+(.+)$/.exec(line);
      if (item) attendeeItems.push(unquote(item[1].trim()));
    }
  }
  if (attendeeItems.length && !fields.attendees) fields.attendees = JSON.stringify(attendeeItems);
  return { fields, extra, body: lines.slice(end + 1).join('\n') };
}

function unquote(value = '') {
  if (value.startsWith('"')) {
    try {
      return JSON.parse(value) as string;
    } catch {
      return value.replace(/^"|"$/g, '');
    }
  }
  return value;
}

function parseList(value = '') {
  const inner = value.trim().replace(/^\[|\]$/g, '').trim();
  if (!inner) return [];
  try {
    return (JSON.parse(`[${inner}]`) as unknown[]).map(String);
  } catch {
    return inner.split(',').map((s) => unquote(s.trim())).filter(Boolean);
  }
}

export function isMeeting(text: string) {
  return splitFrontMatter(text).fields.type === 'meeting';
}

export function parseMeeting(text: string): Meeting {
  const { fields, extra, body } = splitFrontMatter(text);
  const sections: { heading: string; lines: string[] }[] = [{ heading: '', lines: [] }];
  for (const line of body.split(/\r?\n/)) {
    const h = /^##\s+(.*)$/.exec(line);
    if (h) sections.push({ heading: h[1].trim(), lines: [] });
    else sections.at(-1)!.lines.push(line);
  }
  const find = (name: string) => sections.find((s) => s.heading.toLowerCase() === name);
  const actions: ActionItem[] = [];
  for (const line of find('action items')?.lines ?? []) {
    const m = ITEM.exec(line);
    if (!m) continue;
    const mark = TASK_MARK.exec(m[2]);
    actions.push({ text: m[2].replace(TASK_MARK, '').trim(), done: m[1] !== ' ', taskId: mark ? Number(mark[1]) : null });
  }
  const rest = sections
    .filter((s) => !['notes', 'action items'].includes(s.heading.toLowerCase()))
    .map((s) => (s.heading ? `## ${s.heading}\n${s.lines.join('\n')}` : s.lines.join('\n')))
    .join('\n')
    .trim();
  return {
    title: unquote(fields.title),
    date: fields.date ?? '',
    topic: unquote(fields.topic),
    attendees: parseList(fields.attendees),
    notes: (find('notes')?.lines ?? []).join('\n').trim(),
    actions,
    rest,
    extra,
  };
}

export function serializeMeeting(m: Meeting) {
  const q = (s: string) => JSON.stringify(s);
  const items = m.actions
    .filter((a) => a.text.trim())
    .map((a) => `- [${a.done ? 'x' : ' '}] ${a.text.trim()}${a.taskId ? ` <!-- karsa-task:${a.taskId} -->` : ''}`);
  return [
    '---',
    'type: meeting',
    `title: ${q(m.title.trim() || 'Meeting')}`,
    `date: ${m.date}`,
    `topic: ${q(m.topic.trim())}`,
    `attendees: [${m.attendees.map(q).join(', ')}]`,
    ...m.extra,
    '---',
    '',
    '## Notes',
    '',
    ...(m.notes.trim() ? [m.notes.trim(), ''] : []),
    '## Action items',
    '',
    ...(items.length ? [...items, ''] : []),
    ...(m.rest ? [m.rest, ''] : []),
  ].join('\n');
}

const pad = (n: number) => String(n).padStart(2, '0');

export function toLocalDateTime(ms: number) {
  const d = new Date(ms);
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function fromLocalDateTime(value: string) {
  const [date, time = '00:00'] = value.split('T');
  const [y, mo, d] = date.split('-').map(Number);
  const [h, mi] = time.split(':').map(Number);
  return new Date(y, mo - 1, d, h, mi).getTime();
}

export function newMeeting(title: string, at: number): string {
  return serializeMeeting({ title, date: toLocalDateTime(at), topic: '', attendees: [], notes: '', actions: [], rest: '', extra: [] });
}

/** A plain note's title: its first line, without Markdown heading marks. */
export function noteTitle(text: string) {
  const first = text.split(/\r?\n/).find((l) => l.trim());
  return first?.trim().replace(/^#+\s*/, '') || 'Untitled';
}
