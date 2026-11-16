/**
 * TCD-08 — sổ đăng ký (registry) phía test analysis: test condition và liên kết của chúng tới SPEC,
 * yêu cầu được loại khỏi phạm vi kèm lý do, và defect (GIẢ LẬP, chỉ để minh hoạ truy vết ngược).
 * Test case KHÔNG khai ở đây: chúng được đọc thẳng từ tiêu đề test (tag @R-… @TCOND-…).
 */

export interface TestCondition {
  id: string;
  moTa: string;
  /** Nhiều-nhiều: một test condition có thể truy về nhiều yêu cầu. */
  spec: string[];
  ghiChu?: string;
}

/** Phạm vi RTM của bài: hai nhóm yêu cầu. */
export const NHOM_TRONG_PHAM_VI = ['R-STATE', 'R-CANCEL'] as const;

/** Yêu cầu loại khỏi mẫu số, phải có lý do. */
export const LOAI_TRU: Record<string, string> = {
  'R-STATE-08': 'SPEC ghi rõ «tham khảo, không phải yêu cầu kiểm thử» (bảng đối chiếu trạng thái với kiến trúc đích)',
};

export const TEST_CONDITIONS: TestCondition[] = [
  { id: 'TCOND-STATE-01', moTa: 'Chuyến có đúng 8 trạng thái; completed và cancelled là trạng thái kết thúc', spec: ['R-STATE-01'] },
  { id: 'TCOND-STATE-02', moTa: 'Luồng chính requested → searching → … → completed chạy được (6 chuyển)', spec: ['R-STATE-02'] },
  { id: 'TCOND-STATE-03', moTa: 'Các chuyển huỷ hợp lệ: khách ×3, tài xế ×2, hệ thống ×1', spec: ['R-STATE-02'] },
  { id: 'TCOND-STATE-04', moTa: 'Mọi sự kiện ở trạng thái kết thúc bị từ chối, chuyến giữ nguyên', spec: ['R-STATE-03'] },
  { id: 'TCOND-STATE-05', moTa: 'Sự kiện sai thứ tự hoặc khách huỷ sau khi đón bị từ chối, chuyến giữ nguyên', spec: ['R-STATE-03', 'R-CANCEL-01'] },
  { id: 'TCOND-STATE-06', moTa: 'Tài xế huỷ: về searching, bỏ gán, xoá acceptedAt/ETA, tài xế online, đặt lại searchingSince', spec: ['R-STATE-04', 'R-CANCEL-05'] },
  { id: 'TCOND-STATE-07', moTa: 'Tìm tài xế từ 5 phút ⇒ hệ thống huỷ, lý do no_driver, phí 0 (đúng 5 phút tính là hết)', spec: ['R-STATE-05', 'R-CANCEL-05'] },
  { id: 'TCOND-STATE-08', moTa: 'Nhật ký: tạo chuyến ghi 2 dòng; mỗi chuyển 1 dòng; chuyển bị từ chối không ghi', spec: ['R-STATE-06'] },
  { id: 'TCOND-STATE-09', moTa: 'Mốc thời gian acceptedAt, completedAt, cancelledAt được ghi đúng lúc', spec: ['R-STATE-07'] },
  { id: 'TCOND-CANCEL-01', moTa: 'Khách huỷ khi searching ⇒ 0 đ, lý do searching', spec: ['R-CANCEL-01', 'R-CANCEL-06'] },
  { id: 'TCOND-CANCEL-02', moTa: 'Khách huỷ khi accepted chưa đủ 2 phút ⇒ 0 đ, lý do grace_period', spec: ['R-CANCEL-01', 'R-CANCEL-02', 'R-CANCEL-06'] },
  { id: 'TCOND-CANCEL-03', moTa: 'Biên 2 phút: 1:59,999 miễn phí; đúng 2:00 có phí', spec: ['R-CANCEL-02'] },
  { id: 'TCOND-CANCEL-04', moTa: 'Khách huỷ khi accepted từ 2 phút ⇒ phí theo loại xe', spec: ['R-CANCEL-01', 'R-CANCEL-03'] },
  { id: 'TCOND-CANCEL-05', moTa: 'Khách huỷ khi arriving ⇒ có phí, kể cả trong 2 phút đầu', spec: ['R-CANCEL-01'] },
  { id: 'TCOND-CANCEL-06', moTa: 'Mức phí: xe máy 10.000 đ, ô tô 20.000 đ, đơn đồ ăn 10.000 đ', spec: ['R-CANCEL-03'] },
  { id: 'TCOND-CANCEL-07', moTa: 'Quá mốc nhận + ETA + 10 phút ⇒ 0 đ, lý do driver_late', spec: ['R-CANCEL-04', 'R-CANCEL-06'] },
  { id: 'TCOND-CANCEL-08', moTa: 'Biên tài xế trễ: đúng mốc vẫn thu; quá 1 ms thì miễn', spec: ['R-CANCEL-04'] },
  {
    id: 'TCOND-CANCEL-09',
    moTa: 'Thứ tự ưu tiên lý do khi nhiều lý do miễn cùng đúng',
    spec: ['R-CANCEL-06'],
    ghiChu: 'Với ETA ≥ 1 phút (R-ACCEPT-04), grace_period và driver_late không bao giờ cùng đúng — cần hỏi BA (Q-01)',
  },
  { id: 'TCOND-CANCEL-10', moTa: 'Phí > 0: VíXanh thu với khoá trip:<id>:cancel-fee, tiền mặt ghi nhận; phí lưu vào chuyến', spec: ['R-CANCEL-07'] },
  { id: 'TCOND-CANCEL-11', moTa: 'Tài xế huỷ không bao giờ thu phí khách', spec: ['R-CANCEL-05'] },
];

export interface Defect {
  id: string;
  tieuDe: string;
  testCase: string;
  trangThai: 'mở' | 'đã sửa' | 'đóng';
}

/**
 * GIẢ LẬP — không phải defect thật của src/. Mô phỏng lỗi «arriving trong 2 phút đầu được miễn phí»
 * mà bài TEST-033 (decision table) cài trong buggy-cancel-fee.ts, để minh hoạ truy vết defect → yêu cầu.
 */
export const DEFECTS: Defect[] = [
  { id: 'DEF-SIM-001', tieuDe: 'Khách huỷ ở arriving trong 2 phút đầu không bị thu phí', testCase: 'TC-CANCEL-006', trangThai: 'đóng' },
];
