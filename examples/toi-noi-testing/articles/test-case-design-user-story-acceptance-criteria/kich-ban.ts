/**
 * TCD-07 — acceptance criteria dạng scenario (Given/When/Then) viết thành dữ liệu có kiểu.
 * Mỗi bước là một object; step runner (chay-kich-ban.ts) diễn giải chúng thành lời gọi API thật.
 * Giờ ghi theo giờ Việt Nam ngày 04/11/2026 (ngoài cao điểm). Chuyến luôn tạo lúc 09:00:00.
 */
import type { FeeReason } from '../../src/domain/cancel';
import type { TripStatus } from '../../src/domain/trip-state';
import type { Vehicle } from '../../src/domain/types';

/** «HH:MM:SS» hoặc «HH:MM:SS.mmm». */
export type Gio = string;

export type Given =
  | { buoc: 'co_chuyen'; xe: Vehicle; dichVu?: 'ride' | 'food'; thanhToan?: 'cash' | 'vixanh' }
  | { buoc: 'tai_xe_nhan'; taiXe?: string; eta: number; luc: Gio }
  | { buoc: 'tai_xe_toi'; luc: Gio }
  | { buoc: 'tai_xe_don'; luc: Gio }
  | { buoc: 'tai_xe_huy'; luc: Gio };

export type When =
  | { buoc: 'khach_huy'; luc: Gio; khoa?: string }
  | { buoc: 'tai_xe_huy'; luc: Gio };

export type Then =
  | { buoc: 'trang_thai'; la: TripStatus }
  | { buoc: 'phi'; dong: number }
  | { buoc: 'ly_do'; la: FeeReason }
  | { buoc: 'loi'; http: number; ma: string }
  | { buoc: 'nhat_ky_khong_doi' }
  | { buoc: 'khong_goi_vi' }
  | { buoc: 'thu_vi'; dong: number }
  | { buoc: 'payment_status'; la: 'cash' | 'paid' | 'none' }
  | { buoc: 'phat_lai_y_nguyen' }
  | { buoc: 'chua_gan_tai_xe' };

export interface KichBan {
  id: string;
  ten: string;
  ac: string[];
  spec: string[];
  bien?: boolean;
  given: Given[];
  when: When[];
  then: Then[];
}

export const KICH_BAN: KichBan[] = [
  {
    id: 'SC-01', ten: 'huỷ khi đang tìm tài xế thì miễn phí', ac: ['AC-01', 'AC-07'], spec: ['R-CANCEL-01', 'R-CANCEL-06'],
    given: [{ buoc: 'co_chuyen', xe: 'bike' }],
    when: [{ buoc: 'khach_huy', luc: '09:00:30' }],
    then: [{ buoc: 'trang_thai', la: 'cancelled' }, { buoc: 'phi', dong: 0 }, { buoc: 'ly_do', la: 'searching' }, { buoc: 'payment_status', la: 'none' }, { buoc: 'khong_goi_vi' }],
  },
  {
    id: 'SC-02', ten: 'huỷ 1 phút sau khi tài xế nhận thì miễn phí', ac: ['AC-02'], spec: ['R-CANCEL-01', 'R-CANCEL-02'],
    given: [{ buoc: 'co_chuyen', xe: 'car' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:02:00' }],
    then: [{ buoc: 'trang_thai', la: 'cancelled' }, { buoc: 'phi', dong: 0 }, { buoc: 'ly_do', la: 'grace_period' }],
  },
  {
    id: 'SC-03', ten: 'huỷ 1 phút 59,999 giây sau khi nhận vẫn miễn phí', ac: ['AC-02'], spec: ['R-CANCEL-02'], bien: true,
    given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:02:59.999' }],
    then: [{ buoc: 'phi', dong: 0 }, { buoc: 'ly_do', la: 'grace_period' }],
  },
  {
    id: 'SC-04', ten: 'huỷ đúng 2 phút sau khi nhận thì mất 10.000 đ (xe máy, tiền mặt)', ac: ['AC-03', 'AC-07'], spec: ['R-CANCEL-02', 'R-CANCEL-03', 'R-PAY-05'], bien: true,
    given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:03:00.000' }],
    then: [{ buoc: 'trang_thai', la: 'cancelled' }, { buoc: 'phi', dong: 10_000 }, { buoc: 'ly_do', la: 'charged' }, { buoc: 'payment_status', la: 'cash' }, { buoc: 'khong_goi_vi' }],
  },
  {
    id: 'SC-05', ten: 'huỷ chuyến ô tô 5 phút sau khi nhận thì mất 20.000 đ', ac: ['AC-03'], spec: ['R-CANCEL-01', 'R-CANCEL-03'],
    given: [{ buoc: 'co_chuyen', xe: 'car' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:06:00' }],
    then: [{ buoc: 'phi', dong: 20_000 }, { buoc: 'ly_do', la: 'charged' }],
  },
  {
    id: 'SC-06', ten: 'tài xế đã báo tới thì không còn 2 phút miễn phí', ac: ['AC-03'], spec: ['R-CANCEL-01'],
    given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }, { buoc: 'tai_xe_toi', luc: '09:01:30' }],
    when: [{ buoc: 'khach_huy', luc: '09:02:00' }],
    then: [{ buoc: 'trang_thai', la: 'cancelled' }, { buoc: 'phi', dong: 10_000 }, { buoc: 'ly_do', la: 'charged' }],
  },
  {
    id: 'SC-07', ten: 'đơn đồ ăn tính phí như xe máy', ac: ['AC-03'], spec: ['R-CANCEL-03', 'R-GEN-04'],
    given: [{ buoc: 'co_chuyen', xe: 'bike', dichVu: 'food' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:05:00' }],
    then: [{ buoc: 'phi', dong: 10_000 }, { buoc: 'ly_do', la: 'charged' }],
  },
  {
    id: 'SC-08', ten: 'đúng mốc nhận + ETA + 10 phút vẫn thu phí', ac: ['AC-04'], spec: ['R-CANCEL-04'], bien: true,
    given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:16:00.000' }],
    then: [{ buoc: 'phi', dong: 10_000 }, { buoc: 'ly_do', la: 'charged' }],
  },
  {
    id: 'SC-09', ten: 'quá mốc 1 ms thì miễn phí vì tài xế trễ', ac: ['AC-04'], spec: ['R-CANCEL-04', 'R-CANCEL-06'], bien: true,
    given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:16:00.001' }],
    then: [{ buoc: 'phi', dong: 0 }, { buoc: 'ly_do', la: 'driver_late' }],
  },
  {
    id: 'SC-10', ten: 'tài xế tới muộn rồi khách mới huỷ: vẫn miễn phí', ac: ['AC-04'], spec: ['R-CANCEL-04'],
    given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }, { buoc: 'tai_xe_toi', luc: '09:17:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:17:30' }],
    then: [{ buoc: 'trang_thai', la: 'cancelled' }, { buoc: 'phi', dong: 0 }, { buoc: 'ly_do', la: 'driver_late' }],
  },
  {
    id: 'SC-11', ten: 'đã đón khách thì không huỷ được', ac: ['AC-05'], spec: ['R-CANCEL-01', 'R-STATE-03'],
    given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }, { buoc: 'tai_xe_toi', luc: '09:05:00' }, { buoc: 'tai_xe_don', luc: '09:06:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:07:00' }],
    then: [{ buoc: 'loi', http: 409, ma: 'INVALID_TRANSITION' }, { buoc: 'trang_thai', la: 'picked_up' }, { buoc: 'nhat_ky_khong_doi' }, { buoc: 'phi', dong: 0 }],
  },
  {
    id: 'SC-12', ten: 'tài xế huỷ thì khách không mất phí, chuyến tìm tài xế lại', ac: ['AC-06'], spec: ['R-CANCEL-05', 'R-STATE-04'],
    given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'tai_xe_huy', luc: '09:05:00' }],
    then: [{ buoc: 'trang_thai', la: 'searching' }, { buoc: 'chua_gan_tai_xe' }, { buoc: 'phi', dong: 0 }, { buoc: 'khong_goi_vi' }],
  },
  {
    id: 'SC-13', ten: 'tài xế khác nhận lại: 2 phút miễn phí tính từ lần nhận mới', ac: ['AC-06', 'AC-02'], spec: ['R-STATE-04', 'R-CANCEL-02'],
    given: [
      { buoc: 'co_chuyen', xe: 'bike' },
      { buoc: 'tai_xe_nhan', taiXe: 'D101', eta: 5, luc: '09:01:00' },
      { buoc: 'tai_xe_huy', luc: '09:04:00' },
      { buoc: 'tai_xe_nhan', taiXe: 'D102', eta: 5, luc: '09:05:00' },
    ],
    when: [{ buoc: 'khach_huy', luc: '09:06:00' }],
    then: [{ buoc: 'phi', dong: 0 }, { buoc: 'ly_do', la: 'grace_period' }],
  },
  {
    id: 'SC-14', ten: 'trả bằng VíXanh: phí được thu đúng một lần', ac: ['AC-07'], spec: ['R-CANCEL-07', 'R-PAY-05'],
    given: [{ buoc: 'co_chuyen', xe: 'car', thanhToan: 'vixanh' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:06:00' }],
    then: [{ buoc: 'phi', dong: 20_000 }, { buoc: 'ly_do', la: 'charged' }, { buoc: 'thu_vi', dong: 20_000 }, { buoc: 'payment_status', la: 'paid' }],
  },
  {
    id: 'SC-15', ten: 'bấm lại sau mốc 2 phút với cùng khoá: nhận lại kết quả lần đầu', ac: ['AC-02', 'AC-07'], spec: ['R-CANCEL-02', 'R-IDEM-02'], bien: true,
    given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
    when: [{ buoc: 'khach_huy', luc: '09:02:59', khoa: 'huy-sc15-0001' }, { buoc: 'khach_huy', luc: '09:03:30', khoa: 'huy-sc15-0001' }],
    then: [{ buoc: 'phat_lai_y_nguyen' }, { buoc: 'phi', dong: 0 }, { buoc: 'ly_do', la: 'grace_period' }, { buoc: 'khong_goi_vi' }],
  },
];

// ------------------------------------------------------------- in thành văn bản cho người đọc

const XE: Record<Vehicle, string> = { bike: 'xe máy', car: 'ô tô' };

function cauGiven(g: Given): string {
  switch (g.buoc) {
    case 'co_chuyen':
      return `khách C001 có chuyến ${g.dichVu === 'food' ? 'đồ ăn' : XE[g.xe]} ở SG tạo lúc 09:00:00, trả ${g.thanhToan === 'vixanh' ? 'VíXanh' : 'tiền mặt'}`;
    case 'tai_xe_nhan':
      return `tài xế ${g.taiXe ?? 'mặc định'} nhận lúc ${g.luc} với ETA ${g.eta} phút`;
    case 'tai_xe_toi':
      return `tài xế báo đã tới lúc ${g.luc}`;
    case 'tai_xe_don':
      return `tài xế báo đã đón khách lúc ${g.luc}`;
    case 'tai_xe_huy':
      return `tài xế huỷ lúc ${g.luc}`;
  }
}

function cauWhen(w: When): string {
  return w.buoc === 'khach_huy' ? `khách bấm «Huỷ chuyến» lúc ${w.luc}${w.khoa ? ` (khoá ${w.khoa})` : ''}` : `tài xế huỷ lúc ${w.luc}`;
}

function cauThen(t: Then): string {
  switch (t.buoc) {
    case 'trang_thai': return `chuyến ở trạng thái «${t.la}»`;
    case 'phi': return `phí huỷ là ${t.dong.toLocaleString('vi-VN')} đ`;
    case 'ly_do': return `lý do phí là «${t.la}»`;
    case 'loi': return `hệ thống từ chối với ${t.http} ${t.ma}`;
    case 'nhat_ky_khong_doi': return 'nhật ký trạng thái không thêm dòng nào';
    case 'khong_goi_vi': return 'ví VíXanh không bị gọi';
    case 'thu_vi': return `VíXanh thu ${t.dong.toLocaleString('vi-VN')} đ đúng một lần với khoá trip:<id>:cancel-fee`;
    case 'payment_status': return `paymentStatus là «${t.la}»`;
    case 'phat_lai_y_nguyen': return 'lần bấm thứ hai nhận lại đúng phản hồi lần đầu, có header Idempotent-Replayed';
    case 'chua_gan_tai_xe': return 'chuyến không còn tài xế được gán';
  }
}

/** In kịch bản thành văn bản Given/When/Then tiếng Việt (dùng trong README và để review với BA). */
export function vietKichBan(k: KichBan): string {
  const khoi = (nhan: string, cau: string[]) => cau.map((c, i) => `  ${i === 0 ? nhan : 'And  '} ${c}`);
  return [
    `Scenario: ${k.id} — ${k.ten}`,
    ...khoi('Given', k.given.map(cauGiven)),
    ...khoi('When ', k.when.map(cauWhen)),
    ...khoi('Then ', k.then.map(cauThen)),
  ].join('\n');
}
