/**
 * TCD-07 — Test case từ user story và acceptance criteria.
 * Chạy: npx vitest run articles/test-case-design-user-story-acceptance-criteria --reporter=verbose
 */
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { customerCancellationFee, type CancellationFee, type CancellationInput } from '../../src/domain/cancel';
import { DomainError } from '../../src/domain/errors';
import request from 'supertest';
import { createApp } from '../../src/api/app';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { buggyCustomerCancellationFee } from './buggy-cancel-fee';
import { ACCEPTANCE_CRITERIA, BANG_QUY_TAC, ETA, VI_DU_DIEN_HINH, type DongQuyTac } from './cau-chuyen';
import { chayKichBan, luc } from './chay-kich-ban';
import { KICH_BAN, vietKichBan } from './kich-ban';

const here = path.dirname(fileURLToPath(import.meta.url));
const SPEC = readFileSync(path.join(here, '../../SPEC.md'), 'utf8');
const MA_TRONG_SPEC = new Set([...SPEC.matchAll(/\*\*(R-[A-Z]+-\d{2}) —/g)].map((m) => m[1]));

// ------------------------------------------------------------------ 1. kiểm chính bộ AC

describe('TCD-07 · 1. kiểm chính bộ acceptance criteria', () => {
  it('mọi mã R-… mà AC và kịch bản trích dẫn đều có trong SPEC.md', () => {
    const daTrich = new Set([...ACCEPTANCE_CRITERIA.flatMap((a) => a.spec), ...KICH_BAN.flatMap((k) => k.spec)]);
    expect([...daTrich].filter((m) => !MA_TRONG_SPEC.has(m))).toEqual([]);
  });

  it('mỗi AC có căn cứ SPEC có ít nhất một kịch bản; mỗi kịch bản trỏ về AC có thật', () => {
    const coKichBan = new Set(KICH_BAN.flatMap((k) => k.ac));
    const acCoCanCu = ACCEPTANCE_CRITERIA.filter((a) => !a.giaDinh).map((a) => a.id);
    expect(acCoCanCu.filter((id) => !coKichBan.has(id))).toEqual([]);
    const maAc = new Set(ACCEPTANCE_CRITERIA.map((a) => a.id));
    expect([...coKichBan].filter((id) => !maAc.has(id))).toEqual([]);
  });

  it('ma trận AC → kịch bản: 7 AC có căn cứ, 15 kịch bản, 5 kịch bản nằm trên biên; AC-08 là giả định chưa có kịch bản chạy được', () => {
    const maTran = Object.fromEntries(
      ACCEPTANCE_CRITERIA.map((a) => [a.id, KICH_BAN.filter((k) => k.ac.includes(a.id)).map((k) => k.id)]),
    );
    expect(maTran).toEqual({
      'AC-01': ['SC-01'],
      'AC-02': ['SC-02', 'SC-03', 'SC-13', 'SC-15'],
      'AC-03': ['SC-04', 'SC-05', 'SC-06', 'SC-07'],
      'AC-04': ['SC-08', 'SC-09', 'SC-10'],
      'AC-05': ['SC-11'],
      'AC-06': ['SC-12', 'SC-13'],
      'AC-07': ['SC-01', 'SC-04', 'SC-14', 'SC-15'],
      'AC-08': [],
    });
    expect(KICH_BAN).toHaveLength(15);
    expect(KICH_BAN.filter((k) => k.bien).map((k) => k.id)).toEqual(['SC-03', 'SC-04', 'SC-08', 'SC-09', 'SC-15']);
  });

  it('kịch bản in ra được thành văn bản Given/When/Then để review cùng BA', () => {
    expect(vietKichBan(KICH_BAN[3]!)).toBe(
      [
        'Scenario: SC-04 — huỷ đúng 2 phút sau khi nhận thì mất 10.000 đ (xe máy, tiền mặt)',
        '  Given khách C001 có chuyến xe máy ở SG tạo lúc 09:00:00, trả tiền mặt',
        '  And   tài xế mặc định nhận lúc 09:01:00 với ETA 5 phút',
        '  When  khách bấm «Huỷ chuyến» lúc 09:03:00.000',
        '  Then  chuyến ở trạng thái «cancelled»',
        '  And   phí huỷ là 10.000 đ',
        '  And   lý do phí là «charged»',
        '  And   paymentStatus là «cash»',
        '  And   ví VíXanh không bị gọi',
      ].join('\n'),
    );
  });
});

// ------------------------------------------------------------------ 2. AC dạng scenario chạy qua API

describe('TCD-07 · 2. acceptance test dạng scenario (Given/When/Then) qua API', () => {
  for (const k of KICH_BAN) {
    it(`${k.id} · ${k.ac.join(' ')} · ${k.ten}`, async () => {
      await chayKichBan(k);
    });
  }
  it.todo('SC-16 · AC-08 · trước khi xác nhận, khách thấy phí dự kiến — chờ BA quyết (SPEC chưa có API xem trước phí)');
});

// ------------------------------------------------------------------ 3. AC dạng rule chạy trên hàm phí

function dauVao(d: DongQuyTac): CancellationInput {
  const acceptedAt = luc('09:01:00');
  const now = new Date(acceptedAt.getTime() + d.sauNhanMs);
  return d.trangThai === 'searching'
    ? { status: 'searching', vehicle: d.xe, acceptedAt: null, etaMinutes: null, now }
    : { status: d.trangThai, vehicle: d.xe, acceptedAt, etaMinutes: ETA, now };
}

function ketQua(f: (x: CancellationInput) => CancellationFee, d: DongQuyTac) {
  try {
    const r = f(dauVao(d));
    return { phi: r.fee, lyDo: r.reason };
  } catch (e) {
    if (e instanceof DomainError) return { loi: e.code };
    throw e;
  }
}

const dat = (f: (x: CancellationInput) => CancellationFee, bang: DongQuyTac[]) =>
  bang.filter((d) => JSON.stringify(ketQua(f, d)) !== JSON.stringify(d.mongDoi)).map((d) => d.id);

describe('TCD-07 · 3. acceptance criteria dạng rule (bảng vào–ra)', () => {
  for (const d of BANG_QUY_TAC) {
    it(`${d.id} · ${d.trangThai}, ${d.xe}, ${d.sauNhanMs} ms sau khi nhận`, () => {
      expect(ketQua(customerCancellationFee, d)).toEqual(d.mongDoi);
    });
  }
});

// ------------------------------------------------------------------ 4. ví dụ trên biên bắt được gì

describe('TCD-07 · 4. ví dụ trên biên trong AC bắt defect so sánh biên', () => {
  it('5 «ví dụ điển hình» (1′, 5′, 20′, đang tìm, đã đón) đều xanh trên bản lỗi — không bắt được', () => {
    expect(dat(customerCancellationFee, VI_DU_DIEN_HINH)).toEqual([]);
    expect(dat(buggyCustomerCancellationFee, VI_DU_DIEN_HINH)).toEqual([]);
  });

  it('bảng 10 dòng có ví dụ trên biên: bản lỗi trượt đúng 3 dòng TD-03, TD-04, TD-06', () => {
    expect(dat(customerCancellationFee, BANG_QUY_TAC)).toEqual([]);
    expect(dat(buggyCustomerCancellationFee, BANG_QUY_TAC)).toEqual(['TD-03', 'TD-04', 'TD-06']);
  });
});

// ------------------------------------------------------------------ 5. mơ hồ «now là lúc nào»

describe('TCD-07 · 5. mơ hồ trong AC: «now» là lúc khách nhìn màn hình hay lúc máy chủ xử lý?', () => {
  it('khách nhìn lúc 09:02:59,500 (còn miễn phí) nhưng yêu cầu tới máy chủ lúc 09:03:00,200 ⇒ bị thu 10.000 đ', async () => {
    const acceptedAt = luc('09:01:00');
    const xemTruoc = customerCancellationFee({
      status: 'accepted', vehicle: 'bike', acceptedAt, etaMinutes: ETA, now: luc('09:02:59.500'),
    });
    expect(xemTruoc).toEqual({ fee: 0, reason: 'grace_period' });

    const clock = new FixedClock(luc('09:00:00'));
    const system = createMemorySystem({ clock });
    const app = createApp(system.deps, { logErrors: false });
    const q = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
      .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }).expect(200);
    const t = await request(app).post('/api/trips').set('x-user', 'customer:C001').set('Idempotency-Key', 'dat-chuyen-0001')
      .send({ quoteId: q.body.quote.quoteId }).expect(201);
    clock.set(acceptedAt);
    await request(app).post(`/api/trips/${t.body.trip.id}/accept`).set('x-user', 'driver:D101').send({ etaMinutes: ETA }).expect(200);
    clock.set(luc('09:03:00.200'));
    const huy = await request(app).post(`/api/trips/${t.body.trip.id}/cancel`).set('x-user', 'customer:C001')
      .set('Idempotency-Key', 'huy-chuyen-0001').send({}).expect(200);
    expect(huy.body).toMatchObject({ cancellationFee: 10_000, feeReason: 'charged' });
  });
});
