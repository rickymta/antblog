/**
 * TCD-04 · phần 1 — oracle cước độc lập: kiểm chính oracle, đối chiếu với hệ thống qua bảng và qua mẫu ngẫu nhiên,
 * bẫy số thực, và hai anti-pattern: chép kết quả từ hệ thống, assert quá lỏng / quá chặt.
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { calculateFare, type FareInput } from '../../src/domain/fare';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { BANG_CUOC, type DongCuoc } from './bang-cuoc';
import { buggyCalculateFare } from './buggy-fare';
import { LoiNghiepVu, VOUCHER_SPEC, oracleCuoc, phanMuoi, soChiTiet, type DauVaoCuoc } from './oracle-cuoc';
import { PhanSo } from './phan-so';

const tongOracle = (v: DauVaoCuoc): number | { code: string } => {
  try { return Number(oracleCuoc(v).total.tu); } catch (e) { if (e instanceof LoiNghiepVu) return { code: e.code }; throw e; }
};

/** Đầu vào của hàm miền calculateFare ứng với một dòng (voucher lấy từ SPEC, cùng tham số). */
const sangFareInput = (v: DauVaoCuoc, parcelSurcharge = 0): FareInput => ({
  vehicle: v.xe, distanceKm: v.km, peak: v.caoDiem, raining: v.mua, parcelSurcharge,
  voucher: v.voucher ? { code: v.voucher, percent: VOUCHER_SPEC[v.voucher]!.percent, maxDiscount: VOUCHER_SPEC[v.voucher]!.tran, minAmount: VOUCHER_SPEC[v.voucher]!.toiThieu } : null,
});

/** Bộ sinh số giả ngẫu nhiên có hạt giống — chạy lại cho đúng cùng dãy (mulberry32). */
function rng(seed: number) {
  return () => {
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

describe('TCD-04 · 1. kiểm chính oracle bằng các ví dụ có sẵn trong SPEC', () => {
  it('R-FARE-09: 45.300 → 56.625 → 64.625 → −12.925 → 51.700 → 52.000 đ', () => {
    const o = oracleCuoc({ xe: 'bike', km: 7.4, caoDiem: true, mua: true, voucher: 'GIAM20' });
    expect([o.base, o.afterPeak, o.beforeDiscount, o.discount, o.afterDiscount, o.total].map(String))
      .toEqual(['45.300', '56.625', '64.625', '12.925', '51.700', '52.000']);
  });

  it('R-FARE-02: 0,45 ⇒ 0,5; 0,44 ⇒ 0,4; 7,35 ⇒ 7,4; 100,04 hợp lệ; 100,05 ⇒ 100,1 không hợp lệ', () => {
    expect([0.45, 0.44, 7.35, 100.04, 100.05].map((x) => Number(phanMuoi(x)))).toEqual([5, 4, 74, 1000, 1001]);
  });

  it('R-PARCEL-03: 5,1 ⇒ 2.000; 6,0 ⇒ 2.000; 6,1 ⇒ 4.000; 30 ⇒ 50.000 đ', () => {
    const phuPhi = (kg: number) => Number(oracleCuoc({ xe: 'car', km: 3, kg, caoDiem: false, mua: false }).parcelSurcharge.tu);
    expect([5, 5.1, 6, 6.1, 30].map(phuPhi)).toEqual([0, 2_000, 2_000, 4_000, 50_000]);
  });

  it('R-FARE-08 bằng phân số: 51.700 ⇒ 52.000; 52.000 giữ nguyên; 52.001 ⇒ 53.000', () => {
    const lenNghin = (x: number) => Number(PhanSo.of(x, 1000).tran() * 1000n);
    expect([51_700, 52_000, 52_001].map(lenNghin)).toEqual([52_000, 52_000, 53_000]);
  });

  it('bảng 16 dòng: tổng tính tay khớp oracle (kiểm chéo hai nguồn trước khi dùng làm kết quả mong đợi)', () => {
    expect(BANG_CUOC).toHaveLength(16);
    for (const d of BANG_CUOC) expect(tongOracle(d), d.id).toEqual(d.tongTinhTay);
  });
});

describe('TCD-04 · 2. oracle so với hệ thống qua POST /api/quotes — mọi bước của R-FARE-10', () => {
  it.each(BANG_CUOC.map((d) => [d.id, d] as const))('%s', async (_id, d: DongCuoc) => {
    const clock = new FixedClock(d.caoDiem ? vnTime(2026, 11, 4, 11, 30) : vnTime(2026, 11, 4, 9, 0));
    const system = createMemorySystem({ clock, rainingCities: d.mua ? [d.thanhPho] : [] });
    const app = createApp(system.deps, { logErrors: false });
    const body = {
      kind: d.kg === undefined ? 'ride' : 'parcel', cityCode: d.thanhPho, vehicle: d.xe, distanceKm: d.km,
      ...(d.kg === undefined ? {} : { parcelWeightKg: d.kg }), ...(d.voucher ? { voucherCode: d.voucher } : {}),
    };
    const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send(body);

    if (typeof d.tongTinhTay !== 'number') {
      expect(res.status).toBe(422);
      expect(res.body.error.code).toBe(d.tongTinhTay.code);
      return;
    }
    expect(res.status).toBe(200);
    expect(soChiTiet(res.body.quote.fare, oracleCuoc(d))).toEqual([]); // 12 trường, so chính xác tuyệt đối
    expect(res.body.quote).toMatchObject({ total: d.tongTinhTay, discount: res.body.quote.fare.discount, peak: d.caoDiem, raining: d.mua });
  });
});

describe('TCD-04 · 3. oracle trên mẫu ngẫu nhiên và bẫy số thực', () => {
  const sinhMau = (n: number, seed: number): DauVaoCuoc[] => {
    const r = rng(seed);
    return Array.from({ length: n }, () => ({
      xe: r() < 0.5 ? 'bike' : 'car',
      km: Math.round((0.5 + r() * 99.5) * 100) / 100, // hai chữ số thập phân ⇒ có làm tròn
      caoDiem: r() < 0.5, mua: r() < 0.5,
      voucher: r() < 0.5 ? 'GIAM20' : undefined,
    }) as DauVaoCuoc);
  };

  it('500 đầu vào ngẫu nhiên (hạt giống 20261116): calculateFare khớp oracle ở mọi trường', () => {
    const lech = sinhMau(500, 20261116).flatMap((v) => soChiTiet(calculateFare(sangFareInput(v)) as never, oracleCuoc(v)).map((l) => `${JSON.stringify(v)} ${l}`));
    expect(lech).toEqual([]);
  });

  /** «Oracle» viết vội bằng số thực — đúng công thức, sai số học. */
  const oracleFloat = (v: DauVaoCuoc) => {
    const g: [number, number, number] = v.xe === 'bike' ? [12_000, 4_500, 15_000] : [25_000, 11_000, 30_000];
    let x = g[0] + g[1] * v.km;
    if (v.caoDiem) x *= 1.25;
    return Math.ceil(Math.max(x, g[2]) / 1000) * 1000;
  };

  it('ô tô 18,6 km giờ cao điểm: oracle số thực ra 288.000 đ, đúng là 287.000 đ (hệ thống cũng 287.000)', () => {
    const v: DauVaoCuoc = { xe: 'car', km: 18.6, caoDiem: true, mua: false };
    expect(25_000 + 11_000 * 18.6).not.toBe(229_600); // 229600.00000000003
    expect(oracleFloat(v)).toBe(288_000);
    expect(tongOracle(v)).toBe(287_000);
    expect(calculateFare(sangFareInput(v)).total).toBe(287_000);
  });

  it('quét 0,5…100 km × 2 loại xe × cao điểm có/không (3.984 tổ hợp): oracle số thực sai 12 tổ hợp', () => {
    let sai = 0;
    for (let t = 5; t <= 1000; t++) for (const xe of ['bike', 'car'] as const) for (const caoDiem of [false, true]) {
      const v: DauVaoCuoc = { xe, km: t / 10, caoDiem, mua: false };
      if (oracleFloat(v) !== tongOracle(v)) sai++;
    }
    expect(sai).toBe(12);
  });
});

describe('TCD-04 · 4. anti-pattern: «chạy hệ thống rồi chép kết quả vào expected»', () => {
  const dongGiaTri = BANG_CUOC.filter((d) => typeof d.tongTinhTay === 'number');
  const chay = (f: typeof calculateFare, d: DongCuoc) => f(sangFareInput(d, d.kg === undefined ? 0 : Number(oracleCuoc(d).parcelSurcharge.tu)));

  it('expected chép từ bản lỗi: bản lỗi «đạt» 100% bảng của chính nó', () => {
    const chepTuBanLoi = Object.fromEntries(dongGiaTri.map((d) => [d.id, chay(buggyCalculateFare, d).total]));
    const dat = dongGiaTri.filter((d) => chay(buggyCalculateFare, d).total === chepTuBanLoi[d.id]);
    expect(dat).toHaveLength(14);
  });

  it('expected từ oracle độc lập: bản lỗi trượt đúng dòng có cao điểm + mưa (OR-01: 54.000 ≠ 52.000)', () => {
    const truot = dongGiaTri.filter((d) => chay(buggyCalculateFare, d).total !== d.tongTinhTay);
    expect(truot.map((d) => [d.id, chay(buggyCalculateFare, d).total])).toEqual([['OR-01', 54_000]]);
  });

  it('500 mẫu ngẫu nhiên: oracle bắt bản lỗi ở đúng mọi mẫu có cao điểm + mưa, không báo nhầm mẫu nào khác', () => {
    const r = rng(7);
    let caoDiemVaMua = 0;
    let bat = 0;
    for (let i = 0; i < 500; i++) {
      const v: DauVaoCuoc = { xe: r() < 0.5 ? 'bike' : 'car', km: Math.round((0.5 + r() * 99.5) * 10) / 10, caoDiem: r() < 0.5, mua: r() < 0.5 };
      if (v.caoDiem && v.mua) caoDiemVaMua++;
      if (soChiTiet(buggyCalculateFare(sangFareInput(v)) as never, oracleCuoc(v)).length > 0) bat++;
    }
    expect(caoDiemVaMua).toBeGreaterThan(0);
    expect(bat).toBe(caoDiemVaMua);
  });
});

describe('TCD-04 · 5. assert quá lỏng và quá chặt', () => {
  it('quá lỏng: «tổng > 0, ≥ cước tối thiểu, bội số 1.000» đúng với cả bản lỗi ở OR-01', () => {
    const d = BANG_CUOC[0]!;
    const tong = buggyCalculateFare(sangFareInput(d)).total;
    expect(tong).toBe(54_000); // sai 2.000 đ…
    expect(tong > 0 && tong >= 15_000 && tong % 1_000 === 0).toBe(true); // …mà ba bất biến vẫn «đạt»
  });

  it('quá chặt: so nguyên thân báo giá giữa hai lần gọi giống hệt nhau vẫn lệch ở quoteId, createdAt, expiresAt', async () => {
    const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
    const system = createMemorySystem({ clock });
    const app = createApp(system.deps, { logErrors: false });
    const goi = () => request(app).post('/api/quotes').set('x-user', 'customer:C001').send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 });
    const a = (await goi()).body.quote;
    clock.advanceMs(1_000);
    const b = (await goi()).body.quote;
    const truongLech = Object.keys(a).filter((k) => JSON.stringify(a[k]) !== JSON.stringify(b[k]));
    expect(truongLech).toEqual(['quoteId', 'createdAt', 'expiresAt']);

    // vừa đủ chặt: giá trị nghiệp vụ so chính xác; thời điểm so theo đồng hồ đã tiêm và theo quan hệ
    const { quoteId: _a, createdAt: _b, expiresAt: _c, ...giaTriA } = a;
    const { quoteId: _d, createdAt: _e, expiresAt: _f, ...giaTriB } = b;
    expect(giaTriB).toEqual(giaTriA);
    expect(b.createdAt).toBe(clock.now().toISOString());
    expect(Date.parse(b.expiresAt) - Date.parse(b.createdAt)).toBe(5 * 60_000); // R-QUOTE-01
    expect(b.quoteId).not.toBe(a.quoteId);
  });
});
