// Removes <metadata> blocks and the c2pa namespace from every SVG under design/assets.
// Usage (from the repo root): node design/tools/strip-svg-metadata.mjs
import { readdir, readFile, writeFile } from 'node:fs/promises';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', 'assets');

async function walk(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) await walk(path);
    else if (entry.name.endsWith('.svg')) {
      const before = await readFile(path, 'utf8');
      const after = before
        .replace(/<metadata>[\s\S]*?<\/metadata>/g, '')
        .replace(/\s+xmlns:c2pa="[^"]*"/g, '')
        .trim() + '\n';
      if (after !== before) {
        await writeFile(path, after);
        console.log(`${path}: ${before.length} -> ${after.length} bytes`);
      }
    }
  }
}

await walk(root);
