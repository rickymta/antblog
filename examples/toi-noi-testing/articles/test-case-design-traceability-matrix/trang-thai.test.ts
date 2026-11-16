/**
 * TCD-08 — test case cho R-STATE, có tag truy vết trong tiêu đề:
 *   TC-<NHÓM>-nnn @R-… @TCOND-… · mô tả
 * rtm.ts đọc tiêu đề trong tệp này để dựng RTM — giữ tiêu đề là chuỗi hằng.
 */
import { describe, expect, it } from 'vitest';
import { cancelByCustomer, cancelByDriver, driverStep } from '../../src/app/trips';
import { DomainError } from '../../src/domain/errors';
import { TRIP_STATUSES, isTerminal } from '../../src/domain/trip-state';
import { KHACH, NHAN_LUC, PHUT, T0, dungChuyen } from './dung-chuyen';

async function maLoi(p: Promise<unknown>): Promise<string> {
  try {
    await p;
  } catch (e) {
    if (e instanceof DomainError) return e.code;
    throw e;
  }
  return 'KHONG_LOI';
}

describe('TCD-08 · R-STATE', () => {
  it('TC-STATE-001 @R-STATE-01 @TCOND-STATE-01 · chuyến có đúng 8 trạng thái, 2 trạng thái kết thúc', () => {
    expect([...TRIP_STATUSES]).toEqual(['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled']);
    expect(TRIP_STATUSES.filter(isTerminal)).toEqual(['completed', 'cancelled']);
  });

  it('TC-STATE-002 @R-STATE-02 @R-STATE-06 @R-STATE-07 @TCOND-STATE-02 @TCOND-STATE-08 @TCOND-STATE-09 · luồng chính tới completed, mỗi chuyển một dòng nhật ký', async () => {
    const { deps, tripId, clock } = await dungChuyen({ toi: 'completed' });
    const trip = (await deps.trips.findById(tripId))!;
    expect(trip.status).toBe('completed');
    expect(trip.acceptedAt).toEqual(NHAN_LUC);
    expect(trip.completedAt).toEqual(clock.now());
    const nhatKy = await deps.trips.listEvents(tripId);
    expect(nhatKy.map((e) => e.status)).toEqual(['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed']);
    expect(nhatKy[0]?.occurredAt).toEqual(T0);
    expect(nhatKy[1]?.occurredAt).toEqual(T0);
  });

  it('TC-STATE-003 @R-STATE-02 @R-STATE-07 @TCOND-STATE-03 @TCOND-STATE-09 · khách huỷ được ở searching, accepted, arriving; ghi cancelledAt', async () => {
    for (const toi of ['searching', 'accepted', 'arriving'] as const) {
      const { deps, tripId, clock } = await dungChuyen({ toi });
      clock.set(new Date(NHAN_LUC.getTime() + 30_000));
      await cancelByCustomer(deps, KHACH, tripId);
      const trip = (await deps.trips.findById(tripId))!;
      expect(trip.status, toi).toBe('cancelled');
      expect(trip.cancelledAt, toi).toEqual(clock.now());
      expect(trip.cancelledBy, toi).toBe('customer');
    }
  });

  it('TC-STATE-004 @R-STATE-02 @R-STATE-04 @R-CANCEL-05 @TCOND-STATE-03 @TCOND-STATE-06 @TCOND-CANCEL-11 · tài xế huỷ ở accepted: chuyến về searching, khách không mất phí', async () => {
    const { deps, tripId, clock, taiXe } = await dungChuyen({ toi: 'accepted' });
    clock.set(new Date(NHAN_LUC.getTime() + 5 * PHUT)); // nếu là khách huỷ lúc này thì đã mất phí
    await cancelByDriver(deps, taiXe, tripId);
    const trip = (await deps.trips.findById(tripId))!;
    expect(trip).toMatchObject({ status: 'searching', driverId: null, acceptedAt: null, etaMinutes: null, cancellationFee: 0 });
    expect(trip.searchingSince).toEqual(clock.now());
    expect((await deps.drivers.findById(taiXe.id))?.status).toBe('online');
  });

  it('TC-STATE-005 @R-STATE-03 @R-STATE-06 @TCOND-STATE-05 @TCOND-STATE-08 · khách huỷ ở picked_up bị từ chối, chuyến và nhật ký giữ nguyên', async () => {
    const { deps, tripId } = await dungChuyen({ toi: 'picked_up' });
    const truoc = (await deps.trips.findById(tripId))!;
    const soDong = (await deps.trips.listEvents(tripId)).length;
    expect(await maLoi(cancelByCustomer(deps, KHACH, tripId))).toBe('INVALID_TRANSITION');
    expect(await deps.trips.findById(tripId)).toEqual(truoc);
    expect(await deps.trips.listEvents(tripId)).toHaveLength(soDong);
  });

  it('TC-STATE-006 @R-STATE-03 @TCOND-STATE-05 · tài xế báo đón khi chưa báo tới bị từ chối', async () => {
    const { deps, tripId, taiXe } = await dungChuyen({ toi: 'accepted' });
    expect(await maLoi(driverStep(deps, taiXe, tripId, 'pickup'))).toBe('INVALID_TRANSITION');
    expect((await deps.trips.findById(tripId))?.status).toBe('accepted');
  });

  // Test này kiểm đúng một test condition (trạng thái kết thúc) nhưng KHÔNG có tag ⇒ RTM coi là mồ côi
  // và đồng thời báo TCOND-STATE-04 chưa được phủ. Cố ý để lại cho mục 5 của bài.
  it('khách huỷ lần thứ hai một chuyến đã huỷ thì bị từ chối', async () => {
    const { deps, tripId } = await dungChuyen({ toi: 'searching' });
    await cancelByCustomer(deps, KHACH, tripId);
    expect(await maLoi(cancelByCustomer(deps, KHACH, tripId))).toBe('INVALID_TRANSITION');
  });
});
