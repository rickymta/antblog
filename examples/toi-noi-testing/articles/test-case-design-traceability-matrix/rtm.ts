/**
 * TCD-08 — sinh ma trận truy vết (RTM) từ mã nguồn:
 *  - yêu cầu: đọc SPEC.md (mã **R-…** đầu mỗi gạch đầu dòng), lọc theo nhóm trong phạm vi;
 *  - test condition: registry ở dieu-kien.ts;
 *  - test case: đọc tiêu đề it(…) trong các tệp test, lấy mã TC-…, tag @R-… và @TCOND-…;
 *  - kết quả: tuỳ chọn, do reporter (rtm-reporter.ts) đưa vào sau khi chạy;
 *  - defect: registry (giả lập) ở dieu-kien.ts.
 * Hàm thuần, không phụ thuộc vitest — dùng chung cho test, reporter và script in-rtm.ts.
 */
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { DEFECTS, LOAI_TRU, NHOM_TRONG_PHAM_VI, TEST_CONDITIONS, type Defect, type TestCondition } from './dieu-kien';

export const THU_MUC_BAI = path.dirname(fileURLToPath(import.meta.url));
export const SPEC_PATH = path.join(THU_MUC_BAI, '../../SPEC.md');
/** Các tệp chứa test case «sản phẩm» được truy vết (không gồm rtm.test.ts). */
export const TEP_TEST = ['trang-thai.test.ts', 'phi-huy.test.ts'].map((f) => path.join(THU_MUC_BAI, f));

export interface TestCaseTag {
  tieuDe: string;
  tep: string;
  id: string | null;
  spec: string[];
  tcond: string[];
}

export interface RtmInput {
  yeuCau: string[];
  loaiTru: Record<string, string>;
  dieuKien: TestCondition[];
  testCase: TestCaseTag[];
  defect: Defect[];
  /** Tiêu đề test → passed/failed/skipped (nếu có). */
  ketQua?: Map<string, string>;
}

// ------------------------------------------------------------------ đọc nguồn

export function docYeuCau(spec: string, nhom: readonly string[] = NHOM_TRONG_PHAM_VI): string[] {
  const tatCa = [...spec.matchAll(/\*\*(R-[A-Z]+-\d{2}) —/g)].map((m) => m[1]!);
  return [...new Set(tatCa)].filter((m) => nhom.some((n) => m.startsWith(`${n}-`)));
}

/** Lấy tiêu đề của mọi it(…)/test(…) viết bằng chuỗi hằng trong một tệp. */
export function docTieuDeTest(maNguon: string): string[] {
  const re = /\b(?:it|test)(?:\.(?:skip|only|todo|fails))?\(\s*(['"`])((?:\\.|(?!\1).)*)\1/g;
  return [...maNguon.matchAll(re)].map((m) => m[2]!);
}

export function phanTichTag(tieuDe: string, tep: string): TestCaseTag {
  return {
    tieuDe,
    tep,
    id: /\bTC-[A-Z]+-\d{3}\b/.exec(tieuDe)?.[0] ?? null,
    spec: [...tieuDe.matchAll(/@(R-[A-Z]+-\d{2})\b/g)].map((m) => m[1]!),
    tcond: [...tieuDe.matchAll(/@(TCOND-[A-Z]+-\d{2})\b/g)].map((m) => m[1]!),
  };
}

export function docNguon(): RtmInput {
  const spec = readFileSync(SPEC_PATH, 'utf8');
  return {
    yeuCau: docYeuCau(spec),
    loaiTru: LOAI_TRU,
    dieuKien: TEST_CONDITIONS,
    testCase: TEP_TEST.flatMap((f) => docTieuDeTest(readFileSync(f, 'utf8')).map((t) => phanTichTag(t, path.basename(f)))),
    defect: DEFECTS,
  };
}

// ------------------------------------------------------------------ dựng RTM

export interface DongRtm {
  yeuCau: string;
  dieuKien: string[];
  testCase: string[];
  ketQua: string[];
  defect: string[];
}

export interface BaoCaoRtm {
  dong: DongRtm[];
  phamVi: string[];
  phuYeuCau: { daPhu: number; tong: number; phanTram: number };
  phuDieuKien: { daPhu: number; tong: number; phanTram: number };
  yeuCauChuaPhu: string[];
  yeuCauChuaCoDieuKien: string[];
  dieuKienChuaPhu: string[];
  moCoi: { tieuDe: string; lyDo: string }[];
  lechTag: { testCase: string; yeuCau: string }[];
}

const tenTc = (t: TestCaseTag) => t.id ?? `«${t.tieuDe}»`;
const pt = (a: number, b: number) => Math.round((a / b) * 1000) / 10;

/** Yêu cầu mà một test case truy về: tag @R-… trực tiếp ∪ yêu cầu của các @TCOND-…. */
export function yeuCauCuaTestCase(t: TestCaseTag, dk: TestCondition[]): string[] {
  const quaDk = t.tcond.flatMap((id) => dk.find((d) => d.id === id)?.spec ?? []);
  return [...new Set([...t.spec, ...quaDk])].sort();
}

export function dungRtm(x: RtmInput): BaoCaoRtm {
  const phamVi = x.yeuCau.filter((r) => !(r in x.loaiTru));
  const maDk = new Set(x.dieuKien.map((d) => d.id));
  const maYc = new Set(x.yeuCau);

  const dong: DongRtm[] = phamVi.map((r) => {
    const dieuKien = x.dieuKien.filter((d) => d.spec.includes(r)).map((d) => d.id);
    const tcs = x.testCase.filter((t) => yeuCauCuaTestCase(t, x.dieuKien).includes(r));
    const ids = tcs.map(tenTc);
    return {
      yeuCau: r,
      dieuKien,
      testCase: ids,
      ketQua: x.ketQua ? tcs.map((t) => x.ketQua!.get(t.tieuDe) ?? 'chưa chạy') : [],
      defect: x.defect.filter((d) => ids.includes(d.testCase)).map((d) => d.id),
    };
  });

  const dieuKienDaPhu = new Set(x.testCase.flatMap((t) => t.tcond));
  const dieuKienChuaPhu = x.dieuKien.filter((d) => !dieuKienDaPhu.has(d.id)).map((d) => d.id);
  const yeuCauChuaPhu = dong.filter((d) => d.testCase.length === 0).map((d) => d.yeuCau);

  const moCoi: BaoCaoRtm['moCoi'] = [];
  for (const t of x.testCase) {
    if (t.spec.length === 0 && t.tcond.length === 0) moCoi.push({ tieuDe: t.tieuDe, lyDo: 'không có tag @R-… hay @TCOND-…' });
    for (const r of t.spec.filter((r) => !maYc.has(r))) moCoi.push({ tieuDe: t.tieuDe, lyDo: `${r} không có trong test basis` });
    for (const c of t.tcond.filter((c) => !maDk.has(c))) moCoi.push({ tieuDe: t.tieuDe, lyDo: `${c} không có trong registry` });
  }

  // Tag @R-… trực tiếp mà không test condition nào của test case đó truy về ⇒ cần review.
  const lechTag = x.testCase.flatMap((t) => {
    if (t.tcond.length === 0) return [];
    const quaDk = new Set(t.tcond.flatMap((id) => x.dieuKien.find((d) => d.id === id)?.spec ?? []));
    return t.spec.filter((r) => maYc.has(r) && !quaDk.has(r)).map((r) => ({ testCase: tenTc(t), yeuCau: r }));
  });

  const daPhu = phamVi.length - yeuCauChuaPhu.length;
  return {
    dong,
    phamVi,
    phuYeuCau: { daPhu, tong: phamVi.length, phanTram: pt(daPhu, phamVi.length) },
    phuDieuKien: { daPhu: x.dieuKien.length - dieuKienChuaPhu.length, tong: x.dieuKien.length, phanTram: pt(x.dieuKien.length - dieuKienChuaPhu.length, x.dieuKien.length) },
    yeuCauChuaPhu,
    yeuCauChuaCoDieuKien: dong.filter((d) => d.dieuKien.length === 0).map((d) => d.yeuCau),
    dieuKienChuaPhu,
    moCoi,
    lechTag,
  };
}

// ------------------------------------------------------------------ truy vấn

/** Phân tích ảnh hưởng: các yêu cầu đổi ⇒ test condition và test case phải xem lại. */
export function anhHuong(x: RtmInput, yeuCauDoi: string[]) {
  const dieuKien = x.dieuKien.filter((d) => d.spec.some((r) => yeuCauDoi.includes(r))).map((d) => d.id);
  const testCase = x.testCase
    .filter((t) => yeuCauCuaTestCase(t, x.dieuKien).some((r) => yeuCauDoi.includes(r)))
    .map(tenTc);
  return { dieuKien, testCase };
}

/** Truy vết ngược: test case hoặc defect ⇒ test condition ⇒ yêu cầu. */
export function truyNguoc(x: RtmInput, ma: string) {
  const tcId = x.defect.find((d) => d.id === ma)?.testCase ?? ma;
  const t = x.testCase.find((t) => t.id === tcId);
  if (!t) return null;
  return { testCase: tcId, dieuKien: t.tcond, yeuCau: yeuCauCuaTestCase(t, x.dieuKien) };
}

// ------------------------------------------------------------------ in

export function inMarkdown(b: BaoCaoRtm): string {
  const coKetQua = b.dong.some((d) => d.ketQua.length > 0);
  const head = coKetQua
    ? '| Yêu cầu | Test condition | Test case | Kết quả | Defect |\n|---|---|---|---|---|'
    : '| Yêu cầu | Test condition | Test case | Defect |\n|---|---|---|---|';
  const rows = b.dong.map((d) => {
    const dk = d.dieuKien.map((x) => x.replace('TCOND-', '')).join(', ') || '—';
    const tc = d.testCase.map((x) => x.replace(/^TC-/, '')).join(', ') || '**chưa có**';
    const df = d.defect.join(', ') || '—';
    if (!coKetQua) return `| ${d.yeuCau} | ${dk} | ${tc} | ${df} |`;
    const dat = d.ketQua.filter((k) => k === 'passed').length;
    return `| ${d.yeuCau} | ${dk} | ${tc} | ${d.ketQua.length ? `${dat}/${d.ketQua.length} đạt` : '—'} | ${df} |`;
  });
  const vi = (n: number) => n.toLocaleString('vi-VN', { maximumFractionDigits: 1 });
  return [
    head,
    ...rows,
    '',
    `Độ phủ yêu cầu: ${b.phuYeuCau.daPhu}/${b.phuYeuCau.tong} = ${vi(b.phuYeuCau.phanTram)}%`,
    `Độ phủ test condition: ${b.phuDieuKien.daPhu}/${b.phuDieuKien.tong} = ${vi(b.phuDieuKien.phanTram)}%`,
    `Yêu cầu chưa có test case: ${b.yeuCauChuaPhu.join(', ') || 'không'}`,
    `Test condition chưa có test case: ${b.dieuKienChuaPhu.join(', ') || 'không'}`,
    `Test case mồ côi: ${b.moCoi.map((o) => `«${o.tieuDe.slice(0, 40)}…» (${o.lyDo})`).join('; ') || 'không'}`,
    `Tag lệch registry: ${b.lechTag.map((l) => `${l.testCase} → ${l.yeuCau}`).join('; ') || 'không'}`,
  ].join('\n');
}
