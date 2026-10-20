/** Clock style from the Windows regional settings; names stay in English. */
let hour12 = $state(false);

export const prefs = {
  get hour12() {
    return hour12;
  },
  set hour12(value: boolean) {
    hour12 = value;
  },
};

const time24 = new Intl.DateTimeFormat('en-GB', { hour: '2-digit', minute: '2-digit', hourCycle: 'h23' });
const time12 = new Intl.DateTimeFormat('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
const date = new Intl.DateTimeFormat('en-GB', { weekday: 'short', day: 'numeric', month: 'short' });
const dayMonth = new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short' });
const weekday = new Intl.DateTimeFormat('en-GB', { weekday: 'short' });

export function formatTime(d: Date | number) {
  return (hour12 ? time12 : time24).format(d);
}

/** "Mon 19 Oct" */
export function formatDate(d: Date | number) {
  return date.format(d).replace(',', '');
}

/** Javanese time-of-day greeting shown above the date in the panel. */
export function greeting(now: Date) {
  const h = now.getHours();
  if (h >= 5 && h < 11) return 'Sugeng enjing';
  if (h >= 11 && h < 15) return 'Sugeng siang';
  if (h >= 15 && h < 18) return 'Sugeng sonten';
  return 'Sugeng dalu';
}

const DAY = 24 * 60 * 60 * 1000;

function startOfDay(d: Date | number) {
  const x = new Date(d);
  x.setHours(0, 0, 0, 0);
  return x.getTime();
}

type Due = { dueAt: number | null; dueHasTime: boolean; done?: boolean };

/** Days from today to the due day: 0 today, 1 tomorrow, negative when past. */
function dayOffset(dueAt: number, now: Date) {
  return Math.round((startOfDay(dueAt) - startOfDay(now)) / DAY);
}

/** Due today or overdue: shown in bata. */
export function isUrgent(task: Due, now: Date) {
  if (task.dueAt === null || task.done) return false;
  return dayOffset(task.dueAt, now) <= 0;
}

/** Task rows: "Today 15:00", "Tomorrow", "Tue 20 Oct 09:00". */
export function dueLabel(task: Due, now: Date) {
  if (task.dueAt === null) return '';
  const offset = dayOffset(task.dueAt, now);
  const day = offset === 0 ? 'Today' : offset === 1 ? 'Tomorrow' : formatDate(task.dueAt);
  return task.dueHasTime ? `${day} ${formatTime(task.dueAt)}` : day;
}

/** Peek rows, compact: "15:00" today, "Tue" this week, "20 Oct" later or overdue. */
export function dueLabelShort(task: Due, now: Date) {
  if (task.dueAt === null) return '';
  const offset = dayOffset(task.dueAt, now);
  if (offset === 0) return task.dueHasTime ? formatTime(task.dueAt) : 'Today';
  if (offset > 0 && offset < 7) return weekday.format(task.dueAt);
  return dayMonth.format(task.dueAt);
}

// Native date/time inputs work in local "YYYY-MM-DD" and "HH:MM".

const pad = (n: number) => String(n).padStart(2, '0');

export function toDateInput(ms: number) {
  const d = new Date(ms);
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function toTimeInput(ms: number) {
  const d = new Date(ms);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** Local midnight of the date, plus the time when given. */
export function fromInputs(dateValue: string, timeValue: string) {
  const [y, m, d] = dateValue.split('-').map(Number);
  const [hh, mm] = timeValue ? timeValue.split(':').map(Number) : [0, 0];
  return new Date(y, m - 1, d, hh, mm).getTime();
}

/** Local midnight `days` from today. */
export function dayFromToday(days: number) {
  const d = new Date();
  d.setHours(0, 0, 0, 0);
  d.setDate(d.getDate() + days);
  return d.getTime();
}
