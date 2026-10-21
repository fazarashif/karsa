/**
 * The Javanese touches: pasaran and weton, time-of-day greetings, and a proverb of the day.
 * Everything is computed from the local date; nothing is fetched.
 */

export const PASARAN = ['Legi', 'Pahing', 'Pon', 'Wage', 'Kliwon'] as const;
export type Pasaran = (typeof PASARAN)[number];

/** Weekday names as weton are usually written (Sunday first, like Date.getDay()). */
const DINA = ['Minggu', 'Senin', 'Selasa', 'Rabu', 'Kamis', 'Jumat', 'Sabtu'];

const DAY = 86_400_000;
/** 17 August 1945 was Jumat Legi. Day numbers count local calendar days, so DST and
    time zones never shift the result. */
const ANCHOR = Date.UTC(1945, 7, 17);

function dayNumber(d: Date) {
  return Math.round((Date.UTC(d.getFullYear(), d.getMonth(), d.getDate()) - ANCHOR) / DAY);
}

export function pasaran(d: Date): Pasaran {
  return PASARAN[((dayNumber(d) % 5) + 5) % 5];
}

/** "Selasa Kliwon" */
export function weton(d: Date) {
  return `${DINA[d.getDay()]} ${pasaran(d)}`;
}

/** Local date from "YYYY-MM-DD", or null. */
export function parseDate(value: string) {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  return m ? new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3])) : null;
}

export type SpecialDay = 'birthday' | 'weton' | null;

/** The user's birthday, or the day their weton comes round again (every 35 days). */
export function specialDay(now: Date, birthday: string): SpecialDay {
  const born = parseDate(birthday);
  if (!born) return null;
  if (born.getMonth() === now.getMonth() && born.getDate() === now.getDate()) return 'birthday';
  return (dayNumber(now) - dayNumber(born)) % 35 === 0 ? 'weton' : null;
}

/** Time-of-day greeting, the special-day greetings, and the user's name when set. */
export function greeting(now: Date, profile: { name: string; birthday: string }) {
  const special = specialDay(now, profile.birthday);
  const h = now.getHours();
  const base =
    special === 'birthday'
      ? 'Sugeng ambal warsa'
      : special === 'weton'
        ? 'Sugeng wetonan'
        : h >= 5 && h < 11
          ? 'Sugeng enjing'
          : h >= 11 && h < 15
            ? 'Sugeng siang'
            : h >= 15 && h < 18
              ? 'Sugeng sonten'
              : 'Sugeng dalu';
  return profile.name ? `${base}, ${profile.name}` : base;
}

/** Javanese proverbs (paribasan and pitutur) with their meaning in English. */
const PARIBASAN: [string, string][] = [
  ['Alon-alon waton kelakon', 'Slowly, as long as it gets done.'],
  ['Sepi ing pamrih, rame ing gawe', 'Little self-interest, plenty of work.'],
  ['Becik ketitik, ala ketara', 'In time, the good is noticed and the bad shows.'],
  ['Jer basuki mawa beya', 'Every success has its cost.'],
  ['Aja kakehan gludhug kurang udan', 'Don’t be all thunder and no rain.'],
  ['Ana dina ana upa', 'Each day brings its own rice.'],
  ['Kebat kliwat, gancang pincang', 'Rush and you overshoot; hurry and you stumble.'],
  ['Aja nggege mangsa', 'Don’t hurry the season.'],
  ['Ngluruk tanpa bala, menang tanpa ngasorake', 'Advance without an army; win without humiliating anyone.'],
  ['Sugih tanpa bandha', 'Rich without possessions.'],
  ['Urip iku urup', 'To live is to give light.'],
  ['Ajining diri saka lathi', 'Your worth comes from your words.'],
  ['Aja dumeh', 'Don’t be proud of what you happen to have.'],
  ['Nrima ing pandum', 'Accept your share with grace.'],
  ['Ing madya mangun karsa', 'In the middle of things, build the will.'],
  ['Kriwikan dadi grojogan', 'A trickle left alone becomes a waterfall.'],
  ['Sabar subur', 'Patience makes things grow.'],
  ['Ngelmu iku kalakone kanthi laku', 'Knowledge comes through practice.'],
  ['Sapa temen bakal tinemu', 'Whoever is earnest will find it.'],
  ['Wani ngalah luhur wekasane', 'Whoever dares to yield ends up honoured.'],
  ['Memayu hayuning bawana', 'Make the world a little more beautiful.'],
  ['Gemi, nastiti, ngati-ati', 'Be thrifty, thorough and careful.'],
  ['Cancut taliwanda', 'Tie up your sleeves and get to work.'],
  ['Rukun agawe santosa', 'Harmony makes you strong.'],
  ['Tuna satak bathi sanak', 'Lose a little money, gain a friend.'],
  ['Desa mawa cara, negara mawa tata', 'Every village has its ways, every country its rules.'],
  ['Ana rembug dirembug', 'If something needs talking over, talk it over.'],
  ['Sing sapa nandur bakal ngundhuh', 'Whoever plants will harvest.'],
];

/** The same proverb all day, a different one each day. */
export function paribasan(now: Date) {
  const [text, meaning] = PARIBASAN[((dayNumber(now) % PARIBASAN.length) + PARIBASAN.length) % PARIBASAN.length];
  return { text, meaning };
}
