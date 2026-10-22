/**
 * Indonesia's tanggal merah: public holidays (libur nasional) and collective leave (cuti
 * bersama), as the joint decree of three ministers (SKB 3 Menteri) sets them each year.
 *
 * Fixed-date holidays and the Easter ones are worked out for any year. The others follow
 * the Islamic, Chinese, Saka and Buddhist calendars and are decided by the government, so
 * they are listed per year below; add the next year when its decree comes out (usually in
 * September). Sources: the SKB for 2025 (with its August 2025 change), 2026 and 2027, as
 * published by setneg.go.id and the national press.
 */

export type Holiday = { name: string; /** Cuti bersama rather than a public holiday. */ leave: boolean };

type Year = { holidays: [string, string][]; leave: [string, string][] };

const LISTED: Record<number, Year> = {
  2025: {
    holidays: [
      ['01-27', 'Isra Mikraj'],
      ['01-29', 'Tahun Baru Imlek 2576'],
      ['03-29', 'Hari Suci Nyepi'],
      ['03-31', 'Idulfitri 1446 H'],
      ['04-01', 'Idulfitri 1446 H'],
      ['05-12', 'Waisak 2569 BE'],
      ['06-06', 'Iduladha 1446 H'],
      ['06-27', 'Tahun Baru Islam 1447 H'],
      ['09-05', 'Maulid Nabi Muhammad'],
    ],
    leave: [
      ['01-28', 'Tahun Baru Imlek 2576'],
      ['03-28', 'Hari Suci Nyepi'],
      ['04-02', 'Idulfitri 1446 H'],
      ['04-03', 'Idulfitri 1446 H'],
      ['04-04', 'Idulfitri 1446 H'],
      ['04-07', 'Idulfitri 1446 H'],
      ['05-13', 'Waisak 2569 BE'],
      ['05-30', 'Kenaikan Yesus Kristus'],
      ['06-09', 'Iduladha 1446 H'],
      ['08-18', 'Hari Kemerdekaan RI'],
      ['12-26', 'Natal'],
    ],
  },
  2026: {
    holidays: [
      ['01-16', 'Isra Mikraj'],
      ['02-17', 'Tahun Baru Imlek 2577'],
      ['03-19', 'Hari Suci Nyepi'],
      ['03-21', 'Idulfitri 1447 H'],
      ['03-22', 'Idulfitri 1447 H'],
      ['05-27', 'Iduladha 1447 H'],
      ['05-31', 'Waisak 2570 BE'],
      ['06-16', 'Tahun Baru Islam 1448 H'],
      ['08-25', 'Maulid Nabi Muhammad'],
    ],
    leave: [
      ['02-16', 'Tahun Baru Imlek 2577'],
      ['03-18', 'Hari Suci Nyepi'],
      ['03-20', 'Idulfitri 1447 H'],
      ['03-23', 'Idulfitri 1447 H'],
      ['03-24', 'Idulfitri 1447 H'],
      ['05-15', 'Kenaikan Yesus Kristus'],
      ['05-28', 'Iduladha 1447 H'],
      ['12-24', 'Natal'],
    ],
  },
  2027: {
    holidays: [
      ['01-05', 'Isra Mikraj'],
      ['02-06', 'Tahun Baru Imlek 2578'],
      ['03-08', 'Hari Suci Nyepi'],
      ['03-10', 'Idulfitri 1448 H'],
      ['03-11', 'Idulfitri 1448 H'],
      ['05-17', 'Iduladha 1448 H'],
      ['05-20', 'Waisak 2571 BE'],
      ['06-06', 'Tahun Baru Islam 1449 H'],
      ['08-15', 'Maulid Nabi Muhammad'],
      // The Islamic year is shorter, so 27 Rajab comes twice in 2027.
      ['12-26', 'Isra Mikraj'],
    ],
    leave: [
      ['02-05', 'Tahun Baru Imlek 2578'],
      ['03-09', 'Idulfitri 1448 H'],
      ['03-12', 'Idulfitri 1448 H'],
      ['03-15', 'Idulfitri 1448 H'],
      ['03-25', 'Wafat Yesus Kristus'],
      ['05-18', 'Iduladha 1448 H'],
      ['05-19', 'Waisak 2571 BE'],
      ['12-24', 'Natal'],
    ],
  },
};

const FIXED: [string, string][] = [
  ['01-01', 'Tahun Baru Masehi'],
  ['05-01', 'Hari Buruh'],
  ['06-01', 'Hari Lahir Pancasila'],
  ['08-17', 'Hari Kemerdekaan RI'],
  ['12-25', 'Natal'],
];

const pad = (n: number) => String(n).padStart(2, '0');
const monthDay = (d: Date) => `${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;

/** Easter Sunday (Gregorian), by the anonymous algorithm in Meeus. */
function easter(year: number) {
  const a = year % 19;
  const b = Math.floor(year / 100);
  const c = year % 100;
  const h = (19 * a + b - Math.floor(b / 4) - Math.floor((b - Math.floor((b + 8) / 25) + 1) / 3) + 15) % 30;
  const l = (32 + 2 * (b % 4) + 2 * Math.floor(c / 4) - h - (c % 4)) % 7;
  const m = Math.floor((a + 11 * h + 22 * l) / 451);
  const n = h + l - 7 * m + 114;
  return new Date(year, Math.floor(n / 31) - 1, (n % 31) + 1);
}

const years = new Map<number, Map<string, Holiday>>();

function yearOf(year: number) {
  let days = years.get(year);
  if (days) return days;
  days = new Map();
  const add = (key: string, name: string, leave: boolean) => days!.set(key, { name, leave });
  for (const [key, name] of FIXED) add(key, name, false);
  const sunday = easter(year);
  const shifted = (days: number) => new Date(sunday.getFullYear(), sunday.getMonth(), sunday.getDate() + days);
  add(monthDay(shifted(-2)), 'Wafat Yesus Kristus', false);
  add(monthDay(sunday), 'Paskah', false);
  add(monthDay(shifted(39)), 'Kenaikan Yesus Kristus', false);
  for (const [key, name] of LISTED[year]?.holidays ?? []) add(key, name, false);
  // Leave days never replace a holiday on the same date.
  for (const [key, name] of LISTED[year]?.leave ?? []) if (!days.has(key)) add(key, name, true);
  years.set(year, days);
  return days;
}

/** The day's public holiday or cuti bersama, or null. */
export function holidayOn(day: Date): Holiday | null {
  return yearOf(day.getFullYear()).get(monthDay(day)) ?? null;
}

/** Whether all of the year's holidays are known, not just the fixed and Easter ones. */
export const holidaysListed = (year: number) => year in LISTED;
