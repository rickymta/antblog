/**
 * TCD-10 · Sinh ma trận truy vết (RTM) và số liệu coverage từ thiết kế + kết quả chạy thật.
 * Xuôi: yêu cầu → test condition → test case → kết quả. Ngược: mỗi test case trỏ về yêu cầu nào.
 */
import { CONDITIONS, IN_SCOPE, TEST_CASES, type BookTestCase } from './thiet-ke';

export type Outcome = 'pass' | 'fail';
export interface Execution {
  tc: string;
  name: string;
  outcome: Outcome;
}

export interface RtmRow {
  req: string;
  conditions: string[];
  testCases: string[];
  executions: number;
  passed: number;
  failed: number;
  defects: string[];
  status: 'Đạt' | 'Có defect mở' | 'Chưa chạy' | 'Chưa có test case';
}

export interface RtmSummary {
  rows: RtmRow[];
  inScope: number;
  inScopeCovered: number;
  conditions: number;
  conditionsCovered: number;
  logicalTestCases: number;
  plannedExecutions: number;
  executions: number;
  passed: number;
  failed: number;
  orphanTestCases: string[];
}

export function buildRtm(results: Execution[], cases: BookTestCase[] = TEST_CASES): RtmSummary {
  const reqs = [...new Set([...IN_SCOPE, ...cases.flatMap((c) => c.reqs)])];
  const rows: RtmRow[] = reqs.map((req) => {
    const conditions = CONDITIONS.filter((c) => c.req === req).map((c) => c.id);
    const testCases = cases.filter((c) => c.reqs.includes(req)).map((c) => c.id);
    const runs = results.filter((r) => testCases.includes(r.tc));
    const failed = runs.filter((r) => r.outcome === 'fail');
    const defects = [...new Set(failed.map((r) => cases.find((c) => c.id === r.tc)?.knownDefect ?? 'chưa phân loại'))];
    const status: RtmRow['status'] =
      testCases.length === 0 ? 'Chưa có test case' : runs.length === 0 ? 'Chưa chạy' : failed.length > 0 ? 'Có defect mở' : 'Đạt';
    return { req, conditions, testCases, executions: runs.length, passed: runs.length - failed.length, failed: failed.length, defects, status };
  });
  const coveredConditions = new Set(cases.filter((c) => results.some((r) => r.tc === c.id)).flatMap((c) => c.covers));
  const known = new Set(CONDITIONS.map((c) => c.id));
  return {
    rows,
    inScope: IN_SCOPE.length,
    inScopeCovered: rows.filter((r) => (IN_SCOPE as readonly string[]).includes(r.req) && r.executions > 0).length,
    conditions: CONDITIONS.length,
    conditionsCovered: [...coveredConditions].filter((c) => known.has(c)).length,
    logicalTestCases: cases.length,
    plannedExecutions: cases.reduce((s, c) => s + c.concrete, 0),
    executions: results.length,
    passed: results.filter((r) => r.outcome === 'pass').length,
    failed: results.filter((r) => r.outcome === 'fail').length,
    orphanTestCases: cases.filter((c) => c.reqs.length === 0 || c.covers.some((x) => !known.has(x))).map((c) => c.id),
  };
}

export function renderMarkdown(s: RtmSummary, schedule: string[]): string {
  const pct = (a: number, b: number) => `${((100 * a) / b).toFixed(1).replace('.', ',')}%`;
  const lines = [
    '# TCD-10 · RTM «đặt chuyến từ báo giá» (sinh tự động)',
    '',
    `Yêu cầu trong phạm vi có test case đã chạy: ${s.inScopeCovered}/${s.inScope} (${pct(s.inScopeCovered, s.inScope)})`,
    `Test condition có test case đã chạy: ${s.conditionsCovered}/${s.conditions} (${pct(s.conditionsCovered, s.conditions)})`,
    `Test case logic: ${s.logicalTestCases} · test case cụ thể dự kiến ${s.plannedExecutions} · đã chạy ${s.executions} · đạt ${s.passed} · không đạt ${s.failed}`,
    `Test case mồ côi (không truy vết): ${s.orphanTestCases.length === 0 ? 'không có' : s.orphanTestCases.join(', ')}`,
    '',
    '| Yêu cầu | Test condition | Test case | Chạy | Đạt | Không đạt | Defect | Trạng thái |',
    '|---|---|---|---:|---:|---:|---|---|',
    ...s.rows.map((r) =>
      `| ${r.req} | ${r.conditions.map((c) => c.replace('TCOND-BOOK-', '')).join(', ') || '—'} | ${r.testCases.map((t) => t.replace('TC-BOOK-', '')).join(', ')} | ${r.executions} | ${r.passed} | ${r.failed} | ${r.defects.join(', ') || '—'} | ${r.status} |`),
    '',
    `Thứ tự chạy (risk-based, đã sửa phụ thuộc): ${schedule.map((t) => t.replace('TC-BOOK-', '')).join(' → ')}`,
    '',
  ];
  return lines.join('\n');
}
