// Kiểm mọi tệp YAML trong lab có parse được không: node check-yaml.mjs [đường dẫn js-yaml]
// Ví dụ từ gốc repo: node examples/toi-noi-aws/p05-42-eks-terraform/check-yaml.mjs api/node_modules/js-yaml/index.js
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const yamlPath = path.resolve(process.argv[2] ?? 'node_modules/js-yaml/index.js');
const { default: yaml } = await import(pathToFileURL(yamlPath).href);
const here = path.dirname(new URL(import.meta.url).pathname);
let bad = 0;

for (const dir of ['k8s', 'eksctl']) {
  for (const f of fs.readdirSync(path.join(here, dir)).filter((x) => x.endsWith('.yaml'))) {
    const file = path.join(here, dir, f);
    try {
      const docs = yaml.loadAll(fs.readFileSync(file, 'utf8')).filter(Boolean);
      const kinds = docs.map((d) => `${d.kind}${d.metadata?.name ? '/' + d.metadata.name : ''}`);
      console.log(`ok  ${dir}/${f}: ${kinds.join(', ')}`);
    } catch (e) {
      bad++;
      console.log(`LỖI ${dir}/${f}: ${e.message}`);
    }
  }
}
process.exit(bad ? 1 : 0);
