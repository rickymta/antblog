/**
 * TCD-03 — test case LOGIC cho đơn đồ ăn (SPEC R-FOOD) và bảng test data cụ thể.
 *
 * Test case logic chỉ nói «loại» giá trị (tiền món ở biên tối thiểu, giờ cao điểm, voucher chạm trần…)
 * và kết quả mong đợi dưới dạng QUY TẮC. Bảng dữ liệu TD-xx chọn giá trị thật cho từng test case logic.
 * Cột `tinhTay` là kết quả mong đợi người viết tính bằng tay từ SPEC — hàm mở rộng (mo-rong.ts) tính lại
 * bằng công thức và test đối chiếu hai nguồn (ghi hai lần, kiểm chéo).
 */

export type Loi = { http: 400 | 422; code: string };

/** Ràng buộc mà mọi dòng dữ liệu của một test case logic phải thoả — dữ liệu không được «trượt» khỏi lớp. */
export interface DongDuLieu {
  id: string; // TD-nn
  tc: string; // TC-FOOD-nnn (test case logic)
  mon: { name: string; unitPrice: number; quantity: number }[];
  km: number;
  caoDiem: boolean;
  mua: boolean;
  voucher?: string;
  /** undefined = không gửi trường `vehicle`. */
  vehicle?: 'bike' | 'car';
  /** Kết quả mong đợi tính tay: tổng tiền khách trả, hoặc mã lỗi. */
  tinhTay: number | Loi;
}

export interface TestCaseLogic {
  id: string;
  tieuDe: string;
  truyVet: string[];
  /** Tiền điều kiện viết ở mức logic. */
  tienDieuKien: string;
  /** Đầu vào viết ở mức logic. */
  dauVao: string;
  /** Kết quả mong đợi viết ở mức logic (quy tắc, chưa có con số). */
  ketQuaMongDoi: string;
  /** Lỗi mong đợi (nếu là ca âm) — đã cụ thể ngay ở mức logic vì mã lỗi không phụ thuộc dữ liệu. */
  loi?: Loi;
  /** Dòng dữ liệu có thuộc đúng lớp mà test case logic mô tả không. */
  rangBuoc: (d: DongDuLieu) => boolean;
}

export const tienMon = (d: Pick<DongDuLieu, 'mon'>) => d.mon.reduce((s, m) => s + m.unitPrice * m.quantity, 0);

export const TEST_CASE_LOGIC: TestCaseLogic[] = [
  {
    id: 'TC-FOOD-001',
    tieuDe: 'Đơn hợp lệ cơ bản: tổng = tiền món + phí giao',
    truyVet: ['R-FOOD-01', 'R-FOOD-02', 'R-FOOD-04'],
    tienDieuKien: 'ngoài giờ cao điểm, thành phố không mưa',
    dauVao: 'nhiều dòng món, tiền món trên mức tối thiểu, không voucher, quãng đường trung bình',
    ketQuaMongDoi: '200; tổng = tiền món + phí giao tính theo công thức xe máy',
    rangBuoc: (d) => !d.caoDiem && !d.mua && !d.voucher && d.mon.length >= 2 && tienMon(d) > 30_000,
  },
  {
    id: 'TC-FOOD-002',
    tieuDe: 'Tiền món đúng bằng mức tối thiểu được chấp nhận',
    truyVet: ['R-FOOD-03'],
    tienDieuKien: 'ngoài giờ cao điểm, không mưa',
    dauVao: 'tiền món = mức tối thiểu',
    ketQuaMongDoi: '200; báo giá thành công',
    rangBuoc: (d) => tienMon(d) === 30_000 && !d.voucher,
  },
  {
    id: 'TC-FOOD-003',
    tieuDe: 'Tiền món dưới mức tối thiểu một bậc 1.000 đ bị từ chối',
    truyVet: ['R-FOOD-03', 'R-VALID-01'],
    tienDieuKien: 'bất kỳ',
    dauVao: 'tiền món = mức tối thiểu − 1.000 đ, các trường khác hợp lệ',
    ketQuaMongDoi: '422 SUBTOTAL_TOO_LOW',
    loi: { http: 422, code: 'SUBTOTAL_TOO_LOW' },
    rangBuoc: (d) => tienMon(d) === 29_000,
  },
  {
    id: 'TC-FOOD-004',
    tieuDe: 'Phí giao quanh cước tối thiểu xe máy',
    truyVet: ['R-FOOD-04', 'R-FARE-07', 'R-FARE-08'],
    tienDieuKien: 'ngoài giờ cao điểm, không mưa',
    dauVao: 'quãng đường ngắn: một giá trị cho cước cơ bản dưới mức tối thiểu, một giá trị vừa vượt',
    ketQuaMongDoi: '200; phí giao = max(cước cơ bản, tối thiểu) làm tròn lên 1.000 đ',
    rangBuoc: (d) => !d.caoDiem && !d.mua && d.km <= 1,
  },
  {
    id: 'TC-FOOD-005',
    tieuDe: 'Phí giao chịu hệ số cao điểm và phụ phí mưa',
    truyVet: ['R-FOOD-04', 'R-FARE-04', 'R-FARE-05'],
    tienDieuKien: 'cao điểm và/hoặc mưa (mỗi dòng một tổ hợp khác «không cả hai»)',
    dauVao: 'quãng đường cho cước cơ bản có phần lẻ sau khi nhân 1,25',
    ketQuaMongDoi: '200; phí giao theo thứ tự cơ bản → ×1,25 → +8.000 → làm tròn lên',
    rangBuoc: (d) => d.caoDiem || d.mua,
  },
  {
    id: 'TC-FOOD-006',
    tieuDe: 'Voucher phần trăm tính trên tiền món, không trừ vào phí giao',
    truyVet: ['R-FOOD-05', 'R-VOUCHER-06', 'R-FOOD-04'],
    tienDieuKien: 'ngoài giờ cao điểm, không mưa; voucher còn hạn, đúng phạm vi, còn lượt',
    dauVao: 'voucher không có tiền món tối thiểu; percent × tiền món nhỏ hơn trần',
    ketQuaMongDoi: '200; tiền giảm = percent × tiền món; phí giao giữ nguyên; tổng = món − giảm + phí giao',
    rangBuoc: (d) => d.voucher === 'GIAM20' && tienMon(d) * 0.2 < 30_000,
  },
  {
    id: 'TC-FOOD-007',
    tieuDe: 'Tiền giảm của voucher bị chặn ở trần',
    truyVet: ['R-FOOD-05', 'R-VOUCHER-06'],
    tienDieuKien: 'như TC-FOOD-006',
    dauVao: 'percent × tiền món bằng trần; percent × tiền món lớn hơn trần',
    ketQuaMongDoi: '200; tiền giảm = trần',
    rangBuoc: (d) => d.voucher === 'GIAM20' && tienMon(d) * 0.2 >= 30_000,
  },
  {
    id: 'TC-FOOD-008',
    tieuDe: 'Tiền món dưới mức tối thiểu của voucher bị từ chối',
    truyVet: ['R-FOOD-05', 'R-VOUCHER-06', 'R-VOUCHER-08'],
    tienDieuKien: 'voucher có minAmount > 0, còn hạn, đúng phạm vi',
    dauVao: 'tiền món = minAmount − 1.000 đ (nhưng ≥ 30.000 đ)',
    ketQuaMongDoi: '422 VOUCHER_MIN_AMOUNT',
    loi: { http: 422, code: 'VOUCHER_MIN_AMOUNT' },
    rangBuoc: (d) => d.voucher === 'FOOD15' && tienMon(d) === 49_000,
  },
  {
    id: 'TC-FOOD-009',
    tieuDe: 'Tiền món đúng bằng mức tối thiểu của voucher được giảm',
    truyVet: ['R-FOOD-05', 'R-VOUCHER-06'],
    tienDieuKien: 'như TC-FOOD-008',
    dauVao: 'tiền món = minAmount',
    ketQuaMongDoi: '200; tiền giảm = percent × tiền món (có thể không tròn nghìn)',
    rangBuoc: (d) => d.voucher === 'FOOD15' && tienMon(d) === 50_000,
  },
  {
    id: 'TC-FOOD-010',
    tieuDe: 'Đơn đồ ăn gửi loại xe ô tô bị từ chối',
    truyVet: ['R-FOOD-06', 'R-API-06'],
    tienDieuKien: 'bất kỳ',
    dauVao: 'vehicle = car, các trường khác hợp lệ',
    ketQuaMongDoi: '400 VALIDATION_ERROR',
    loi: { http: 400, code: 'VALIDATION_ERROR' },
    rangBuoc: (d) => d.vehicle === 'car',
  },
  {
    id: 'TC-FOOD-011',
    tieuDe: 'Bỏ trống loại xe hay gửi bike cho cùng kết quả',
    truyVet: ['R-FOOD-06', 'R-GEN-04'],
    tienDieuKien: 'ngoài giờ cao điểm, không mưa',
    dauVao: 'cùng đơn, một lần không gửi vehicle, một lần gửi bike',
    ketQuaMongDoi: '200; vehicle trả về là bike; tổng như nhau',
    rangBuoc: (d) => d.vehicle !== 'car' && !d.voucher,
  },
  {
    id: 'TC-FOOD-012',
    tieuDe: 'Dòng món sai định dạng bị từ chối',
    truyVet: ['R-FOOD-02', 'R-API-06'],
    tienDieuKien: 'bất kỳ',
    dauVao: 'một dòng món có đơn giá không phải bội số 1.000 đ, hoặc số lượng ngoài [1; 50]',
    ketQuaMongDoi: '400 VALIDATION_ERROR',
    loi: { http: 400, code: 'VALIDATION_ERROR' },
    rangBuoc: (d) => d.mon.some((m) => m.unitPrice % 1_000 !== 0 || m.quantity < 1 || m.quantity > 50),
  },
];

const COM_TAM = { name: 'Cơm tấm sườn', unitPrice: 35_000, quantity: 2 };
const TRA_DA = { name: 'Trà đá', unitPrice: 5_000, quantity: 1 };
const BUN_BO = { name: 'Bún bò Huế', unitPrice: 45_000, quantity: 1 };

/** Bảng test data — mỗi dòng thuộc đúng một test case logic. */
export const BANG_DU_LIEU: DongDuLieu[] = [
  { id: 'TD-01', tc: 'TC-FOOD-001', mon: [COM_TAM, TRA_DA], km: 3.2, caoDiem: false, mua: false, tinhTay: 102_000 },
  { id: 'TD-02', tc: 'TC-FOOD-002', mon: [{ name: 'Bánh mì', unitPrice: 15_000, quantity: 2 }], km: 2, caoDiem: false, mua: false, tinhTay: 51_000 },
  {
    id: 'TD-03', tc: 'TC-FOOD-003',
    mon: [{ name: 'Bánh mì', unitPrice: 14_000, quantity: 1 }, { name: 'Sữa đậu', unitPrice: 15_000, quantity: 1 }],
    km: 2, caoDiem: false, mua: false, tinhTay: { http: 422, code: 'SUBTOTAL_TOO_LOW' },
  },
  { id: 'TD-04', tc: 'TC-FOOD-004', mon: [{ name: 'Phở bò', unitPrice: 40_000, quantity: 1 }], km: 0.6, caoDiem: false, mua: false, tinhTay: 55_000 },
  { id: 'TD-05', tc: 'TC-FOOD-004', mon: [{ name: 'Phở bò', unitPrice: 40_000, quantity: 1 }], km: 0.7, caoDiem: false, mua: false, tinhTay: 56_000 },
  { id: 'TD-06', tc: 'TC-FOOD-005', mon: [COM_TAM, TRA_DA], km: 3.3, caoDiem: true, mua: false, tinhTay: 109_000 },
  { id: 'TD-07', tc: 'TC-FOOD-005', mon: [COM_TAM, TRA_DA], km: 3.3, caoDiem: false, mua: true, tinhTay: 110_000 },
  { id: 'TD-08', tc: 'TC-FOOD-005', mon: [COM_TAM, TRA_DA], km: 3.3, caoDiem: true, mua: true, tinhTay: 117_000 },
  { id: 'TD-09', tc: 'TC-FOOD-006', mon: [COM_TAM, TRA_DA], km: 3.2, caoDiem: false, mua: false, voucher: 'GIAM20', tinhTay: 87_000 },
  { id: 'TD-10', tc: 'TC-FOOD-007', mon: [{ name: 'Lẩu thái', unitPrice: 150_000, quantity: 1 }], km: 3.2, caoDiem: false, mua: false, voucher: 'GIAM20', tinhTay: 147_000 },
  { id: 'TD-11', tc: 'TC-FOOD-007', mon: [{ name: 'Lẩu thái', unitPrice: 200_000, quantity: 1 }], km: 3.2, caoDiem: false, mua: false, voucher: 'GIAM20', tinhTay: 197_000 },
  {
    id: 'TD-12', tc: 'TC-FOOD-008', mon: [{ name: 'Gà rán', unitPrice: 49_000, quantity: 1 }], km: 3.2, caoDiem: false, mua: false,
    voucher: 'FOOD15', tinhTay: { http: 422, code: 'VOUCHER_MIN_AMOUNT' },
  },
  { id: 'TD-13', tc: 'TC-FOOD-009', mon: [{ name: 'Gà rán', unitPrice: 50_000, quantity: 1 }], km: 3.2, caoDiem: false, mua: false, voucher: 'FOOD15', tinhTay: 69_500 },
  {
    id: 'TD-14', tc: 'TC-FOOD-010', mon: [COM_TAM, TRA_DA], km: 3.2, caoDiem: false, mua: false, vehicle: 'car',
    tinhTay: { http: 400, code: 'VALIDATION_ERROR' },
  },
  { id: 'TD-15', tc: 'TC-FOOD-011', mon: [BUN_BO], km: 4, caoDiem: false, mua: false, tinhTay: 75_000 },
  { id: 'TD-16', tc: 'TC-FOOD-011', mon: [BUN_BO], km: 4, caoDiem: false, mua: false, vehicle: 'bike', tinhTay: 75_000 },
  {
    id: 'TD-17', tc: 'TC-FOOD-012', mon: [{ name: 'Chè', unitPrice: 15_500, quantity: 2 }], km: 3.2, caoDiem: false, mua: false,
    tinhTay: { http: 400, code: 'VALIDATION_ERROR' },
  },
  {
    id: 'TD-18', tc: 'TC-FOOD-012', mon: [{ name: 'Chè', unitPrice: 15_000, quantity: 0 }], km: 3.2, caoDiem: false, mua: false,
    tinhTay: { http: 400, code: 'VALIDATION_ERROR' },
  },
  {
    id: 'TD-19', tc: 'TC-FOOD-012', mon: [{ name: 'Chè', unitPrice: 1_000, quantity: 51 }], km: 3.2, caoDiem: false, mua: false,
    tinhTay: { http: 400, code: 'VALIDATION_ERROR' },
  },
];
