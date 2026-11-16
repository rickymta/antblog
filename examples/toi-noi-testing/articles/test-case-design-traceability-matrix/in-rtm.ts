/**
 * TCD-08 — in RTM tĩnh (không chạy test) và trả lời truy vấn.
 *   npx tsx articles/test-case-design-traceability-matrix/in-rtm.ts
 *   npx tsx articles/test-case-design-traceability-matrix/in-rtm.ts --impact R-CANCEL-02
 *   npx tsx articles/test-case-design-traceability-matrix/in-rtm.ts --back DEF-SIM-001
 */
import { anhHuong, docNguon, dungRtm, inMarkdown, truyNguoc } from './rtm';

const args = process.argv.slice(2);
const nguon = docNguon();
const i = args.indexOf('--impact');
const j = args.indexOf('--back');

if (i >= 0) {
  const doi = args.slice(i + 1).filter((a) => a.startsWith('R-'));
  console.log(`Ảnh hưởng khi đổi ${doi.join(', ')}:`, JSON.stringify(anhHuong(nguon, doi), null, 2));
} else if (j >= 0) {
  console.log(`Truy vết ngược ${args[j + 1]}:`, JSON.stringify(truyNguoc(nguon, args[j + 1] ?? ''), null, 2));
} else {
  console.log(inMarkdown(dungRtm(nguon)));
}
