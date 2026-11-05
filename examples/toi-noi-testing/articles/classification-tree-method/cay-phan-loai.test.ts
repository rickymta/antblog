/**
 * TEST-036 · Classification Tree Method cho POST /api/quotes.
 * (1) đếm tổ hợp của cây và ràng buộc; (2) kiểm bộ each-choice 9 dòng phủ mọi class, kể cả khi tính hiệu ứng che lỗi;
 * (3) sinh bộ base choice; (4) chạy các dòng trên API lab (supertest, đồng hồ giả, thời tiết giả).
 *
 * Chạy: npx vitest run articles/classification-tree-method --reporter=verbose
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import type { CityCode } from '../../src/domain/types';
import {
  allCombos, baseChoice, expectedOutcome, isInfeasible, moTa, toRequest, uncoveredClasses, uncoveredEffective,
  type Combo, type Outcome,
} from './cay-phan-loai';

const row = (kind: Combo['kind'], weight: Combo['weight'], vehicle: Combo['vehicle'], city: Combo['city'], rain: boolean, peak: boolean, voucher: Combo['voucher']): Combo =>
  ({ kind, weight, vehicle, city, rain, peak, voucher });

/** Bộ each-choice thiết kế tay: 5 dòng hợp lệ + 2 dòng class không hợp lệ + 2 dòng vi phạm quy tắc. Tổng tiền tính tay trong bài. */
const EACH_CHOICE: { id: string; c: Combo; total?: number }[] = [
  { id: 'E1', c: row('ride', null, 'bike', 'SG', false, false, 'khong'), total: 46_000 },
  { id: 'E2', c: row('parcel', 'nhe', 'bike', 'HN', true, true, 'hop-le'), total: 64_000 },
  { id: 'E3', c: row('parcel', 'vua', 'car', 'SG', true, false, 'khong'), total: 157_000 },
  { id: 'E4', c: row('parcel', 'nang', 'car', 'HN', false, true, 'khong'), total: 219_000 },
  { id: 'E5', c: row('food', null, 'bike', 'SG', false, true, 'hop-le'), total: 80_000 },
  { id: 'E6', c: row('ride', null, 'bike', 'VT', false, false, 'khong') },
  { id: 'E7', c: row('ride', null, 'bike', 'SG', false, false, 'sai-pham-vi') },
  { id: 'E8', c: row('food', null, 'car', 'SG', false, false, 'khong') },
  { id: 'E9', c: row('parcel', 'nang', 'bike', 'SG', false, false, 'khong') },
];

/** Bộ «ngây thơ» 5 dòng: đủ mặt mọi class, nhưng dòng N3 bị lỗi thành phố chặn trước. */
const NAIVE: Combo[] = [
  row('ride', null, 'bike', 'SG', false, false, 'khong'),
  row('parcel', 'nhe', 'car', 'HN', true, true, 'hop-le'),
  row('parcel', 'vua', 'bike', 'VT', false, false, 'sai-pham-vi'),
  row('parcel', 'nang', 'car', 'SG', false, false, 'khong'),
  row('food', null, 'bike', 'HN', false, true, 'khong'),
];

const STATUS: Record<Outcome, number> = { OK: 200, VALIDATION_ERROR: 400, CITY_NOT_SUPPORTED: 422, VEHICLE_TOO_SMALL: 422, VOUCHER_NOT_APPLICABLE: 422 };

async function quote(c: Combo) {
  const { body, time, raining } = toRequest(c);
  const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, time.hour, time.minute)), rainingCities: raining as CityCode[] });
  return request(createApp(system.deps, { logErrors: false })).post('/api/quotes').set('x-user', 'customer:C001').send(body);
}

describe('Mô hình cây phân loại', () => {
  it('đếm tổ hợp: 360 theo cây, 60 bị loại trừ, 300 dựng được — phân theo loại kết quả', () => {
    const all = allCombos();
    const feasible = all.filter((c) => !isInfeasible(c));
    const byOutcome: Record<string, number> = {};
    for (const c of feasible) byOutcome[expectedOutcome(c)] = (byOutcome[expectedOutcome(c)] ?? 0) + 1;
    console.log(`Tổ hợp theo cây: ${all.length} · loại trừ: ${all.length - feasible.length} · dựng được: ${feasible.length}`, byOutcome);
    expect([all.length, feasible.length]).toEqual([360, 300]);
    expect(byOutcome).toEqual({ OK: 128, VOUCHER_NOT_APPLICABLE: 64, VEHICLE_TOO_SMALL: 24, CITY_NOT_SUPPORTED: 54, VALIDATION_ERROR: 30 });
  });

  it('bộ each-choice 9 dòng phủ mọi class, kể cả khi chỉ tính dòng class thực sự được xử lý', () => {
    const rows = EACH_CHOICE.map((r) => r.c);
    expect(uncoveredClasses(rows)).toEqual([]);
    expect(uncoveredEffective(rows)).toEqual([]);
  });

  it('bộ «ngây thơ» 5 dòng đủ mặt mọi class nhưng hai class bị lỗi thành phố che', () => {
    expect(uncoveredClasses(NAIVE)).toEqual([]);
    const masked = uncoveredEffective(NAIVE);
    console.log('Bộ ngây thơ — class bị che:', masked.join(', '));
    expect(masked).toEqual(['weight=vua', 'voucher=sai-pham-vi']);
  });

  it('base choice từ dòng gốc E1: 12 dòng, đúng một dòng rơi vào ràng buộc «> 20 kg phải ô tô»', () => {
    const rows = baseChoice(EACH_CHOICE[0]!.c);
    console.log(rows.map((r, i) => `B${String(i + 1).padStart(2, '0')} ${moTa(r)} ⇒ ${expectedOutcome(r)}`).join('\n'));
    expect(rows).toHaveLength(12);
    expect(rows.filter(isInfeasible)).toEqual([]);
    expect(rows.filter((r) => expectedOutcome(r) === 'VEHICLE_TOO_SMALL')).toHaveLength(1);
    expect(uncoveredEffective(rows)).toEqual(['weight=nang']);
  });
});

describe('Chạy bộ each-choice trên POST /api/quotes', () => {
  it.each(EACH_CHOICE)('$id · đúng mã kết quả và tổng tiền', async ({ c, total }) => {
    const res = await quote(c);
    const outcome = expectedOutcome(c);
    expect(res.status).toBe(STATUS[outcome]);
    if (outcome === 'OK') expect(res.body.quote.total).toBe(total);
    else expect(res.body.error.code).toBe(outcome);
  });
});

describe('Chạy bộ base choice trên POST /api/quotes (oracle mức loại kết quả)', () => {
  it.each(baseChoice(EACH_CHOICE[0]!.c).map((c, i) => ({ id: `B${String(i + 1).padStart(2, '0')}`, c })))('$id', async ({ c }) => {
    const res = await quote(c);
    expect(res.status).toBe(STATUS[expectedOutcome(c)]);
    if (expectedOutcome(c) !== 'OK') expect(res.body.error.code).toBe(expectedOutcome(c));
  });
});
