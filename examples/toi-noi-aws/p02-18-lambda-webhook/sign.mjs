// In giá trị header x-vixanh-signature cho một tệp body, dùng để thử bằng curl.
// Cách dùng: node sign.mjs <secret> sample-webhook.json
import { createHmac } from 'node:crypto';
import { readFileSync } from 'node:fs';

const [secret, file] = process.argv.slice(2);
if (!secret || !file) {
  console.error('Cách dùng: node sign.mjs <secret> <tệp-body>');
  process.exit(1);
}
const body = readFileSync(file, 'utf8');
const t = Math.floor(Date.now() / 1000);
const v1 = createHmac('sha256', secret).update(`${t}.${body}`).digest('hex');
console.log(`t=${t},v1=${v1}`);
