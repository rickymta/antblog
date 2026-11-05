/**
 * TEST-033 — Decision Table Testing cho phí huỷ chuyến (SPEC R-CANCEL-01…06, R-STATE-02/04/05).
 * Chạy: npx vitest run articles/decision-table-testing --reporter=verbose
 */
import { describe, expect, it } from 'vitest';
import { customerCancellationFee, type CancellationFee } from '../../src/domain/cancel';
import { DomainError } from '../../src/domain/errors';
import { vnTime } from '../../src/domain/time';
import { checksum, fullTable, infeasibleReason, matches, nhan, RULES, type Combo, type KetQua, type Rule } from './bang-quyet-dinh';
import { buggyCustomerCancellationFee } from './buggy-cancel-fee';
import { chayToHop, dauVaoHamPhi, type QuanSat } from './chay-to-hop';

const feasible = fullTable().filter((x) => infeasibleReason(x) === null);
const ruleOf = (x: Combo): Rule => {
  const found = RULES.filter((r) => matches(r, x));
  if (found.length !== 1) throw new Error(`${nhan(x)} khớp ${found.length} quy tắc`);
  return found[0]!;
};

/** Quy đổi điều quan sát được về cùng dạng «hành động» của bảng để so sánh. */
function ketQuaQuanSat(q: QuanSat): KetQua {
  if (q.loi !== null) return { loai: 'tu_choi', ma: q.loi as 'INVALID_TRANSITION' };
  if (q.sau === 'searching' && q.truoc !== 'searching') return { loai: 've_searching' };
  if (q.sau === 'cancelled') {
    return q.lyDoHuy === 'no_driver'
      ? { loai: 'huy', phi: q.phi, feeReason: q.feeReason, lyDoHuy: 'no_driver' }
      : { loai: 'huy', phi: q.phi, feeReason: q.feeReason };
  }
  return { loai: 'tu_choi', ma: 'INVALID_TRANSITION' }; // không đổi gì mà cũng không báo lỗi ⇒ sẽ lệch bảng
}

async function kiemTheoBang(x: Combo, rule: Rule): Promise<void> {
  const q = await chayToHop(x);
  expect(ketQuaQuanSat(q)).toEqual(rule.then);
  if (rule.then.loai === 'tu_choi') {
    expect(q.sau).toBe(q.truoc); // R-STATE-03: chuyến giữ nguyên
    expect(q.soDongNhatKy.sau).toBe(q.soDongNhatKy.truoc); // không ghi trip_events
  }
  if (rule.then.loai === 've_searching') expect(q.taiXe).toBeNull(); // R-STATE-04: bỏ gán tài xế
}

describe('TEST-033 · kiểm chính bảng quyết định (trước khi chạy test nào)', () => {
  it('bảng đầy đủ 96 cột = 40 khả thi + 56 không khả thi', () => {
    expect(fullTable()).toHaveLength(96);
    expect(feasible).toHaveLength(40);
    const demLyDo = new Map<string, number>();
    for (const x of fullTable()) {
      const r = infeasibleReason(x);
      if (r !== null) demLyDo.set(r, (demLyDo.get(r) ?? 0) + 1);
    }
    expect([...demLyDo.values()]).toEqual([24, 12, 2, 18]); // theo thứ tự xét trong infeasibleReason
  });

  it('đủ và không chồng lấn: mỗi cột khả thi khớp ĐÚNG MỘT quy tắc của bảng rút gọn', () => {
    const sai = feasible.filter((x) => RULES.filter((r) => matches(r, x)).length !== 1).map(nhan);
    expect(sai).toEqual([]);
  });

  it('không có quy tắc thừa: mỗi quy tắc khớp ≥ 1 cột khả thi; bộ dữ liệu đại diện khả thi và đúng quy tắc', () => {
    for (const r of RULES) {
      expect(feasible.some((x) => matches(r, x)), r.id).toBe(true);
      expect(infeasibleReason(r.dai), r.id).toBeNull();
      expect(ruleOf(r.dai).id).toBe(r.id);
    }
  });

  it('checksum = 48 = 40 cột khả thi + 8 cột không khả thi lọt vào các «–»', () => {
    expect(checksum()).toBe(48);
    const lot = fullTable().filter((x) => infeasibleReason(x) !== null && RULES.some((r) => matches(r, x)));
    expect(lot).toHaveLength(8);
    expect(new Set(lot.map((x) => infeasibleReason(x)))).toEqual(new Set(['trễ nghĩa là đã qua ≥ ETA + 10 ≥ 11 phút ⇒ chắc chắn đủ 2 phút']));
  });
});

describe('TEST-033 · 100% decision table coverage của bảng rút gọn (13 ca, tầng use case)', () => {
  it.each(RULES.map((r) => [r.id, nhan(r.dai), r] as const))('%s · %s', async (_id, _nhan, r) => {
    await kiemTheoBang(r.dai, r);
  });
});

describe('TEST-033 · 100% coverage của bảng đầy đủ — 40 cột khả thi (rủi ro tiền: không rút gọn)', () => {
  it.each(feasible.map((x) => [nhan(x), x] as const))('%s', async (_nhan, x) => {
    await kiemTheoBang(x, ruleOf(x));
  });
});

describe('TEST-033 · kết hợp với BVA (TEST-032) ở mốc thời gian — bảng quyết định không tự chọn giá trị biên', () => {
  const acceptedAt = vnTime(2026, 11, 4, 9, 0);
  const at = (status: 'accepted' | 'arriving', ms: number): CancellationFee =>
    customerCancellationFee({ status, vehicle: 'bike', acceptedAt, etaMinutes: 5, now: new Date(acceptedAt.getTime() + ms) });

  it('C3 ở biên R-CANCEL-02: 1:59,999 ⇒ grace_period; 2:00 ⇒ charged', () => {
    expect(at('accepted', 119_999)).toEqual({ fee: 0, reason: 'grace_period' });
    expect(at('accepted', 120_000)).toEqual({ fee: 10_000, reason: 'charged' });
  });

  it('C4 ở biên R-CANCEL-04 (ETA 5): đúng 15:00 ⇒ charged; 15:00,001 ⇒ driver_late', () => {
    expect(at('arriving', 15 * 60_000)).toEqual({ fee: 10_000, reason: 'charged' });
    expect(at('arriving', 15 * 60_000 + 1)).toEqual({ fee: 0, reason: 'driver_late' });
  });
});

describe('TEST-033 · bảng quyết định bắt defect «miễn phí 2 phút cả ở arriving»', () => {
  type Fn = (i: ReturnType<typeof dauVaoHamPhi>) => CancellationFee;
  const ketQuaHam = (fn: Fn, x: Combo): KetQua => {
    try {
      const { fee, reason } = fn(dauVaoHamPhi(x));
      return { loai: 'huy', phi: fee, feeReason: reason };
    } catch (e) {
      if (e instanceof DomainError) return { loai: 'tu_choi', ma: e.code as 'INVALID_TRANSITION' };
      throw e;
    }
  };
  const khachRules = RULES.filter((r) => r.when.nguoiHuy === 'khach');
  const truot = (fn: Fn, cot: Combo[]) => cot.filter((x) => JSON.stringify(ketQuaHam(fn, x)) !== JSON.stringify(ruleOf(x).then));

  it('bản đúng của src/ đạt mọi cột của khách', () => {
    expect(truot(customerCancellationFee, khachRules.map((r) => r.dai))).toEqual([]);
  });

  it('bộ 13 ca: bản lỗi trượt đúng một cột — DT-06 (arriving, 1 phút, xe máy)', () => {
    const fail = khachRules.filter((r) => truot(buggyCustomerCancellationFee, [r.dai]).length > 0).map((r) => r.id);
    expect(fail).toEqual(['DT-06']);
  });

  it('bộ 40 cột: bản lỗi trượt 2 cột (arriving, chưa đủ 2 phút, xe máy và ô tô)', () => {
    const cot = feasible.filter((x) => x.nguoiHuy === 'khach');
    expect(truot(buggyCustomerCancellationFee, cot).map(nhan)).toEqual([
      'khach/arriving/≥2′:F/trễ:F/bike',
      'khach/arriving/≥2′:F/trễ:F/car',
    ]);
  });

  it('nếu chọn đại diện «–» của DT-06 là ≥2′ = T: vẫn 100% coverage bảng rút gọn nhưng LỌT defect', () => {
    const dai: Combo[] = khachRules.map((r) => (r.id === 'DT-06' ? { ...r.dai, du2Phut: true } : r.dai));
    expect(truot(buggyCustomerCancellationFee, dai)).toEqual([]);
  });

  it('bộ «mỗi trạng thái một ca» (searching, accepted 5′, arriving 5′, picked_up) cũng LỌT defect', () => {
    const ngayThat: Combo[] = [
      { nguoiHuy: 'khach', trangThai: 'searching', du2Phut: false, taiXeTre: false, xe: 'bike' },
      { nguoiHuy: 'khach', trangThai: 'accepted', du2Phut: true, taiXeTre: false, xe: 'bike' },
      { nguoiHuy: 'khach', trangThai: 'arriving', du2Phut: true, taiXeTre: false, xe: 'bike' },
      { nguoiHuy: 'khach', trangThai: 'khac', du2Phut: true, taiXeTre: false, xe: 'bike' },
    ];
    expect(truot(buggyCustomerCancellationFee, ngayThat)).toEqual([]);
  });
});
