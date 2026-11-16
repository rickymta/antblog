/**
 * TCD-09 · Bộ test case R-PAY + R-IDEM viết thành DỮ LIỆU để áp các chiến lược ưu tiên của CTFL 5.1.5.
 *
 * - Coverage item = test condition (TCOND-…) rút từ SPEC R-PAY-03…06 và R-IDEM-01…05.
 * - Rủi ro (RR-…): khả năng × tác động, thang 1–3 — quy ước bài tập, không phải con số của CTFL.
 * - Ưu tiên yêu cầu (1 cao nhất) do PO đặt — GIẢ ĐỊNH BÀI TẬP, SPEC không có.
 * - `dependsOn`: phụ thuộc ĐÃ KHAI BÁO của test procedure chạy tay trên môi trường staging.
 */

export interface TestCondition {
  id: string;
  req: string;
  text: string;
}

export interface Risk {
  id: string;
  text: string;
  likelihood: 1 | 2 | 3;
  impact: 1 | 2 | 3;
}

export interface TestCaseSpec {
  id: string;
  title: string;
  reqs: string[];
  risks: string[];
  covers: string[];
  dependsOn: string[];
}

export const CONDITIONS: TestCondition[] = [
  { id: 'TCOND-PAY-01', req: 'R-PAY-03', text: 'Hoàn thành chuyến vixanh ⇒ thu đúng fare, khoá trip:<id>:fare, paid' },
  { id: 'TCOND-PAY-02', req: 'R-PAY-04', text: 'Hết giờ ⇒ thử lại với CÙNG khoá' },
  { id: 'TCOND-PAY-03', req: 'R-PAY-04', text: 'Lỗi tạm thời ⇒ thử lại' },
  { id: 'TCOND-PAY-04', req: 'R-PAY-04', text: 'Tối đa 3 lần gọi rồi dừng' },
  { id: 'TCOND-PAY-05', req: 'R-PAY-04', text: 'Bị từ chối ⇒ không thử lại' },
  { id: 'TCOND-PAY-06', req: 'R-PAY-04', text: 'Cổng đã trừ nhưng mất phản hồi ⇒ thử lại không trừ lần hai' },
  { id: 'TCOND-PAY-07', req: 'R-PAY-05', text: 'Phí huỷ > 0, vixanh ⇒ thu với khoá trip:<id>:cancel-fee' },
  { id: 'TCOND-PAY-08', req: 'R-PAY-05', text: 'Phí huỷ > 0, cash ⇒ paymentStatus cash, không gọi ví' },
  { id: 'TCOND-PAY-09', req: 'R-PAY-05', text: 'Phí huỷ 0 ⇒ paymentStatus none, không gọi ví' },
  { id: 'TCOND-PAY-10', req: 'R-PAY-06', text: 'Thu thất bại không chặn hoàn thành chuyến; failed; HTTP 200' },
  { id: 'TCOND-IDEM-01', req: 'R-IDEM-01', text: 'Thiếu hoặc sai định dạng khoá ⇒ 400 IDEMPOTENCY_KEY_REQUIRED' },
  { id: 'TCOND-IDEM-02', req: 'R-IDEM-02', text: 'Cùng khoá + cùng thân ⇒ trả lại đúng phản hồi đầu, Idempotent-Replayed' },
  { id: 'TCOND-IDEM-03', req: 'R-IDEM-02', text: 'Đổi thứ tự khoá JSON vẫn là cùng nội dung' },
  { id: 'TCOND-IDEM-04', req: 'R-IDEM-03', text: 'Cùng khoá + khác thân ⇒ 422 IDEMPOTENCY_KEY_REUSED' },
  { id: 'TCOND-IDEM-05', req: 'R-IDEM-04', text: 'Yêu cầu đầu chưa xong ⇒ 409 IDEMPOTENCY_IN_PROGRESS' },
  { id: 'TCOND-IDEM-06', req: 'R-IDEM-05', text: 'Phản hồi 4xx do bước xử lý tạo ra được lưu và trả lại' },
  { id: 'TCOND-IDEM-07', req: 'R-IDEM-05', text: 'Phản hồi 5xx không được lưu; gửi lại được xử lý như mới' },
  { id: 'TCOND-IDEM-08', req: 'R-IDEM-02', text: 'Phạm vi khoá là (người gọi, endpoint): khách khác dùng cùng khoá không bị ảnh hưởng' },
  { id: 'TCOND-IDEM-09', req: 'R-IDEM-02', text: 'Gửi lại lệnh huỷ có phí không thu tiền lần hai' },
];

export const RISKS: Risk[] = [
  { id: 'RR-01', text: 'Trừ tiền khách hai lần khi thử lại hoặc gửi lại', likelihood: 2, impact: 3 },
  { id: 'RR-02', text: 'Gửi lại yêu cầu tạo ra chuyến trùng', likelihood: 3, impact: 2 },
  { id: 'RR-03', text: 'Ví lỗi làm kẹt chuyến hoặc thử lại sai luật', likelihood: 2, impact: 2 },
  { id: 'RR-04', text: 'Khoá dùng lại hoặc gửi đồng thời bị xử lý như yêu cầu mới', likelihood: 2, impact: 2 },
  { id: 'RR-05', text: 'Thu sai số tiền hoặc sai khoá idempotency', likelihood: 1, impact: 3 },
  { id: 'RR-06', text: 'Lỗi tạm thời bị lưu thành kết quả vĩnh viễn', likelihood: 1, impact: 3 },
  { id: 'RR-07', text: 'Trạng thái thanh toán của phí huỷ sai (cash/none)', likelihood: 2, impact: 1 },
  { id: 'RR-08', text: 'Thiếu khoá hoặc lỗi đã lưu được xử lý khác đặc tả', likelihood: 2, impact: 1 },
];

/** Ưu tiên yêu cầu do PO đặt (1 = cao nhất). Giả định bài tập. */
export const REQUIREMENT_PRIORITY: Record<string, 1 | 2 | 3> = {
  'R-PAY-04': 1,
  'R-IDEM-02': 1,
  'R-PAY-03': 2,
  'R-PAY-06': 2,
  'R-IDEM-03': 2,
  'R-IDEM-04': 2,
  'R-PAY-05': 3,
  'R-IDEM-01': 3,
  'R-IDEM-05': 3,
};

export const TEST_CASES: TestCaseSpec[] = [
  { id: 'TC-PAY-001', title: 'Hoàn thành chuyến vixanh, ví bình thường ⇒ paid, 1 lần gọi', reqs: ['R-PAY-03'], risks: ['RR-05'], covers: ['TCOND-PAY-01'], dependsOn: [] },
  { id: 'TC-PAY-002', title: 'Ví hết giờ, hết giờ, rồi trả lời ⇒ paid ở lần gọi thứ 3, cùng khoá', reqs: ['R-PAY-04'], risks: ['RR-01'], covers: ['TCOND-PAY-02'], dependsOn: [] },
  { id: 'TC-PAY-003', title: 'Ví lỗi tạm thời liên tục ⇒ failed sau đúng 3 lần gọi, chuyến vẫn completed', reqs: ['R-PAY-04', 'R-PAY-06'], risks: ['RR-03'], covers: ['TCOND-PAY-03', 'TCOND-PAY-04', 'TCOND-PAY-10'], dependsOn: [] },
  { id: 'TC-PAY-004', title: 'Ví từ chối ⇒ failed sau 1 lần gọi, chuyến vẫn completed', reqs: ['R-PAY-04', 'R-PAY-06'], risks: ['RR-03'], covers: ['TCOND-PAY-05', 'TCOND-PAY-10'], dependsOn: [] },
  { id: 'TC-PAY-005', title: 'Ví đã trừ nhưng mất phản hồi, lần sau trả lời ⇒ paid, trừ đúng một lần', reqs: ['R-PAY-04'], risks: ['RR-01'], covers: ['TCOND-PAY-02', 'TCOND-PAY-06'], dependsOn: [] },
  { id: 'TC-PAY-006', title: 'Khách huỷ chuyến vixanh có phí ⇒ thu 10.000 đ khoá cancel-fee', reqs: ['R-PAY-05'], risks: ['RR-05'], covers: ['TCOND-PAY-07'], dependsOn: [] },
  { id: 'TC-PAY-007', title: 'Khách huỷ chuyến cash có phí ⇒ paymentStatus cash, không gọi ví', reqs: ['R-PAY-05'], risks: ['RR-07'], covers: ['TCOND-PAY-08'], dependsOn: [] },
  { id: 'TC-PAY-008', title: 'Khách huỷ chuyến vixanh khi searching ⇒ 0 đ, none, không gọi ví', reqs: ['R-PAY-05'], risks: ['RR-07'], covers: ['TCOND-PAY-09'], dependsOn: [] },
  { id: 'TC-IDEM-001', title: 'Thiếu khoá / khoá 7 ký tự / ký tự lạ ⇒ 400, không tạo chuyến', reqs: ['R-IDEM-01'], risks: ['RR-08'], covers: ['TCOND-IDEM-01'], dependsOn: [] },
  { id: 'TC-IDEM-002', title: 'Gửi lại cùng thân (đảo thứ tự khoá) ⇒ 201 replayed, một chuyến', reqs: ['R-IDEM-02'], risks: ['RR-02'], covers: ['TCOND-IDEM-02', 'TCOND-IDEM-03'], dependsOn: [] },
  { id: 'TC-IDEM-003', title: 'Cùng khoá, khác thân ⇒ 422 IDEMPOTENCY_KEY_REUSED', reqs: ['R-IDEM-03'], risks: ['RR-04'], covers: ['TCOND-IDEM-04'], dependsOn: [] },
  { id: 'TC-IDEM-004', title: 'Yêu cầu thứ hai khi yêu cầu đầu chưa xong ⇒ 409 IDEMPOTENCY_IN_PROGRESS', reqs: ['R-IDEM-04'], risks: ['RR-04'], covers: ['TCOND-IDEM-05'], dependsOn: [] },
  { id: 'TC-IDEM-005', title: 'QUOTE_EXPIRED được lưu: gửi lại ⇒ 422 replayed', reqs: ['R-IDEM-05'], risks: ['RR-08'], covers: ['TCOND-IDEM-06'], dependsOn: [] },
  { id: 'TC-IDEM-006', title: '500 không được lưu: gửi lại cùng khoá ⇒ được xử lý như mới (201)', reqs: ['R-IDEM-05'], risks: ['RR-06'], covers: ['TCOND-IDEM-07'], dependsOn: [] },
  { id: 'TC-IDEM-007', title: 'Gửi lại lệnh huỷ của TC-PAY-006 ⇒ 200 replayed, ví không bị gọi thêm', reqs: ['R-IDEM-02', 'R-PAY-05'], risks: ['RR-01'], covers: ['TCOND-IDEM-02', 'TCOND-IDEM-09'], dependsOn: ['TC-PAY-006'] },
];
