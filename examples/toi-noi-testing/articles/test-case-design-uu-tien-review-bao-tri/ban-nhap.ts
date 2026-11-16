/**
 * TCD-09 · Tám test case ở dạng BẢN NHÁP (cài sẵn anti-pattern) và bản đã sửa sau review.
 * Dùng cho review-tu-dong.ts: phần checklist mà máy kiểm được. Phần còn lại cần người đọc.
 */

export interface WrittenTestCase {
  id: string;
  title: string;
  objectives: string[];
  reqs: string[];
  covers: string[];
  dependsOn: string[];
  preconditions: string[];
  data: string[];
  steps: string[];
  expected: string[];
}

const TRIP_VIXANH = 'Chuyến SG xe máy 7,4 km, vixanh, đặt lúc T0 = 04/11/2026 09:00 giờ VN (+07:00), đồng hồ giả';

export const BAN_NHAP: WrittenTestCase[] = [
  {
    id: 'TC-PAY-003',
    title: 'Ví lỗi',
    objectives: ['Kiểm thử lại khi ví lỗi'],
    reqs: ['R-PAY-04'],
    covers: ['TCOND-PAY-03'],
    dependsOn: [],
    preconditions: [TRIP_VIXANH],
    data: ['Ví chế độ error'],
    steps: ['Tài xế hoàn thành chuyến'],
    expected: ['Hệ thống xử lý lỗi đúng và thử lại theo quy định'],
  },
  {
    id: 'TC-PAY-004',
    title: 'Ví từ chối, lỗi tạm thời và hết giờ',
    objectives: ['Ví từ chối không thử lại', 'Lỗi tạm thời thử lại', 'Hết giờ thử lại'],
    reqs: ['R-PAY-04'],
    covers: ['TCOND-PAY-05', 'TCOND-PAY-03', 'TCOND-PAY-02'],
    dependsOn: [],
    preconditions: [TRIP_VIXANH],
    data: ['Ví decline, rồi error, rồi timeout'],
    steps: ['Hoàn thành chuyến với ví decline', 'Đổi ví sang error, hoàn thành chuyến thứ hai', 'Đổi ví sang timeout, hoàn thành chuyến thứ ba'],
    expected: ['Chuyến 1: 1 lần gọi, `failed`', 'Chuyến 2: 3 lần gọi, `failed`', 'Chuyến 3: 3 lần gọi, `failed`'],
  },
  {
    id: 'TC-IDEM-007',
    title: 'Gửi lại lệnh huỷ',
    objectives: ['Gửi lại lệnh huỷ có phí không thu tiền lần hai'],
    reqs: ['R-IDEM-02'],
    covers: ['TCOND-IDEM-09'],
    dependsOn: [],
    preconditions: ['Chuyến đã huỷ ở ca trước'],
    data: ['Khoá huy-0001'],
    steps: ['Gửi lại yêu cầu huỷ của ca trước với cùng khoá'],
    expected: ['HTTP 200, header `Idempotent-Replayed: true`, ví có đúng 1 lần gọi, trừ 10.000 đ'],
  },
  {
    id: 'TC-PAY-006',
    title: 'Huỷ có phí qua VíXanh',
    objectives: ['Phí huỷ > 0 với chuyến vixanh được thu bằng khoá cancel-fee'],
    reqs: ['R-PAY-05'],
    covers: ['TCOND-PAY-07'],
    dependsOn: [],
    preconditions: ['Chuyến vixanh, D101 nhận lúc 09:00 ngày 04/11/2026, ETA 5'],
    data: ['Khách huỷ lúc 09:05 ngày 04/11/2026'],
    steps: ['Chờ 5 phút sau khi tài xế nhận', 'Khách huỷ'],
    expected: ['HTTP 200, `cancellationFee` = 10000, `feeReason` = `charged`, khoá `trip:<id>:cancel-fee`, `paymentStatus` = `paid`'],
  },
  {
    id: 'TC-PAY-002',
    title: 'Ví hết giờ hai lần',
    objectives: ['Hết giờ thì thử lại cùng khoá'],
    reqs: ['R-PAY-04'],
    covers: ['TCOND-PAY-02'],
    dependsOn: [],
    preconditions: [TRIP_VIXANH],
    data: ['Ví: timeout, timeout, rồi bình thường'],
    steps: ['Tài xế hoàn thành chuyến'],
    expected: ['HTTP 200, `paymentStatus` = `paid`, 3 lần gọi cùng khoá `trip:<id>:fare`, ví trừ 46.000 đ một lần'],
  },
  {
    id: 'TC-PAY-005',
    title: 'Ví mất phản hồi',
    objectives: ['Ví đã trừ nhưng mất phản hồi'],
    reqs: ['R-PAY-04'],
    covers: ['TCOND-PAY-06'],
    dependsOn: [],
    preconditions: [TRIP_VIXANH],
    data: ['Ví: timeout, timeout, rồi bình thường'], // chép từ TC-PAY-002, quên đổi chế độ lỗi
    steps: ['Tài xế hoàn thành chuyến'],
    expected: ['HTTP 200, `paymentStatus` = `paid`, 3 lần gọi cùng khoá `trip:<id>:fare`, ví trừ 46.000 đ một lần'],
  },
  {
    id: 'TC-IDEM-002',
    title: 'Gửi lại tạo chuyến',
    objectives: ['Gửi lại cùng thân không tạo chuyến mới'],
    reqs: ['R-IDEM-02'],
    covers: ['TCOND-IDEM-02'],
    dependsOn: [],
    preconditions: ['Báo giá còn hạn của C001'],
    data: ['Khoá dat-chuyen-0001'],
    steps: ['POST /api/trips', 'POST /api/trips lần hai, cùng khoá, cùng thân'],
    expected: ['`MemoryIdempotencyStore` có đúng 1 bản ghi; `fingerprint` là SHA-256 của thân'],
  },
  {
    id: 'TC-PAY-009',
    title: 'Thanh toán nhanh',
    objectives: ['Thanh toán đủ nhanh'],
    reqs: [],
    covers: [],
    dependsOn: [],
    preconditions: [TRIP_VIXANH],
    data: ['Ví bình thường'],
    steps: ['Tài xế hoàn thành chuyến'],
    expected: ['Thanh toán nhanh, người dùng hài lòng'],
  },
  {
    id: 'TC-IDEM-006',
    title: 'Lỗi 500 không được lưu',
    objectives: ['Phản hồi 5xx không được lưu'],
    reqs: ['R-IDEM-05'],
    covers: ['TCOND-IDEM-07'],
    dependsOn: [],
    preconditions: ['Báo giá còn hạn của C001; kho chuyến lỗi đúng một lần'],
    data: ['Khoá dat-chuyen-0500'],
    steps: ['POST /api/trips', 'Gửi lại cùng khoá, cùng thân'],
    expected: ['Lần gửi lại không phải replay'],
  },
];

/** Bản sau review: mỗi ca sửa đúng chỗ bị nêu. TC-PAY-004 tách thành ba ca, TC-PAY-009 bị loại khỏi bộ. */
export const BAN_SUA: WrittenTestCase[] = [
  {
    ...BAN_NHAP[0]!,
    title: 'Ví lỗi tạm thời liên tục ⇒ failed sau đúng 3 lần gọi, chuyến vẫn completed',
    objectives: ['Lỗi tạm thời được thử lại tối đa 2 lần rồi ghi failed mà không chặn hoàn thành'],
    reqs: ['R-PAY-04', 'R-PAY-06'],
    covers: ['TCOND-PAY-03', 'TCOND-PAY-04', 'TCOND-PAY-10'],
    expected: ['HTTP 200, `status` = `completed`, `paymentStatus` = `failed`', 'Ví nhận đúng 3 lần gọi, cùng khoá `trip:<id>:fare`, trừ 0 đ'],
  },
  {
    ...BAN_NHAP[1]!,
    title: 'Ví từ chối ⇒ failed sau 1 lần gọi, chuyến vẫn completed',
    objectives: ['Bị từ chối thì không thử lại'],
    reqs: ['R-PAY-04', 'R-PAY-06'],
    covers: ['TCOND-PAY-05', 'TCOND-PAY-10'],
    data: ['Ví chế độ decline'],
    steps: ['Tài xế hoàn thành chuyến'],
    expected: ['HTTP 200, `status` = `completed`, `paymentStatus` = `failed`', 'Ví nhận đúng 1 lần gọi, trừ 0 đ'],
  },
  {
    ...BAN_NHAP[2]!,
    title: 'Gửi lại lệnh huỷ của TC-PAY-006 ⇒ 200 replayed, ví không bị gọi thêm',
    reqs: ['R-IDEM-02', 'R-PAY-05'],
    covers: ['TCOND-IDEM-02', 'TCOND-IDEM-09'],
    dependsOn: ['TC-PAY-006'],
    preconditions: ['Hậu điều kiện của TC-PAY-006: chuyến đã huỷ, khoá huy-0001, ví đã trừ 10.000 đ'],
    steps: ['Gửi lại POST /api/trips/<id>/cancel với khoá huy-0001, thân {}'],
  },
  {
    ...BAN_NHAP[3]!,
    preconditions: ['Chuyến vixanh, D101 nhận tại T0 = 04/11/2026 09:00 giờ VN (+07:00), ETA 5'],
    data: ['Khách huỷ tại T0 + 5 phút (đồng hồ giả)'],
    steps: ['Dời đồng hồ giả tới T0 + 5 phút', 'Khách huỷ, khoá huy-0001'],
  },
  BAN_NHAP[4]!,
  {
    ...BAN_NHAP[5]!,
    title: 'Ví đã trừ nhưng mất phản hồi, lần sau trả lời ⇒ paid, trừ đúng một lần',
    covers: ['TCOND-PAY-02', 'TCOND-PAY-06'],
    data: ['Ví: lost_response, rồi bình thường'],
    expected: ['HTTP 200, `paymentStatus` = `paid`, 2 lần gọi cùng khoá `trip:<id>:fare`, ví trừ 46.000 đ đúng một lần'],
  },
  {
    ...BAN_NHAP[6]!,
    covers: ['TCOND-IDEM-02', 'TCOND-IDEM-03'],
    data: ['Khoá dat-chuyen-0001; thân lần hai đảo thứ tự khoá JSON'],
    expected: [
      'Hai phản hồi 201 cùng `trip.id`; lần hai có header `Idempotent-Replayed: true`',
      'Danh sách chuyến chờ của D101 có đúng 1 chuyến',
    ],
  },
  {
    ...BAN_NHAP[8]!,
    title: '500 không được lưu: gửi lại cùng khoá ⇒ được xử lý như mới (201)',
    expected: ['Lần đầu 500 `INTERNAL_ERROR`; lần gửi lại không có header `Idempotent-Replayed` và trả 201 với chuyến mới'],
  },
];
