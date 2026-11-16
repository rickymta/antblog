/**
 * TCD-10 · Thiết kế bộ test case «đặt chuyến từ báo giá» viết thành DỮ LIỆU:
 * test condition (TCOND-BOOK-…), rủi ro (RB-…), test case logic (TC-BOOK-…) kèm kỹ thuật, truy vết,
 * số test case cụ thể dự kiến và phụ thuộc. dat-chuyen.test.ts cài đặt từng ca; rtm.ts sinh RTM từ đây
 * cộng kết quả chạy thật.
 */
import type { Risk, TestCaseSpec } from '../test-case-design-uu-tien-review-bao-tri/bo-test-case';

/** Yêu cầu trong phạm vi bài (test basis chính). */
export const IN_SCOPE = [
  'R-API-08', 'R-QUOTE-03', 'R-QUOTE-04', 'R-VOUCHER-07', 'R-IDEM-01', 'R-IDEM-02',
  'R-IDEM-03', 'R-IDEM-04', 'R-IDEM-05', 'R-VALID-02', 'R-API-13',
] as const;

export interface BookCondition {
  id: string;
  req: string;
  text: string;
}

export const CONDITIONS: BookCondition[] = [
  { id: 'TCOND-BOOK-01', req: 'R-API-08', text: 'Đặt từ báo giá hợp lệ ⇒ 201, searching, giá và bảng chi tiết lấy từ báo giá, mặc định cash' },
  { id: 'TCOND-BOOK-02', req: 'R-API-08', text: 'paymentMethod vixanh ⇒ pending, không gọi ví lúc tạo' },
  { id: 'TCOND-BOOK-03', req: 'R-QUOTE-04', text: 'Giá chuyến là ảnh chụp báo giá dù cao điểm, mưa đã đổi' },
  { id: 'TCOND-BOOK-04', req: 'R-API-08', text: 'Giá do client gửi bị bỏ qua' },
  { id: 'TCOND-BOOK-05', req: 'R-QUOTE-03', text: 'Không có báo giá hoặc báo giá của khách khác ⇒ 422 QUOTE_NOT_FOUND, cùng thông điệp' },
  { id: 'TCOND-BOOK-06', req: 'R-QUOTE-03', text: 'Hết hạn ở đúng expiresAt; còn hạn ngay trước đó' },
  { id: 'TCOND-BOOK-07', req: 'R-QUOTE-03', text: 'Báo giá đã dùng ⇒ 409 QUOTE_ALREADY_USED, kể cả hai yêu cầu đồng thời' },
  { id: 'TCOND-BOOK-08', req: 'R-QUOTE-03', text: 'Thứ tự lỗi báo giá: NOT_FOUND → EXPIRED → ALREADY_USED → VOUCHER_USAGE_LIMIT' },
  { id: 'TCOND-BOOK-09', req: 'R-VOUCHER-07', text: 'Hết lượt khi tạo chuyến ⇒ 422, không tạo chuyến, báo giá không bị tiêu' },
  { id: 'TCOND-BOOK-10', req: 'R-VOUCHER-07', text: 'Chuyến bị huỷ trả lại lượt' },
  { id: 'TCOND-BOOK-11', req: 'R-VOUCHER-07', text: 'Ngày của lượt dùng tính theo thời điểm tạo chuyến, giờ Việt Nam' },
  { id: 'TCOND-BOOK-12', req: 'R-VOUCHER-07', text: 'Biên perCustomerPerDay = 2 (FOOD15)' },
  { id: 'TCOND-BOOK-13', req: 'R-IDEM-01', text: 'Thiếu khoá hoặc sai định dạng (8–128 ký tự, bộ ký tự) ⇒ 400' },
  { id: 'TCOND-BOOK-14', req: 'R-IDEM-02', text: 'Gửi lại cùng nội dung (khác thứ tự khoá, khoảng trắng) ⇒ trả lại phản hồi đầu, không tạo thêm' },
  { id: 'TCOND-BOOK-15', req: 'R-IDEM-02', text: 'Phạm vi khoá theo người gọi' },
  { id: 'TCOND-BOOK-16', req: 'R-IDEM-03', text: 'Cùng khoá, khác nội dung ⇒ 422, không xử lý' },
  { id: 'TCOND-BOOK-17', req: 'R-IDEM-04', text: 'Yêu cầu đầu chưa xong ⇒ 409 IDEMPOTENCY_IN_PROGRESS' },
  { id: 'TCOND-BOOK-18', req: 'R-IDEM-05', text: '4xx của bước xử lý được lưu, kể cả khi nguyên nhân đã hết' },
  { id: 'TCOND-BOOK-19', req: 'R-IDEM-05', text: '5xx không lưu; gửi lại cùng khoá được xử lý như mới' },
  { id: 'TCOND-BOOK-20', req: 'R-IDEM-05', text: '429 không gắn với khoá' },
  { id: 'TCOND-BOOK-21', req: 'R-API-13', text: 'Tối đa 5 yêu cầu/60 giây mỗi khách; thứ 6 ⇒ 429 + Retry-After' },
  { id: 'TCOND-BOOK-22', req: 'R-API-13', text: 'Biên của cửa sổ trượt 60 giây và giá trị Retry-After' },
  { id: 'TCOND-BOOK-23', req: 'R-API-13', text: 'Đếm cả yêu cầu lỗi và gửi lại; không đếm yêu cầu bị 429' },
  { id: 'TCOND-BOOK-24', req: 'R-API-13', text: 'Bộ đếm theo từng khách' },
  { id: 'TCOND-BOOK-25', req: 'R-VALID-02', text: 'Thứ tự lỗi 401 → 403 → 429 → idempotency → định dạng → báo giá' },
  { id: 'TCOND-BOOK-26', req: 'R-STATE-06', text: 'Tạo chuyến ghi 2 dòng nhật ký requested, searching' },
];

export const RISKS: Risk[] = [
  { id: 'RB-01', text: 'Gửi lại hoặc gửi đồng thời tạo chuyến trùng, đặt hai chuyến từ một báo giá', likelihood: 3, impact: 3 },
  { id: 'RB-02', text: 'Giá chuyến khác giá đã báo, hoặc client chỉnh được giá', likelihood: 2, impact: 3 },
  { id: 'RB-03', text: 'Khách dùng voucher vượt số lượt', likelihood: 2, impact: 2 },
  { id: 'RB-04', text: 'Khách hợp lệ bị chặn sai (khoá, tần suất, lỗi đã lưu, lượt chưa trả)', likelihood: 2, impact: 2 },
  { id: 'RB-05', text: 'Lộ sự tồn tại báo giá/chuyến của khách khác', likelihood: 1, impact: 3 },
  { id: 'RB-06', text: 'Đặt được từ báo giá hết hạn', likelihood: 2, impact: 2 },
  { id: 'RB-07', text: 'Tạo chuyến hàng loạt không bị giới hạn', likelihood: 2, impact: 2 },
  { id: 'RB-08', text: 'Mã lỗi sai thứ tự khiến client xử lý sai', likelihood: 2, impact: 1 },
  { id: 'RB-09', text: 'Trạng thái thanh toán ban đầu sai', likelihood: 1, impact: 2 },
];

export interface BookTestCase extends TestCaseSpec {
  technique: string;
  /** Số test case cụ thể (dòng dữ liệu) dự kiến. */
  concrete: number;
  /** Defect đã biết làm ca này đỏ theo SPEC (chạy it.fails). */
  knownDefect?: string;
}

const SMOKE = ['TC-BOOK-001'];

const tc = (
  id: string, title: string, technique: string, reqs: string[], covers: string[], risks: string[], concrete = 1, knownDefect?: string,
): BookTestCase => ({ id, title, technique, reqs, covers, risks, concrete, dependsOn: id === 'TC-BOOK-001' ? [] : SMOKE, knownDefect });

export const TEST_CASES: BookTestCase[] = [
  tc('TC-BOOK-001', 'Đặt từ báo giá hợp lệ, mặc định cash: chuyến, giá, nhật ký, ví', 'EP (ca dương)', ['R-API-08', 'R-STATE-06'], ['TCOND-BOOK-01', 'TCOND-BOOK-26'], ['RB-02']),
  tc('TC-BOOK-002', 'paymentMethod vixanh ⇒ pending, không gọi ví', 'EP', ['R-API-08'], ['TCOND-BOOK-02'], ['RB-09']),
  tc('TC-BOOK-003', 'Báo giá 12:58 cao điểm + mưa + GIAM20, đặt 13:01 khi hết mưa ⇒ giữ 52.000 đ', 'Decision table (ảnh chụp giá)', ['R-QUOTE-04'], ['TCOND-BOOK-03'], ['RB-02']),
  tc('TC-BOOK-004', 'Client gửi fare/total/discount ⇒ bị bỏ qua', 'Error guessing', ['R-API-08', 'R-API-14'], ['TCOND-BOOK-04'], ['RB-02']),
  tc('TC-BOOK-010', 'Báo giá không tồn tại và báo giá của C002 ⇒ cùng 422 QUOTE_NOT_FOUND', 'EP (ca âm)', ['R-QUOTE-03', 'R-API-05'], ['TCOND-BOOK-05'], ['RB-05']),
  tc('TC-BOOK-011', 'Hạn báo giá: 4:59,999 ⇒ 201; 5:00,000 ⇒ 422 QUOTE_EXPIRED', 'BVA 2 giá trị', ['R-QUOTE-03', 'R-QUOTE-01'], ['TCOND-BOOK-06'], ['RB-06'], 2),
  tc('TC-BOOK-012', 'Báo giá đã dùng, khoá khác ⇒ 409 QUOTE_ALREADY_USED, vẫn một chuyến', 'EP (ca âm)', ['R-QUOTE-03'], ['TCOND-BOOK-07'], ['RB-01']),
  tc('TC-BOOK-013', 'Hai yêu cầu đồng thời, cùng báo giá, khác khoá ⇒ một 201, một 409', 'Error guessing (đồng thời)', ['R-QUOTE-03'], ['TCOND-BOOK-07'], ['RB-01']),
  tc('TC-BOOK-014', 'Thứ tự lỗi báo giá: ba cặp lỗi liền kề', 'Cặp vi phạm liền kề', ['R-QUOTE-03'], ['TCOND-BOOK-08'], ['RB-08'], 3),
  tc('TC-BOOK-020', 'GIAM20 đã dùng hôm nay ⇒ 422 VOUCHER_USAGE_LIMIT, không tạo chuyến', 'EP (ca âm)', ['R-VOUCHER-07', 'R-QUOTE-03'], ['TCOND-BOOK-09'], ['RB-03']),
  tc('TC-BOOK-021', 'Huỷ chuyến đã dùng GIAM20 ⇒ báo giá thứ hai đặt được, giảm 9.060 đ', 'State transition (lượt)', ['R-VOUCHER-07'], ['TCOND-BOOK-10', 'TCOND-BOOK-09'], ['RB-04']),
  tc('TC-BOOK-022', 'Nửa đêm giờ VN: 23:59:59,999 ⇒ 422; 00:00:00,000 ⇒ 201', 'BVA 2 giá trị (ngày)', ['R-VOUCHER-07'], ['TCOND-BOOK-11'], ['RB-03', 'RB-04'], 2),
  tc('TC-BOOK-023', 'FOOD15 (2 lần/ngày): chuyến 1, 2 ⇒ 201; chuyến 3 ⇒ 422', 'BVA (số lượt)', ['R-VOUCHER-07'], ['TCOND-BOOK-12'], ['RB-03']),
  tc('TC-BOOK-030', 'Thiếu Idempotency-Key ⇒ 400, báo giá không bị tiêu', 'EP (ca âm)', ['R-IDEM-01'], ['TCOND-BOOK-13'], ['RB-01']),
  tc('TC-BOOK-031', 'Độ dài khoá 7 / 8 / 128 / 129', 'BVA 2 giá trị', ['R-IDEM-01'], ['TCOND-BOOK-13'], ['RB-04'], 4),
  tc('TC-BOOK-032', 'Bộ ký tự khoá: A–Z a–z 0–9 _ . : - hợp lệ; khoảng trắng, /, * không', 'EP', ['R-IDEM-01'], ['TCOND-BOOK-13'], ['RB-04'], 4),
  tc('TC-BOOK-033', 'Gửi lại cùng nội dung (đảo khoá, thêm khoảng trắng) ⇒ 201 replayed, một chuyến, 2 dòng nhật ký', 'State transition (bản ghi khoá)', ['R-IDEM-02'], ['TCOND-BOOK-14'], ['RB-01']),
  tc('TC-BOOK-034', 'C002 dùng cùng khoá với C001 ⇒ xử lý riêng, không trả chuyến của C001', 'EP', ['R-IDEM-02'], ['TCOND-BOOK-15'], ['RB-05']),
  tc('TC-BOOK-035', 'Cùng khoá, đổi paymentMethod ⇒ 422 IDEMPOTENCY_KEY_REUSED, chuyến cũ giữ cash', 'EP (ca âm)', ['R-IDEM-03'], ['TCOND-BOOK-16'], ['RB-01']),
  tc('TC-BOOK-036', 'Yêu cầu thứ hai cùng khoá khi yêu cầu đầu chưa xong ⇒ 409', 'Error guessing (đồng thời)', ['R-IDEM-04'], ['TCOND-BOOK-17'], ['RB-01']),
  tc('TC-BOOK-037', 'VOUCHER_USAGE_LIMIT đã lưu: huỷ chuyến kia rồi gửi lại cùng khoá ⇒ vẫn 422 replayed', 'State transition (bản ghi khoá)', ['R-IDEM-05', 'R-VOUCHER-07'], ['TCOND-BOOK-18'], ['RB-04']),
  tc('TC-BOOK-038', '500 không lưu: gửi lại cùng khoá ⇒ 201', 'Error guessing (lỗi hạ tầng)', ['R-IDEM-05'], ['TCOND-BOOK-19'], ['RB-04'], 1, 'DR-050-01'),
  tc('TC-BOOK-039', 'Yêu cầu bị 429 với khoá K; sau cửa sổ, gửi K ⇒ 201, không replay', 'State transition (bản ghi khoá)', ['R-IDEM-05', 'R-API-13'], ['TCOND-BOOK-20'], ['RB-04']),
  tc('TC-BOOK-050', '5 yêu cầu cùng lúc ⇒ 201; thứ 6 ⇒ 429, Retry-After 60', 'BVA (số yêu cầu)', ['R-API-13'], ['TCOND-BOOK-21'], ['RB-07']),
  tc('TC-BOOK-051', 'Cửa sổ trượt: yêu cầu thứ 6 ở +59,999 giây ⇒ 429, Retry-After 1; ở +60,000 giây ⇒ 201', 'BVA 2 giá trị (thời gian)', ['R-API-13'], ['TCOND-BOOK-22'], ['RB-07', 'RB-04'], 2),
  tc('TC-BOOK-052', '1 tạo + 4 gửi lại ⇒ yêu cầu thứ 6 bị 429', 'EP', ['R-API-13'], ['TCOND-BOOK-23'], ['RB-07']),
  tc('TC-BOOK-053', '5 yêu cầu ở 0 giây, 5 yêu cầu bị 429 ở 30 giây ⇒ ở 60 giây vẫn đặt được', 'EP', ['R-API-13'], ['TCOND-BOOK-23'], ['RB-04']),
  tc('TC-BOOK-054', 'C001 bị 429 ⇒ C002 cùng lúc vẫn 201', 'EP', ['R-API-13'], ['TCOND-BOOK-24'], ['RB-04']),
  tc('TC-BOOK-060', 'Thứ tự lỗi tạo chuyến: sáu cặp vi phạm', 'Cặp vi phạm liền kề', ['R-VALID-02'], ['TCOND-BOOK-25'], ['RB-08'], 6),
  tc('TC-BOOK-061', 'JSON hỏng, không có x-user ⇒ 401', 'Cặp vi phạm liền kề', ['R-VALID-02', 'R-API-01'], ['TCOND-BOOK-25'], ['RB-08'], 1, 'DR-050-02'),
  tc('TC-BOOK-062', 'Khách đã hết lượt tần suất gửi JSON hỏng ⇒ 429', 'Cặp vi phạm liền kề', ['R-VALID-02', 'R-API-13'], ['TCOND-BOOK-25'], ['RB-08'], 1, 'DR-TCD10-01'),
];

/** Test data dùng chung (TD-…). Giá tính tay theo R-FARE, không gọi mã sản phẩm. */
export const TD = {
  /** TD-01 · SG, xe máy, 7,4 km, 09:00 giờ VN, không mưa: 12.000 + 4.500 × 7,4 = 45.300 ⇒ 46.000 đ. */
  RIDE: { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 },
  RIDE_FARE: 46_000,
  /** TD-02 · như TD-01 + GIAM20: giảm 20% × 45.300 = 9.060 ⇒ 36.240 ⇒ 37.000 đ. */
  RIDE_GIAM20: { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4, voucherCode: 'GIAM20' },
  RIDE_GIAM20_FARE: 37_000,
  RIDE_GIAM20_DISCOUNT: 9_060,
  /** TD-03 · TD-02 lúc 12:58, SG mưa: 45.300 × 1,25 = 56.625 + 8.000 = 64.625 − 12.925 = 51.700 ⇒ 52.000 đ (R-FARE-09). */
  PEAK_RAIN_FARE: 52_000,
  PEAK_RAIN_DISCOUNT: 12_925,
  /** TD-04 · Đồ ăn SG 3 km, một món 60.000 đ, FOOD15: phí giao 25.500 ⇒ 26.000; giảm 9.000; tổng 77.000 đ. */
  FOOD: { kind: 'food', cityCode: 'SG', distanceKm: 3, items: [{ name: 'Cơm tấm sườn', unitPrice: 60_000, quantity: 1 }], voucherCode: 'FOOD15' },
  FOOD_TOTAL: 77_000,
} as const;
