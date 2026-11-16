#!/usr/bin/env node
// Chụp ảnh DBStudio cho series Cassandra.
//   node shoot.mjs              chụp mọi ảnh của mọi phần
//   node shoot.mjs cs-006       chỉ chụp ảnh có id bắt đầu bằng "cs-006"
// Ảnh WebP được ghi vào apps/web/public/images/articles/cassandra/<id>.webp
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { launch } from './lib.mjs';
import p01 from './p01.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.resolve(here, '../../../apps/web/public/images/articles/cassandra');
const filter = process.argv[2] ?? '';
const shots = [...p01].filter((s) => s.id.startsWith(filter));

const browser = await launch();
let failed = 0;
for (const s of shots) {
  const file = path.join(OUT, `${s.id}.webp`);
  try {
    await s.run(browser, file);
    console.log(`OK   ${s.id}`);
  } catch (e) {
    failed++;
    console.log(`FAIL ${s.id}: ${String(e.message).split('\n')[0]}`);
  } finally {
    for (const ctx of browser.contexts()) await ctx.close();
  }
}
await browser.close();
console.log(`\n${shots.length - failed}/${shots.length} ảnh`);
process.exit(failed ? 1 : 0);
