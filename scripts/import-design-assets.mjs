// Copies the SVGs the app uses from design/assets into src/assets, without the
// provenance <metadata> block the design tool embeds (~8 KB per file).
// Run from the repository root: node scripts/import-design-assets.mjs
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const groups = {
  icons: null, // every file
  eyes: null,
  ornaments: ['kawung-mark.svg', 'kawung-tile.svg', 'aksara-karsa.svg'],
};

let before = 0;
let after = 0;
for (const [group, only] of Object.entries(groups)) {
  const from = join('design', 'assets', group);
  const to = join('src', 'assets', group);
  mkdirSync(to, { recursive: true });
  for (const file of only ?? readdirSync(from).filter((f) => f.endsWith('.svg'))) {
    const source = readFileSync(join(from, file), 'utf8');
    const clean = source
      .replace(/<metadata>[\s\S]*?<\/metadata>/g, '')
      .replace(/\s+xmlns:c2pa="[^"]*"/g, '')
      .trim();
    writeFileSync(join(to, file), clean + '\n');
    before += source.length;
    after += clean.length + 1;
  }
}
console.log(`imported: ${(before / 1024).toFixed(1)} KB -> ${(after / 1024).toFixed(1)} KB`);
