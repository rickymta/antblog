/**
 * TCD-02 — kiểm tài liệu test case trước khi chạy: đúng dạng (zod) và viết tốt (lint).
 * Chạy: npx vitest run articles/test-case-design-giai-phau-test-case
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { lintTestCase } from './lint-test-case';
import { BAD, GOOD } from './tc-accept';
import { testCaseSchema } from './test-case-schema';

const spec = readFileSync(new URL('../../SPEC.md', import.meta.url), 'utf8');
const specCodes = new Set(spec.match(/R-[A-Z]+-\d{2}/g) ?? []);

describe('TCD-02 · schema: test case đúng dạng', () => {
  it.each(GOOD.map((tc) => [tc.id, tc] as const))('%s khớp schema', (_id, tc) => {
    expect(testCaseSchema.safeParse(tc).success).toBe(true);
  });

  it('bản thiếu bước, sai mã ID, không truy vết bị schema từ chối với đúng ba đường dẫn trường', () => {
    const broken = { ...GOOD[0], id: 'TC1', steps: [], traceability: { requirements: [], conditions: [] } };
    const r = testCaseSchema.safeParse(broken);
    expect(r.success).toBe(false);
    const paths = r.success ? [] : r.error.issues.map((i) => i.path.join('.')).sort();
    expect(paths).toEqual(['id', 'steps', 'traceability.requirements']);
  });

  it('dòng dữ liệu mà bước trỏ tới có thật trong testData', () => {
    for (const tc of GOOD) {
      const ids = tc.testData.map((d) => d.id);
      for (const s of tc.steps) if (s.data) expect(ids, tc.id).toContain(s.data);
    }
  });

  it('ID duy nhất trong bộ', () => {
    const ids = [...GOOD, ...BAD.map((b) => b.doc)].map((tc) => tc.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

describe('TCD-02 · lint: test case viết tốt', () => {
  it.each(GOOD.map((tc) => [tc.id, tc] as const))('%s không có cảnh báo nào', (_id, tc) => {
    expect(lintTestCase(tc, specCodes)).toEqual([]);
  });

  it.each(BAD.map((b) => [b.doc.id, b.label, b] as const))('%s (%s) vẫn đúng dạng nhưng bị lint bắt đúng luật', (_id, _label, b) => {
    expect(testCaseSchema.safeParse(b.doc).success).toBe(true); // schema không đủ để bắt lỗi viết
    expect(lintTestCase(b.doc, specCodes).map((f) => f.rule).sort()).toEqual([...b.violates].sort());
  });
});
