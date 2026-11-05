/**
 * TEST-045 — voucher thử và ba bộ test thiết kế ở mục 4 của bài.
 * Voucher HN11 (giả định, dựng bằng `voucher()` của seed): chỉ đặt xe ở Hà Nội, 01–30/11/2026,
 * giảm 10 %, trần 15.000 đ, đơn tối thiểu 40.000 đ.
 */
import { vnTime } from '../../src/domain/time';
import { voucher } from '../../src/infra/seed';
import type { DuLieuTest } from './dieu-kien-voucher';

export const HN11 = voucher({
  code: 'HN11',
  percent: 10,
  maxDiscount: 15_000,
  minAmount: 40_000,
  kinds: ['ride'],
  cityCodes: ['HN'],
  validFrom: '2026-11-01',
  validTo: '2026-11-30',
});

const trongThang = vnTime(2026, 11, 4, 9, 0);

/** Bộ A — đạt condition coverage, KHÔNG đạt decision coverage (bài tập của curriculum). */
export const BO_A: DuLieuTest[] = [
  { id: 'A1', at: vnTime(2026, 12, 5, 9, 0), kind: 'ride', cityCode: 'HN', amountDong: 50_000 }, // F T T T
  { id: 'A2', at: trongThang, kind: 'parcel', cityCode: 'SG', amountDong: 20_000 }, // T F F F
];

/** Bộ B — condition coverage + decision coverage chỉ với 2 test. */
export const BO_B: DuLieuTest[] = [
  { id: 'B1', at: trongThang, kind: 'ride', cityCode: 'HN', amountDong: 50_000 }, // T T T T
  { id: 'B2', at: vnTime(2026, 10, 20, 9, 0), kind: 'parcel', cityCode: 'SG', amountDong: 20_000 }, // F F F F
];

/** Bộ C — mỗi điều kiện được lật riêng so với B1 (N + 1 = 5 test, cầu nối sang MC/DC). */
export const BO_C: DuLieuTest[] = [
  { id: 'C0', at: trongThang, kind: 'ride', cityCode: 'HN', amountDong: 50_000 }, // T T T T
  { id: 'C1', at: vnTime(2026, 12, 1, 9, 0), kind: 'ride', cityCode: 'HN', amountDong: 50_000 }, // F T T T
  { id: 'C2', at: trongThang, kind: 'parcel', cityCode: 'HN', amountDong: 50_000 }, // T F T T
  { id: 'C3', at: trongThang, kind: 'ride', cityCode: 'SG', amountDong: 50_000 }, // T T F T
  { id: 'C4', at: trongThang, kind: 'ride', cityCode: 'HN', amountDong: 39_999 }, // T T T F
];
