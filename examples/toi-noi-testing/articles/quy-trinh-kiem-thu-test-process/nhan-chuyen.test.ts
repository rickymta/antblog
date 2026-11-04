/**
 * Bài TEST-007 «Quy trình kiểm thử (Test Process)» — sản phẩm của hoạt động test implementation:
 * các test case TC-ACC-01…07 (thiết kế ở mục 4 của bài) viết thành test tự động cho tính năng
 * «tài xế nhận chuyến» (SPEC R-ACCEPT). Tên mỗi test mở đầu bằng mã test case rồi tới mã yêu cầu
 * ⇒ log chạy test chính là một bảng truy vết test case → yêu cầu → kết quả.
 *
 * Chạy: npx vitest run articles/quy-trinh-kiem-thu-test-process
 */
import { beforeEach, describe, expect, it } from 'vitest';
import type { Actor } from '../../src/app/deps';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, cancelByCustomer, createTrip } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';

// Test data lấy từ Phụ lục A của SPEC: C001 là khách; D101, D102 xe máy SG online;
// D103 ô tô SG online; D104 xe máy SG offline; D201 xe máy HN online.
const C001: Actor = { role: 'customer', id: 'C001' };
const driver = (id: string): Actor => ({ role: 'driver', id });

describe('TEST-007 · Tài xế nhận chuyến (R-ACCEPT)', () => {
  const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0)); // 09:00 giờ VN, ngoài cao điểm
  const system: LabSystem = createMemorySystem({ clock });

  beforeEach(async () => {
    await system.reset(); // mỗi test case bắt đầu từ cùng một bộ dữ liệu mẫu
    clock.set(vnTime(2026, 11, 4, 9, 0));
  });

  /** Tiền điều kiện chung: một chuyến xe máy ở SG đang `searching`. */
  async function chuyenXeMaySG() {
    const quote = await createQuote(system.deps, C001, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 });
    return createTrip(system.deps, C001, { quoteId: quote.id });
  }
  const trangThaiTaiXe = async (id: string) => (await system.deps.drivers.findById(id))?.status;

  it('TC-ACC-01 · R-ACCEPT-01/04/05 · tài xế đủ điều kiện nhận được chuyến, chuyển sang busy', async () => {
    const trip = await chuyenXeMaySG();
    const accepted = await acceptTrip(system.deps, driver('D101'), trip.id, 5);
    expect(accepted).toMatchObject({ status: 'accepted', driverId: 'D101', etaMinutes: 5 });
    expect(accepted.acceptedAt).toEqual(vnTime(2026, 11, 4, 9, 0));
    expect(await trangThaiTaiXe('D101')).toBe('busy');
  });

  it('TC-ACC-02 · R-ACCEPT-02 · tài xế offline bị từ chối DRIVER_OFFLINE', async () => {
    const trip = await chuyenXeMaySG();
    await expect(acceptTrip(system.deps, driver('D104'), trip.id, 5)).rejects.toMatchObject({ code: 'DRIVER_OFFLINE' });
  });

  it('TC-ACC-03 · R-ACCEPT-02 · tài xế khác thành phố bị từ chối CITY_MISMATCH', async () => {
    const trip = await chuyenXeMaySG();
    await expect(acceptTrip(system.deps, driver('D201'), trip.id, 5)).rejects.toMatchObject({ code: 'CITY_MISMATCH' });
  });

  it('TC-ACC-04 · R-ACCEPT-02 · tài xế ô tô nhận chuyến xe máy bị từ chối VEHICLE_MISMATCH', async () => {
    const trip = await chuyenXeMaySG();
    await expect(acceptTrip(system.deps, driver('D103'), trip.id, 5)).rejects.toMatchObject({ code: 'VEHICLE_MISMATCH' });
  });

  it('TC-ACC-05 · R-ACCEPT-03 · hai tài xế cùng nhận: đúng một người thắng, người thua vẫn online', async () => {
    const trip = await chuyenXeMaySG();
    const results = await Promise.allSettled([
      acceptTrip(system.deps, driver('D101'), trip.id, 5),
      acceptTrip(system.deps, driver('D102'), trip.id, 7),
    ]);
    const thang = results.filter((r) => r.status === 'fulfilled');
    const thua = results.filter((r) => r.status === 'rejected');
    expect(thang).toHaveLength(1);
    expect(thua).toHaveLength(1);
    expect((thua[0] as PromiseRejectedResult).reason).toMatchObject({ code: 'TRIP_ALREADY_TAKEN' });
    const statuses = [await trangThaiTaiXe('D101'), await trangThaiTaiXe('D102')].sort();
    expect(statuses).toEqual(['busy', 'online']);
  });

  it('TC-ACC-06 · R-ACCEPT-07 · gửi lại lệnh nhận cho chuyến mình đang giữ: trả chuyến cũ, không đổi ETA', async () => {
    const trip = await chuyenXeMaySG();
    const first = await acceptTrip(system.deps, driver('D101'), trip.id, 5);
    clock.advanceMinutes(1);
    const again = await acceptTrip(system.deps, driver('D101'), trip.id, 9);
    expect(again).toMatchObject({ status: 'accepted', driverId: 'D101', etaMinutes: 5 });
    expect(again.acceptedAt).toEqual(first.acceptedAt);
  });

  it('TC-ACC-07 · R-ACCEPT-02 · chuyến khách đã huỷ bị từ chối INVALID_TRANSITION', async () => {
    const trip = await chuyenXeMaySG();
    await cancelByCustomer(system.deps, C001, trip.id);
    await expect(acceptTrip(system.deps, driver('D101'), trip.id, 5)).rejects.toMatchObject({ code: 'INVALID_TRANSITION' });
    expect(await trangThaiTaiXe('D101')).toBe('online');
  });
});
