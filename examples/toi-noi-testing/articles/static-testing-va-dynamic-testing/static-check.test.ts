/**
 * STATIC TESTING bằng công cụ (tự viết, tối giản): đọc mã nguồn như văn bản, không thực thi.
 * In phát hiện ra console để đối chiếu với bài viết:
 *   npx vitest run articles/static-testing-va-dynamic-testing/static-check.test.ts
 */
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { analyzeFareSource } from './static-check';

const read = (relative: string) => readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf8');

describe('Static analysis tự viết trên mã tính cước', () => {
  it('bản lỗi: chỉ ra ĐÚNG dòng chứa defect D1 (S2) và số viết cứng D3 (S1), không chạy hàm nào', () => {
    const findings = analyzeFareSource(read('./buggy-fare.ts'));
    for (const f of findings) console.log(`buggy-fare.ts:${f.line} [${f.rule}] ${f.message}\n    ${f.text}`);
    expect(findings.map((f) => `${f.rule}@${f.line}`)).toEqual(['S1@29', 'S2@29']);
  });

  it('bản đúng src/domain/fare.ts: không có phát hiện nào', () => {
    expect(analyzeFareSource(read('../../src/domain/fare.ts'))).toEqual([]);
  });
});
