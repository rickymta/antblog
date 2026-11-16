/**
 * TCD-07 — step runner tí hon: diễn giải một KichBan thành lời gọi HTTP thật (supertest + createApp)
 * trên hệ thống bộ nhớ với FixedClock. Không cần Cucumber: bước là dữ liệu có kiểu, mỗi loại bước
 * có đúng một hàm thực thi. Không sửa src/.
 */
import request, { type Response } from 'supertest';
import { expect } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';
import type { Given, Gio, KichBan, Then, When } from './kich-ban';

/** «09:03:00.000» ⇒ Date giờ Việt Nam ngày 04/11/2026. */
export function luc(gio: Gio): Date {
  const m = /^(\d{2}):(\d{2}):(\d{2})(?:\.(\d{3}))?$/.exec(gio);
  if (!m) throw new Error(`Giờ không hợp lệ: ${gio}`);
  return new Date(vnTime(2026, 11, 4, Number(m[1]), Number(m[2]), Number(m[3])).getTime() + Number(m[4] ?? 0));
}

const TAI_XE_MAC_DINH = { bike: 'D101', car: 'D103' } as const; // seed: SG xe máy / SG ô tô (SPEC Phụ lục A)

interface NguCanh {
  system: LabSystem;
  clock: FixedClock;
  app: ReturnType<typeof createApp>;
  tripId: string;
  taiXe: string;
  phanHoi: Response[];
  soDongNhatKyTruocWhen: number;
  soKhoa: number;
}

async function chayGiven(c: NguCanh, g: Given): Promise<void> {
  const tx = (path: string, body: object = {}) =>
    request(c.app).post(`/api/trips/${c.tripId}/${path}`).set('x-user', `driver:${c.taiXe}`).send(body);
  switch (g.buoc) {
    case 'co_chuyen': {
      c.clock.set(luc('09:00:00'));
      const yeuCau =
        g.dichVu === 'food'
          ? { kind: 'food', cityCode: 'SG', distanceKm: 3, items: [{ name: 'Cơm tấm', unitPrice: 45_000, quantity: 1 }] }
          : { kind: 'ride', cityCode: 'SG', vehicle: g.xe, distanceKm: 5 };
      const q = await request(c.app).post('/api/quotes').set('x-user', 'customer:C001').send(yeuCau).expect(200);
      const t = await request(c.app)
        .post('/api/trips')
        .set('x-user', 'customer:C001')
        .set('Idempotency-Key', 'dat-chuyen-0001')
        .send({ quoteId: q.body.quote.quoteId, paymentMethod: g.thanhToan ?? 'cash' })
        .expect(201);
      c.tripId = t.body.trip.id;
      c.taiXe = TAI_XE_MAC_DINH[g.xe];
      return;
    }
    case 'tai_xe_nhan':
      c.clock.set(luc(g.luc));
      if (g.taiXe) c.taiXe = g.taiXe;
      await tx('accept', { etaMinutes: g.eta }).expect(200);
      return;
    case 'tai_xe_toi':
      c.clock.set(luc(g.luc));
      await tx('arrive').expect(200);
      return;
    case 'tai_xe_don':
      c.clock.set(luc(g.luc));
      await tx('pickup').expect(200);
      return;
    case 'tai_xe_huy':
      c.clock.set(luc(g.luc));
      await request(c.app)
        .post(`/api/trips/${c.tripId}/cancel`)
        .set('x-user', `driver:${c.taiXe}`)
        .set('Idempotency-Key', `tai-xe-huy-${++c.soKhoa}-0001`)
        .send({})
        .expect(200);
      return;
  }
}

async function chayWhen(c: NguCanh, w: When): Promise<void> {
  c.clock.set(luc(w.luc));
  const nguoi = w.buoc === 'khach_huy' ? 'customer:C001' : `driver:${c.taiXe}`;
  const khoa = w.buoc === 'khach_huy' && w.khoa ? w.khoa : `huy-${++c.soKhoa}-0001`;
  c.phanHoi.push(await request(c.app).post(`/api/trips/${c.tripId}/cancel`).set('x-user', nguoi).set('Idempotency-Key', khoa).send({}));
}

async function kiemThen(c: NguCanh, t: Then): Promise<void> {
  const cuoi = c.phanHoi.at(-1)!;
  const trip = (await c.system.deps.trips.findById(c.tripId))!;
  switch (t.buoc) {
    case 'trang_thai':
      expect(trip.status).toBe(t.la);
      return;
    case 'phi':
      // Kết quả trả về (nếu có) và giá trị lưu trong chuyến phải cùng khớp (R-CANCEL-07).
      if (cuoi.status === 200 && 'cancellationFee' in cuoi.body) expect(cuoi.body.cancellationFee).toBe(t.dong);
      expect(trip.cancellationFee).toBe(t.dong);
      return;
    case 'ly_do':
      expect(cuoi.status).toBe(200);
      expect(cuoi.body.feeReason).toBe(t.la);
      return;
    case 'loi':
      expect(cuoi.status).toBe(t.http);
      expect(cuoi.body.error.code).toBe(t.ma);
      return;
    case 'nhat_ky_khong_doi':
      expect((await c.system.deps.trips.listEvents(c.tripId)).length).toBe(c.soDongNhatKyTruocWhen);
      return;
    case 'khong_goi_vi':
      expect(c.system.payments.calls).toHaveLength(0);
      return;
    case 'thu_vi':
      expect(c.system.payments.calls).toEqual([
        expect.objectContaining({ idempotencyKey: `trip:${c.tripId}:cancel-fee`, amount: t.dong, customerId: 'C001' }),
      ]);
      return;
    case 'payment_status':
      expect(trip.paymentStatus).toBe(t.la);
      return;
    case 'phat_lai_y_nguyen': {
      const [dau, lai] = c.phanHoi as [Response, Response];
      expect(lai.status).toBe(dau.status);
      expect(lai.body).toEqual(dau.body);
      expect(lai.headers['idempotent-replayed']).toBe('true');
      return;
    }
    case 'chua_gan_tai_xe':
      expect(trip.driverId).toBeNull();
      expect(trip.acceptedAt).toBeNull();
      return;
  }
}

/** Chạy một kịch bản từ đầu trên hệ thống mới — kịch bản độc lập nhau. */
export async function chayKichBan(k: KichBan): Promise<void> {
  const clock = new FixedClock(luc('09:00:00'));
  const system = createMemorySystem({ clock });
  const c: NguCanh = {
    system,
    clock,
    app: createApp(system.deps, { logErrors: false }),
    tripId: '',
    taiXe: '',
    phanHoi: [],
    soDongNhatKyTruocWhen: 0,
    soKhoa: 0,
  };
  for (const g of k.given) await chayGiven(c, g);
  c.soDongNhatKyTruocWhen = (await system.deps.trips.listEvents(c.tripId)).length;
  for (const w of k.when) await chayWhen(c, w);
  for (const t of k.then) await kiemThen(c, t);
}
