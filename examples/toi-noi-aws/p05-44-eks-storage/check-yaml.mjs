// Kiểm mọi YAML trong k8s/ parse được: node check-yaml.mjs <đường dẫn js-yaml/index.js>
// Ví dụ từ gốc repo: node examples/toi-noi-aws/p05-44-eks-storage/check-yaml.mjs api/node_modules/js-yaml/index.js
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const { default: yaml } = await import(pathToFileURL(path.resolve(process.argv[2] ?? 'node_modules/js-yaml/index.js')).href);
const dir = path.join(path.dirname(new URL(import.meta.url).pathname), 'k8s');
let bad = 0;

for (const f of fs.readdirSync(dir).filter((x) => x.endsWith('.yaml')).sort()) {
  try {
    const docs = yaml.loadAll(fs.readFileSync(path.join(dir, f), 'utf8')).filter(Boolean);
    console.log(`ok  k8s/${f}: ${docs.map((d) => `${d.kind}/${d.metadata?.name}`).join(', ')}`);
  } catch (e) {
    bad++;
    console.log(`LỖI k8s/${f}: ${e.message}`);
  }
}
process.exit(bad ? 1 : 0);
