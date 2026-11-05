/**
 * TEST-035 · Mô hình kịch bản (scenario model) của use case UC-01 «Khách đặt xe máy»
 * và cách đo scenario coverage theo CTAL-TA v4.0 mục 3.2.3:
 *   coverage = số kịch bản đã thực thi / số kịch bản đã xác định.
 */

export type ScenarioType = 'main' | 'extension' | 'exception';

export interface Scenario {
  id: string;
  type: ScenarioType;
  /** Đường đi trong mô hình: số bước của luồng chính + nhánh rẽ (kiểu «3a»). */
  path: string;
  title: string;
  basis: string[];
  priority: 'cao' | 'trung bình' | 'thấp';
  /** Mức test thực thi kịch bản này trong bài. */
  executedAt: 'api' | 'ui';
}

/** Kịch bản đã xác định từ UC-01 (bước đánh số theo bảng luồng chính trong bài). */
export const UC01_SCENARIOS: readonly Scenario[] = [
  { id: 'KB-01', type: 'main', path: '1→2→3→4→5→6→7→8→9', title: 'Luồng chính: báo giá có GIAM20, đặt chuyến VíXanh, D101 chở tới nơi, thu tiền', basis: ['R-FARE-03', 'R-QUOTE-03', 'R-STATE-02', 'R-PAY-03'], priority: 'cao', executedAt: 'api' },
  { id: 'KB-02', type: 'extension', path: '1→1a→1→2→…→9', title: 'Voucher hết hạn ⇒ khách bỏ voucher, báo giá lại', basis: ['R-VOUCHER-04', 'R-VOUCHER-08'], priority: 'trung bình', executedAt: 'api' },
  { id: 'KB-03', type: 'extension', path: '1→2→3a→1→2→3→…→9', title: 'Báo giá hết hạn trước khi đặt ⇒ báo giá lại', basis: ['R-QUOTE-01', 'R-QUOTE-03'], priority: 'trung bình', executedAt: 'api' },
  { id: 'KB-04', type: 'extension', path: '1→…→5→5a→4→5→…→9', title: 'Tài xế huỷ ⇒ chuyến về tìm tài xế, tài xế khác nhận (vòng lặp)', basis: ['R-STATE-02', 'R-STATE-04', 'R-ACCEPT-05'], priority: 'cao', executedAt: 'api' },
  { id: 'KB-05', type: 'exception', path: '1→2→3→4b', title: 'Không có tài xế nhận trong 5 phút ⇒ hệ thống huỷ, phí 0 đ', basis: ['R-STATE-05', 'R-CANCEL-05'], priority: 'cao', executedAt: 'api' },
  { id: 'KB-06', type: 'exception', path: '1→…→5→5b', title: 'Khách huỷ sau khi có tài xế ⇒ có thể mất phí huỷ', basis: ['R-CANCEL-01', 'R-CANCEL-02', 'R-CANCEL-03'], priority: 'cao', executedAt: 'api' },
  { id: 'KB-07', type: 'extension', path: '1→2→2a→1→2→…→9', title: 'Khách sửa một ô sau khi báo giá ⇒ phải báo giá lại trước khi đặt', basis: ['R-UI-01'], priority: 'thấp', executedAt: 'ui' },
];

/** Scenario coverage (CTAL-TA 3.2.3) — trả cả danh sách kịch bản chưa chạy để báo cáo. */
export function scenarioCoverage(identified: readonly Scenario[], executed: ReadonlySet<string>) {
  const run = identified.filter((s) => executed.has(s.id));
  const missing = identified.filter((s) => !executed.has(s.id)).map((s) => s.id);
  return { executed: run.length, identified: identified.length, percent: (100 * run.length) / identified.length, missing };
}

/** Simple loop coverage của vòng «tài xế huỷ ⇒ tìm lại»: 0, 1, nhiều lần và tối đa (nếu xác định được). */
export const LOOP_ITEMS = ['0 lần', '1 lần', 'nhiều lần', 'tối đa'] as const;
export type LoopItem = (typeof LOOP_ITEMS)[number];
