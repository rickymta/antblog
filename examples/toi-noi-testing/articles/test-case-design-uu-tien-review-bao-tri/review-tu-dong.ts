/**
 * TCD-09 · Phần MÁY KIỂM ĐƯỢC của checklist review test case CL-TC (bài viết mục 4.4).
 * Đây là heuristic để người review khỏi mất công ở lỗi cơ học — không thay individual review:
 * «kết quả mong đợi có ĐÚNG đặc tả không», «dữ liệu có chạm điều kiện không» vẫn cần người đọc.
 */
import { readFileSync } from 'node:fs';
import { CONDITIONS } from './bo-test-case';
import type { WrittenTestCase } from './ban-nhap';

export interface Finding {
  tc: string;
  item: 'CL-TC-01' | 'CL-TC-02' | 'CL-TC-03' | 'CL-TC-04' | 'CL-TC-05' | 'CL-TC-06' | 'CL-TC-07';
  note: string;
}

/** Mọi mã R-… có trong SPEC.md của lab. */
export function specCodes(): Set<string> {
  const spec = readFileSync(new URL('../../SPEC.md', import.meta.url), 'utf8');
  return new Set(spec.match(/R-[A-Z]+-\d{2}/g) ?? []);
}

const CODE_LIKE = /`[^`]+`|\b[A-Z][A-Z_]{3,}\b|\d/; // giá trị cụ thể: mã, tên trường, con số
const IMPLEMENTATION = /MemoryIdempotencyStore|fingerprint|SHA-256|chargeWithRetry|src\/|\bMap\b/;
const HIDDEN_DEP = /ca trước|bước trước|ca kia/i;
const TC_REF = /TC-[A-Z]+-\d{3}/g;
const CLOCK_TIME = /\b\d{1,2}:\d{2}\b|\b\d{2}\/\d{2}\/\d{4}\b/;
const HAS_ZONE = /\+07:00|giờ VN|UTC/;
const REAL_WAIT = /chờ \d+ (giây|phút)/i;

export function reviewMachineCheckable(cases: WrittenTestCase[], codes = specCodes()): Finding[] {
  const findings: Finding[] = [];
  const conditionIds = new Set(CONDITIONS.map((c) => c.id));
  const signatures = new Map<string, string>();

  for (const tc of cases) {
    const text = [...tc.preconditions, ...tc.data, ...tc.steps];

    if (tc.objectives.length !== 1) {
      findings.push({ tc: tc.id, item: 'CL-TC-01', note: `${tc.objectives.length} mục tiêu trong một ca` });
    }
    const vague = tc.expected.filter((e) => !CODE_LIKE.test(e));
    if (vague.length > 0) {
      findings.push({ tc: tc.id, item: 'CL-TC-02', note: `kết quả mong đợi không có giá trị kiểm chứng được: «${vague[0]}»` });
    }
    const refs = text.flatMap((t) => t.match(TC_REF) ?? []).filter((r) => r !== tc.id && !tc.dependsOn.includes(r));
    if (text.some((t) => HIDDEN_DEP.test(t)) || refs.length > 0) {
      findings.push({ tc: tc.id, item: 'CL-TC-03', note: 'dựa vào kết quả ca khác mà không khai báo phụ thuộc' });
    }
    if (text.some((t) => (CLOCK_TIME.test(t) && !HAS_ZONE.test(t)) || REAL_WAIT.test(t))) {
      findings.push({ tc: tc.id, item: 'CL-TC-04', note: 'giờ không kèm múi giờ hoặc chờ thời gian thật' });
    }
    const signature = JSON.stringify([tc.preconditions, tc.data, tc.steps, tc.expected]);
    const twin = signatures.get(signature);
    if (twin) findings.push({ tc: tc.id, item: 'CL-TC-05', note: `trùng tiền điều kiện, dữ liệu, bước và kết quả với ${twin}` });
    else signatures.set(signature, tc.id);
    if ([...tc.steps, ...tc.expected].some((t) => IMPLEMENTATION.test(t))) {
      findings.push({ tc: tc.id, item: 'CL-TC-06', note: 'kiểm cấu trúc bên trong thay vì hành vi quan sát được' });
    }
    const badReq = tc.reqs.filter((r) => !codes.has(r));
    const badCond = tc.covers.filter((c) => !conditionIds.has(c));
    if (tc.reqs.length === 0 || tc.covers.length === 0 || badReq.length > 0 || badCond.length > 0) {
      findings.push({ tc: tc.id, item: 'CL-TC-07', note: 'không truy vết được tới R-… hoặc test condition' });
    }
  }
  return findings;
}
