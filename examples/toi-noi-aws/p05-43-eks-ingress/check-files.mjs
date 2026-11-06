// Kiểm YAML và JSON trong lab parse được: node check-files.mjs <đường dẫn js-yaml/index.js>
// Ví dụ từ gốc repo: node examples/toi-noi-aws/p05-43-eks-ingress/check-files.mjs api/node_modules/js-yaml/index.js
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const { default: yaml } = await import(pathToFileURL(path.resolve(process.argv[2] ?? 'node_modules/js-yaml/index.js')).href);
const here = path.dirname(new URL(import.meta.url).pathname);
let bad = 0;

function walk(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    if (e.name === '.terraform') return [];
    const p = path.join(dir, e.name);
    return e.isDirectory() ? walk(p) : [p];
  });
}

for (const file of walk(here)) {
  const rel = path.relative(here, file);
  try {
    if (file.endsWith('.yaml')) {
      const docs = yaml.loadAll(fs.readFileSync(file, 'utf8')).filter(Boolean);
      console.log(`ok  ${rel}: ${docs.map((d) => d.kind ?? 'values').join(', ')}`);
    } else if (file.endsWith('.json')) {
      JSON.parse(fs.readFileSync(file, 'utf8'));
      console.log(`ok  ${rel}`);
    }
  } catch (e) {
    bad++;
    console.log(`LỖI ${rel}: ${e.message}`);
  }
}
process.exit(bad ? 1 : 0);
