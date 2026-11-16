/**
 * TCD-03 — test procedure, test suite và test execution schedule.
 *
 * Test procedure = chuỗi bước có thứ tự (setup → các bước → teardown) để chạy một hoặc nhiều test case.
 * Runner ở đây chạy teardown cả khi một bước thất bại, và ghi nhật ký từng bước — đó là test log tối thiểu.
 */
import request from 'supertest';
import { expect } from 'vitest';
import { createApp } from '../../src/api/app';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';
import { THOI_DIEM } from './mo-rong';

export interface NguCanh {
  clock: FixedClock;
  system: LabSystem;
  app: ReturnType<typeof createApp>;
  bien: Record<string, string>;
}

export interface Buoc {
  ten: string;
  chay: (ctx: NguCanh) => Promise<void>;
}

export interface TestProcedure {
  id: string;
  tieuDe: string;
  testCases: string[];
  setup: Buoc[];
  buoc: Buoc[];
  teardown: Buoc[];
}

export interface KetQuaThuTuc {
  id: string;
  ketQua: 'pass' | 'fail';
  nhatKy: string[];
  loi?: string;
}

/** Môi trường mới hoàn toàn cho mỗi procedure — không chia sẻ trạng thái giữa các lần chạy. */
export function taoNguCanh(): NguCanh {
  const clock = new FixedClock(new Date(THOI_DIEM.ngoaiCaoDiem.iso));
  const system = createMemorySystem({ clock });
  return { clock, system, app: createApp(system.deps, { logErrors: false }), bien: {} };
}

export async function chayThuTuc(tp: TestProcedure, ctx: NguCanh = taoNguCanh()): Promise<KetQuaThuTuc> {
  const nhatKy: string[] = [];
  let loi: string | undefined;
  try {
    for (const [nhom, ds] of [['setup', tp.setup], ['bước', tp.buoc]] as const) {
      for (const b of ds) {
        await b.chay(ctx);
        nhatKy.push(`${nhom}: ${b.ten}`);
      }
    }
  } catch (e) {
    loi = e instanceof Error ? e.message.split('\n')[0] : String(e);
    nhatKy.push(`LỖI: ${loi}`);
  } finally {
    for (const b of tp.teardown) {
      await b.chay(ctx);
      nhatKy.push(`teardown: ${b.ten}`);
    }
  }
  return { id: tp.id, ketQua: loi ? 'fail' : 'pass', nhatKy, loi };
}

// ---------------------------------------------------------------- các bước dùng lại

const KHACH = 'customer:C001';
const DON_75K = { kind: 'food', cityCode: 'SG', distanceKm: 3.2, items: [{ name: 'Cơm tấm sườn', unitPrice: 35_000, quantity: 2 }, { name: 'Trà đá', unitPrice: 5_000, quantity: 1 }] };

const baoGia = (body: object, tongMongDoi: number, bienRa = 'quoteId'): Buoc => ({
  ten: `báo giá đơn đồ ăn ⇒ 200, tổng ${tongMongDoi}`,
  chay: async (ctx) => {
    const res = await request(ctx.app).post('/api/quotes').set('x-user', KHACH).send(body);
    expect(res.status).toBe(200);
    expect(res.body.quote.total).toBe(tongMongDoi);
    ctx.bien[bienRa] = res.body.quote.quoteId;
  },
});

const datChuyen = (khoa: string, bienVao = 'quoteId', bienRa = 'tripId'): Buoc => ({
  ten: 'đặt chuyến từ báo giá ⇒ 201, searching',
  chay: async (ctx) => {
    const res = await request(ctx.app).post('/api/trips').set('x-user', KHACH).set('Idempotency-Key', khoa)
      .send({ quoteId: ctx.bien[bienVao] });
    expect(res.status).toBe(201);
    expect(res.body.trip.status).toBe('searching');
    ctx.bien[bienRa] = res.body.trip.id;
  },
});

const taiXeNhan: Buoc = {
  ten: 'tài xế D101 nhận, ETA 5 phút ⇒ accepted',
  chay: async (ctx) => {
    const res = await request(ctx.app).post(`/api/trips/${ctx.bien.tripId}/accept`).set('x-user', 'driver:D101').send({ etaMinutes: 5 });
    expect(res.status).toBe(200);
    expect(res.body.trip.status).toBe('accepted');
  },
};

const sauPhut = (phut: number): Buoc => ({
  ten: `đồng hồ +${phut} phút`,
  chay: async (ctx) => ctx.clock.advanceMinutes(phut),
});

const khachHuy = (phi: number, lyDo: string): Buoc => ({
  ten: `khách huỷ ⇒ 200, phí ${phi}, ${lyDo}`,
  chay: async (ctx) => {
    const res = await request(ctx.app).post(`/api/trips/${ctx.bien.tripId}/cancel`).set('x-user', KHACH)
      .set('Idempotency-Key', `huy-${ctx.bien.tripId}`).send({});
    expect(res.status).toBe(200);
    expect(res.body).toMatchObject({ cancellationFee: phi, feeReason: lyDo, trip: { status: 'cancelled' } });
  },
});

const taiXeVeOnline: Buoc = {
  ten: 'kiểm hậu điều kiện: D101 về online',
  chay: async (ctx) => {
    expect((await ctx.system.deps.drivers.findById('D101'))?.status).toBe('online');
  },
};

const datLaiTrangThai: Buoc = {
  ten: 'dọn: reset kho, thời tiết, ví giả',
  chay: async (ctx) => ctx.system.reset(),
};

const setupChung: Buoc[] = [
  { ten: `đồng hồ ${THOI_DIEM.ngoaiCaoDiem.moTa}`, chay: async (ctx) => ctx.clock.set(new Date(THOI_DIEM.ngoaiCaoDiem.iso)) },
  { ten: 'SG không mưa', chay: async (ctx) => ctx.system.weather.setRaining([]) },
];

// ---------------------------------------------------------------- procedure

export const TP_FOOD_01: TestProcedure = {
  id: 'TP-FOOD-01',
  tieuDe: 'Đặt đơn đồ ăn, tài xế nhận, khách huỷ trong 2 phút đầu',
  testCases: ['TC-FOOD-001', 'TC-FOOD-013'],
  setup: setupChung,
  buoc: [baoGia(DON_75K, 102_000), datChuyen('tp-food-01-dat'), taiXeNhan, sauPhut(1), khachHuy(0, 'grace_period')],
  teardown: [taiXeVeOnline, datLaiTrangThai],
};

export const TP_FOOD_02: TestProcedure = {
  id: 'TP-FOOD-02',
  tieuDe: 'Đặt đơn đồ ăn, tài xế nhận, khách huỷ sau 5 phút',
  testCases: ['TC-FOOD-001', 'TC-FOOD-014'],
  setup: setupChung,
  buoc: [baoGia(DON_75K, 102_000), datChuyen('tp-food-02-dat'), taiXeNhan, sauPhut(5), khachHuy(10_000, 'charged')],
  teardown: [taiXeVeOnline, datLaiTrangThai],
};

// ---- lượt voucher FOOD15 (2 lần/ngày, R-VOUCHER-07): bản PHỤ THUỘC và bản ĐỘC LẬP

const DON_FOOD15 = { ...DON_75K, voucherCode: 'FOOD15' }; // 75.000 − 11.250 + 27.000 = 90.750

const baoGiaLanBaBiChan: Buoc = {
  ten: 'báo giá FOOD15 lần ba trong ngày ⇒ 422 VOUCHER_USAGE_LIMIT',
  chay: async (ctx) => {
    const res = await request(ctx.app).post('/api/quotes').set('x-user', KHACH).send(DON_FOOD15);
    expect(res.status).toBe(422);
    expect(res.body.error.code).toBe('VOUCHER_USAGE_LIMIT');
  },
};

const haiDonFood15: Buoc[] = [
  baoGia(DON_FOOD15, 90_750, 'q1'), datChuyen('food15-lan-1', 'q1', 't1'),
  baoGia(DON_FOOD15, 90_750, 'q2'), datChuyen('food15-lan-2', 'q2', 't2'),
];

/** Bản phụ thuộc: 03a để lại hai chuyến trong kho; 03b dựa vào chúng mà không tự dựng. */
export const TP_FOOD_03A: TestProcedure = {
  id: 'TP-FOOD-03a', tieuDe: 'Đặt hai đơn FOOD15 (để lại dữ liệu cho 03b)', testCases: ['TC-FOOD-015'],
  setup: setupChung, buoc: haiDonFood15, teardown: [],
};
export const TP_FOOD_03B: TestProcedure = {
  id: 'TP-FOOD-03b', tieuDe: 'Lần thứ ba bị chặn (dựa vào 03a)', testCases: ['TC-FOOD-015'],
  setup: [], buoc: [baoGiaLanBaBiChan], teardown: [],
};

/** Bản độc lập: tự dựng hai lượt đã dùng trong setup, tự dọn ở teardown. */
export const TP_FOOD_03: TestProcedure = {
  id: 'TP-FOOD-03', tieuDe: 'Lượt FOOD15 thứ ba trong ngày bị chặn', testCases: ['TC-FOOD-015'],
  setup: [...setupChung, ...haiDonFood15], buoc: [baoGiaLanBaBiChan], teardown: [datLaiTrangThai],
};

// ---------------------------------------------------------------- test suite

export interface TestSuite {
  id: string;
  mucDich: string;
  thanhPhan: string[]; // id test case cụ thể hoặc id procedure
}

// ---------------------------------------------------------------- test execution schedule

export interface MucLich {
  id: string;
  uuTien: number; // 1 = cao nhất
  phuThuoc: string[];
}

/**
 * Lập test execution schedule: thứ tự topo theo phụ thuộc; trong số mục đã sẵn sàng, ưu tiên cao chạy trước,
 * hoà thì theo id. Có vòng phụ thuộc ⇒ ném lỗi (không thể lập lịch).
 */
export function lapLich(muc: readonly MucLich[]): string[] {
  const conLai = new Map(muc.map((m) => [m.id, m]));
  for (const m of muc) for (const p of m.phuThuoc) if (!conLai.has(p)) throw new Error(`${m.id} phụ thuộc ${p} không có trong lịch`);
  const thuTu: string[] = [];
  while (conLai.size > 0) {
    const sanSang = [...conLai.values()]
      .filter((m) => m.phuThuoc.every((p) => thuTu.includes(p)))
      .sort((a, b) => a.uuTien - b.uuTien || a.id.localeCompare(b.id));
    if (sanSang.length === 0) throw new Error(`Vòng phụ thuộc: ${[...conLai.keys()].join(', ')}`);
    thuTu.push(sanSang[0]!.id);
    conLai.delete(sanSang[0]!.id);
  }
  return thuTu;
}

/** Chạy theo lịch: mục nào có phụ thuộc trượt hoặc bị chặn thì «blocked», không chạy. */
export function chayTheoLich(
  muc: readonly MucLich[],
  ketQuaThat: Record<string, 'pass' | 'fail'>,
): Record<string, 'pass' | 'fail' | 'blocked'> {
  const out: Record<string, 'pass' | 'fail' | 'blocked'> = {};
  const theoId = new Map(muc.map((m) => [m.id, m]));
  for (const id of lapLich(muc)) {
    const chan = theoId.get(id)!.phuThuoc.some((p) => out[p] !== 'pass');
    out[id] = chan ? 'blocked' : (ketQuaThat[id] ?? 'fail');
  }
  return out;
}
