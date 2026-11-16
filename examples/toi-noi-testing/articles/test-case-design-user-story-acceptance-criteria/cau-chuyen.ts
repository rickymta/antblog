/**
 * TCD-07 — user story «huỷ chuyến và biết trước phí» viết thành DỮ LIỆU:
 * acceptance criteria dạng rule (AC-01…AC-08) và bảng vào–ra dạng rule (TD-01…TD-10).
 * Kịch bản Given/When/Then nằm ở kich-ban.ts. Không sửa src/.
 */
import type { FeeReason } from '../../src/domain/cancel';
import type { TripStatus } from '../../src/domain/trip-state';
import type { Vehicle } from '../../src/domain/types';

export const USER_STORY = {
  id: 'US-CANCEL-01',
  vaiTro: 'khách',
  mongMuon: 'huỷ chuyến và biết trước mình mất bao nhiêu phí',
  giaTri: 'để tự quyết định huỷ ngay hay chờ tài xế',
} as const;

export interface AcceptanceCriterion {
  id: string;
  /** Dạng rule-oriented: một câu kiểm chứng được. */
  noiDung: string;
  /** Mã SPEC làm căn cứ. Rỗng khi là giả định. */
  spec: string[];
  /** true = tiêu chí do cuộc trao đổi đề xuất, CHƯA có trong SPEC (giả định bài tập). */
  giaDinh?: boolean;
}

export const ACCEPTANCE_CRITERIA: AcceptanceCriterion[] = [
  { id: 'AC-01', noiDung: 'Chưa có tài xế nhận: khách huỷ miễn phí, lý do «searching».', spec: ['R-CANCEL-01', 'R-CANCEL-06'] },
  {
    id: 'AC-02',
    noiDung: 'Tài xế đã nhận nhưng chưa báo tới, chưa đủ 2 phút kể từ lúc nhận: miễn phí, lý do «grace_period».',
    spec: ['R-CANCEL-01', 'R-CANCEL-02'],
  },
  {
    id: 'AC-03',
    noiDung: 'Từ đúng 2 phút trở đi, hoặc tài xế đã báo tới: 10.000 đ xe máy (mọi đơn đồ ăn), 20.000 đ ô tô, lý do «charged».',
    spec: ['R-CANCEL-01', 'R-CANCEL-02', 'R-CANCEL-03'],
  },
  {
    id: 'AC-04',
    noiDung: 'Quá hẳn mốc «lúc nhận + ETA + 10 phút»: miễn phí, lý do «driver_late»; đúng bằng mốc vẫn thu.',
    spec: ['R-CANCEL-04', 'R-CANCEL-06'],
  },
  {
    id: 'AC-05',
    noiDung: 'Đã đón khách trở đi: không huỷ được (409 INVALID_TRANSITION), chuyến và nhật ký giữ nguyên.',
    spec: ['R-CANCEL-01', 'R-STATE-03'],
  },
  {
    id: 'AC-06',
    noiDung: 'Tài xế huỷ: khách không mất phí, chuyến quay lại tìm tài xế; mốc 2 phút tính lại từ lần nhận mới.',
    spec: ['R-CANCEL-05', 'R-STATE-04', 'R-CANCEL-02'],
  },
  {
    id: 'AC-07',
    noiDung: 'Kết quả huỷ báo số phí và lý do; phí > 0 thu qua VíXanh (khoá trip:<id>:cancel-fee) hoặc ghi nhận nếu tiền mặt; phí lưu vào chuyến.',
    spec: ['R-CANCEL-06', 'R-CANCEL-07', 'R-PAY-05', 'R-UI-01'],
  },
  {
    id: 'AC-08',
    noiDung: 'Trước khi xác nhận huỷ, khách thấy phí dự kiến và mốc giờ phí thay đổi.',
    spec: [],
    giaDinh: true,
  },
];

// ------------------------------------------------------------- dạng rule: bảng vào–ra

export interface DongQuyTac {
  id: string;
  trangThai: TripStatus;
  xe: Vehicle;
  /** Thời gian kể từ lúc nhận (ms). ETA cố định 5 phút ⇒ mốc trễ = 15 phút. */
  sauNhanMs: number;
  mongDoi: { phi: number; lyDo: FeeReason } | { loi: 'INVALID_TRANSITION' };
  /** Dòng nằm đúng trên biên của R-CANCEL-02/04. */
  bien?: boolean;
}

export const PHUT = 60_000;
export const ETA = 5;

export const BANG_QUY_TAC: DongQuyTac[] = [
  { id: 'TD-01', trangThai: 'searching', xe: 'bike', sauNhanMs: 0, mongDoi: { phi: 0, lyDo: 'searching' } },
  { id: 'TD-02', trangThai: 'accepted', xe: 'bike', sauNhanMs: 2 * PHUT - 1, mongDoi: { phi: 0, lyDo: 'grace_period' }, bien: true },
  { id: 'TD-03', trangThai: 'accepted', xe: 'bike', sauNhanMs: 2 * PHUT, mongDoi: { phi: 10_000, lyDo: 'charged' }, bien: true },
  { id: 'TD-04', trangThai: 'accepted', xe: 'car', sauNhanMs: 2 * PHUT, mongDoi: { phi: 20_000, lyDo: 'charged' }, bien: true },
  { id: 'TD-05', trangThai: 'arriving', xe: 'bike', sauNhanMs: PHUT / 2, mongDoi: { phi: 10_000, lyDo: 'charged' } },
  { id: 'TD-06', trangThai: 'accepted', xe: 'car', sauNhanMs: 15 * PHUT, mongDoi: { phi: 20_000, lyDo: 'charged' }, bien: true },
  { id: 'TD-07', trangThai: 'accepted', xe: 'car', sauNhanMs: 15 * PHUT + 1, mongDoi: { phi: 0, lyDo: 'driver_late' }, bien: true },
  { id: 'TD-08', trangThai: 'arriving', xe: 'bike', sauNhanMs: 16 * PHUT, mongDoi: { phi: 0, lyDo: 'driver_late' } },
  { id: 'TD-09', trangThai: 'picked_up', xe: 'bike', sauNhanMs: 6 * PHUT, mongDoi: { loi: 'INVALID_TRANSITION' } },
  { id: 'TD-10', trangThai: 'completed', xe: 'car', sauNhanMs: 30 * PHUT, mongDoi: { loi: 'INVALID_TRANSITION' } },
];

/**
 * «Ví dụ điển hình» mà một AC viết vội hay có: 1 phút, 5 phút, 20 phút, đang tìm, đã đón.
 * Không ví dụ nào nằm trên biên.
 */
export const VI_DU_DIEN_HINH: DongQuyTac[] = [
  { id: 'VD-1', trangThai: 'searching', xe: 'bike', sauNhanMs: 0, mongDoi: { phi: 0, lyDo: 'searching' } },
  { id: 'VD-2', trangThai: 'accepted', xe: 'bike', sauNhanMs: 1 * PHUT, mongDoi: { phi: 0, lyDo: 'grace_period' } },
  { id: 'VD-3', trangThai: 'accepted', xe: 'bike', sauNhanMs: 5 * PHUT, mongDoi: { phi: 10_000, lyDo: 'charged' } },
  { id: 'VD-4', trangThai: 'accepted', xe: 'bike', sauNhanMs: 20 * PHUT, mongDoi: { phi: 0, lyDo: 'driver_late' } },
  { id: 'VD-5', trangThai: 'picked_up', xe: 'bike', sauNhanMs: 6 * PHUT, mongDoi: { loi: 'INVALID_TRANSITION' } },
];
