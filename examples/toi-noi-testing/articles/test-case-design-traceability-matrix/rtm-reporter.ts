/**
 * TCD-08 — reporter vitest tự viết: sau khi chạy, ghép KẾT QUẢ thật của từng test vào RTM
 * rồi in ra và ghi reports/rtm-tcd08.md (thư mục reports/ không commit).
 *   npx vitest run articles/test-case-design-traceability-matrix \
 *     --reporter=default --reporter=./articles/test-case-design-traceability-matrix/rtm-reporter.ts
 */
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import type { Reporter, TestModule } from 'vitest/node';
import { THU_MUC_BAI, docNguon, dungRtm, inMarkdown } from './rtm';

export default class RtmReporter implements Reporter {
  onTestRunEnd(testModules: ReadonlyArray<TestModule>): void {
    const ketQua = new Map<string, string>();
    for (const m of testModules) {
      for (const t of m.children.allTests()) ketQua.set(t.name, t.result().state);
    }
    const md = inMarkdown(dungRtm({ ...docNguon(), ketQua }));
    const out = path.join(THU_MUC_BAI, '../../reports/rtm-tcd08.md');
    mkdirSync(path.dirname(out), { recursive: true });
    writeFileSync(out, `# RTM — R-STATE, R-CANCEL (sinh tự động)\n\n${md}\n`);
    console.log(`\n${md}\n\nĐã ghi ${path.relative(process.cwd(), out)}`);
  }
}
