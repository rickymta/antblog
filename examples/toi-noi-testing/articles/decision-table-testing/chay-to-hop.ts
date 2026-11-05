/**
 * TEST-033 — dựng một chuyến đúng tổ hợp điều kiện của bảng quyết định rồi gửi lệnh huỷ qua tầng use case
 * (src/app/trips.ts) với kho trong bộ nhớ và đồng hồ giả. Trả về những gì quan sát được.
 */
import { createQuote } from '../../src/app/quote';
import { acceptTrip, cancelByCustomer, cancelByDriver, createTrip, driverStep, expireStaleSearches } from '../../src/app/trips';
import type { CancellationInput, FeeReason } from '../../src/domain/cancel';
import { DomainError } from '../../src/domain/errors';
import { vnTime } from '../../src/domain/time';
import type { TripStatus } from '../../src/domain/trip-state';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import type { Combo } from './bang-quyet-dinh';

export const ETA_PHUT = 5; // mốc «tài xế trễ» = nhận + 5 + 10 = 15 phút

/** Dữ liệu thời gian cho C3/C4: F/F → 1 phút; T/F → 5 phút; T/T → 20 phút sau khi nhận. */
export function giayKeTuNhan(x: Combo): number {
  if (x.taiXeTre) return 20 * 60;
  return x.du2Phut ? 5 * 60 : 60;
}

/** «khac» được đại diện bằng picked_up — một phần tử của phân vùng «không được huỷ». */
export const DAI_DIEN_KHAC: TripStatus = 'picked_up';

export interface QuanSat {
  loi: string | null;
  truoc: TripStatus;
  sau: TripStatus;
  phi: number;
  feeReason: FeeReason | null;
  lyDoHuy: string | null;
  taiXe: string | null;
  soDongNhatKy: { truoc: number; sau: number };
}

export async function chayToHop(x: Combo): Promise<QuanSat> {
  const t0 = vnTime(2026, 11, 4, 9, 0); // 09:00 giờ VN — ngoài cao điểm, không ảnh hưởng phí huỷ
  const clock = new FixedClock(t0);
  const { deps } = createMemorySystem({ clock });
  const khach = { role: 'customer' as const, id: 'C001' };
  const taiXe = { role: 'driver' as const, id: x.xe === 'bike' ? 'D101' : 'D103' }; // seed: SG xe máy / SG ô tô

  const quote = await createQuote(deps, khach, { kind: 'ride', cityCode: 'SG', vehicle: x.xe, distanceKm: 5 });
  const trip = await createTrip(deps, khach, { quoteId: quote.id });

  if (x.trangThai !== 'searching') {
    await acceptTrip(deps, taiXe, trip.id, ETA_PHUT);
    if (x.trangThai === 'arriving' || x.trangThai === 'khac') {
      clock.advanceMs(10_000);
      await driverStep(deps, taiXe, trip.id, 'arrive');
    }
    if (x.trangThai === 'khac') {
      clock.advanceMs(10_000);
      await driverStep(deps, taiXe, trip.id, 'pickup');
    }
    clock.set(new Date(t0.getTime() + giayKeTuNhan(x) * 1000));
  }

  const truoc = (await deps.trips.findById(trip.id))!;
  const nhatKyTruoc = (await deps.trips.listEvents(trip.id)).length;
  let loi: string | null = null;
  let feeReason: FeeReason | null = null;
  try {
    if (x.nguoiHuy === 'khach') {
      feeReason = (await cancelByCustomer(deps, khach, trip.id)).feeReason;
    } else if (x.nguoiHuy === 'tai_xe') {
      await cancelByDriver(deps, taiXe, trip.id);
    } else {
      clock.advanceMinutes(deps.config.searchTimeoutMinutes); // R-STATE-05: đủ 5 phút kể từ searchingSince
      await expireStaleSearches(deps);
    }
  } catch (e) {
    if (!(e instanceof DomainError)) throw e;
    loi = e.code;
  }
  const sau = (await deps.trips.findById(trip.id))!;
  return {
    loi,
    truoc: truoc.status,
    sau: sau.status,
    phi: sau.cancellationFee,
    feeReason,
    lyDoHuy: sau.cancelledBy === 'system' ? sau.cancelReason : null,
    taiXe: sau.driverId,
    soDongNhatKy: { truoc: nhatKyTruoc, sau: (await deps.trips.listEvents(trip.id)).length },
  };
}

/** Đầu vào cho hàm thuần customerCancellationFee ứng với một tổ hợp (chỉ dùng khi người huỷ là khách). */
export function dauVaoHamPhi(x: Combo): CancellationInput {
  const acceptedAt = vnTime(2026, 11, 4, 9, 0);
  const now = new Date(acceptedAt.getTime() + giayKeTuNhan(x) * 1000);
  if (x.trangThai === 'searching') return { status: 'searching', vehicle: x.xe, acceptedAt: null, etaMinutes: null, now };
  const status: TripStatus = x.trangThai === 'khac' ? DAI_DIEN_KHAC : x.trangThai;
  return { status, vehicle: x.xe, acceptedAt, etaMinutes: ETA_PHUT, now };
}
