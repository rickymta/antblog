/**
 * TCD-02 — bộ lint nhỏ cho tài liệu test case: bắt các lỗi viết hay gặp mà schema không bắt được.
 * Mỗi luật là một heuristic có chủ đích đơn giản; nó không thay review của con người (TCD-09).
 */
import type { TestCaseDoc } from './test-case-schema';

export type LintRuleId = 'TC-L01' | 'TC-L02' | 'TC-L03' | 'TC-L04' | 'TC-L05' | 'TC-L06' | 'TC-L07';

export interface LintFinding {
  rule: LintRuleId;
  field: keyof TestCaseDoc;
  message: string;
}

/** Cụm từ không kiểm chứng được: hai người chạy có thể kết luận khác nhau. */
const VAGUE = [
  /hoạt động (đúng|bình thường|tốt|ổn định)/i,
  /như (mong đợi|mong muốn|thiết kế)/i,
  /đúng (yêu cầu|đặc tả|nghiệp vụ)/i,
  /không (có )?lỗi/i,
  /hiển thị (đúng|hợp lệ)/i,
  /^\s*(thành công|ok|pass|đạt)\.?\s*$/i,
];

/** Một kết quả mong đợi quan sát được phải neo vào ít nhất một giá trị: số, `mã`, hoặc MÃ_LỖI. */
const OBSERVABLE = /\d|`[^`]+`|\b[A-Z][A-Z0-9]*_[A-Z0-9_]+\b/;

/** Lời không tất định trong kết quả mong đợi. */
const NONDETERMINISTIC = /(^|[\s(])(có thể|tuỳ|tùy|khoảng|ngẫu nhiên|thường thì|hoặc là)(?=[\s,.)]|$)/i;

/** Động từ mục tiêu: xuất hiện từ hai lần trong objective thường là hai mục tiêu ghép lại. */
const OBJECTIVE_VERB = /(^|\s)(kiểm tra|kiểm|xác nhận|đảm bảo|chứng minh)(?=\s)/gi;

export function lintTestCase(tc: TestCaseDoc, specCodes: ReadonlySet<string>): LintFinding[] {
  const out: LintFinding[] = [];

  for (const e of tc.expectedResults) {
    if (VAGUE.some((re) => re.test(e))) {
      out.push({ rule: 'TC-L01', field: 'expectedResults', message: `kết quả mong đợi mơ hồ: «${e}»` });
    }
    if (!OBSERVABLE.test(e)) {
      out.push({ rule: 'TC-L02', field: 'expectedResults', message: `kết quả mong đợi không neo vào giá trị quan sát được: «${e}»` });
    }
    if (NONDETERMINISTIC.test(e)) {
      out.push({ rule: 'TC-L06', field: 'expectedResults', message: `kết quả mong đợi không tất định: «${e}»` });
    }
  }

  if (tc.preconditions.length === 0) {
    out.push({ rule: 'TC-L03', field: 'preconditions', message: 'thiếu tiền điều kiện: không biết hệ thống phải ở trạng thái nào trước bước 1' });
  }

  const verbs = tc.objective.match(OBJECTIVE_VERB) ?? [];
  if (verbs.length >= 2 || tc.objective.includes(';')) {
    out.push({ rule: 'TC-L04', field: 'objective', message: `nhiều mục tiêu trong một test case: «${tc.objective}»` });
  }

  const otherTc = [...tc.preconditions, ...tc.steps.map((s) => s.action)]
    .flatMap((t) => t.match(/TC-[A-Z]+-\d{3}/g) ?? [])
    .filter((id) => id !== tc.id);
  if (otherTc.length > 0) {
    out.push({ rule: 'TC-L05', field: 'preconditions', message: `phụ thuộc test case khác (${[...new Set(otherTc)].join(', ')}): không chạy độc lập được` });
  }

  const unknown = tc.traceability.requirements.filter((code) => !specCodes.has(code));
  if (unknown.length > 0) {
    out.push({ rule: 'TC-L07', field: 'traceability', message: `truy vết tới mã không có trong test basis: ${unknown.join(', ')}` });
  }

  return out;
}
