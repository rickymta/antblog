/**
 * Sinh bộ collection + environment Postman v2.1 cho lab Tới Nơi, dùng trong CurlStudio (hoặc Postman).
 *   node curlstudio/build-collection.mjs            (chạy từ examples/toi-noi-testing)
 * Ghi: toi-noi-lab.postman_collection.json, toi-noi-lab-local.postman_environment.json,
 *      toi-noi-prod-like.postman_environment.json, data/bang-cuoc.csv, data/che-do-vi.json
 * Giá trị mong đợi trong test script và bảng dữ liệu được tính tay theo SPEC (R-FARE, R-VOUCHER) — không gọi lại mã của lab.
 */
import { mkdirSync, writeFileSync } from 'node:fs';

const out = new URL('./', import.meta.url);
mkdirSync(new URL('data/', out), { recursive: true });

const json = (o) => ({ mode: 'raw', raw: JSON.stringify(o, null, 2), options: { raw: { language: 'json' } } });
const url = (path) => ({ raw: `{{baseUrl}}${path}`, host: ['{{baseUrl}}'], path: path.split('/').filter(Boolean) });
const tests = (...lines) => [{ listen: 'test', script: { type: 'text/javascript', exec: lines } }];
const req = (name, method, path, { user, body, key, test = [], description } = {}) => ({
  name,
  ...(test.length ? { event: tests(...test) } : {}),
  request: {
    method,
    header: [
      ...(body !== undefined ? [{ key: 'Content-Type', value: 'application/json' }] : []),
      ...(user ? [{ key: 'x-user', value: user }] : []),
      ...(key ? [{ key: 'Idempotency-Key', value: key }] : []),
    ],
    ...(body !== undefined ? { body: json(body) } : {}),
    url: url(path),
    ...(description ? { description } : {}),
  },
});
const folder = (name, description, item) => ({ name, description, item });

const QUOTE_5KM = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 };
const status = (code) => `pm.test('HTTP ${code}', () => pm.response.to.have.status(${code}));`;
const errorCode = (code) => `pm.test('Mã lỗi ${code}', () => pm.expect(pm.response.json().error.code).to.eql('${code}'));`;

const setup = [
  req('Reset dữ liệu mẫu', 'POST', '/api/test/reset', { test: [status(200)] }),
  req('Đặt đồng hồ 09:00 ngày 04/11/2026', 'POST', '/api/test/clock', { body: { now: '2026-11-04T09:00:00+07:00' }, test: [status(200)] }),
  req('Trời không mưa', 'POST', '/api/test/weather', { body: { raining: [] }, test: [status(200)] }),
  req('Ví VíXanh bình thường', 'POST', '/api/test/payment', { body: { mode: 'none' }, test: [status(200)] }),
];

const driverStep = (name, step, extra = {}) =>
  req(name, 'POST', `/api/trips/{{tripId}}/${step}`, { user: 'driver:D101', body: extra, test: [status(200)] });

const bookingJourney = (prefix, paymentMethod) => [
  req(`${prefix} Báo giá xe máy 5 km`, 'POST', '/api/quotes', {
    user: 'customer:C001', body: QUOTE_5KM,
    test: [status(200), "const q = pm.response.json().quote;", "pm.test('Cước 35.000 đ (R-FARE-01/08)', () => pm.expect(q.total).to.eql(35000));", "pm.collectionVariables.set('quoteId', q.quoteId);"],
  }),
  req(`${prefix} Đặt chuyến (${paymentMethod})`, 'POST', '/api/trips', {
    user: 'customer:C001', key: '{{$guid}}', body: { quoteId: '{{quoteId}}', paymentMethod },
    test: [status(201), "const t = pm.response.json().trip;", "pm.test('Chuyến ở trạng thái searching', () => pm.expect(t.status).to.eql('searching'));", "pm.collectionVariables.set('tripId', t.id);"],
  }),
  driverStep(`${prefix} Tài xế D101 nhận chuyến`, 'accept', { etaMinutes: 5 }),
  driverStep(`${prefix} Tài xế tới điểm đón`, 'arrive'),
  driverStep(`${prefix} Đón khách`, 'pickup'),
  driverStep(`${prefix} Bắt đầu chuyến`, 'start'),
];

const smoke = folder('SYS-01 · Smoke', 'Ba phép kiểm smoke SM-01…03: health, chặn thiếu danh tính, hành trình đặt → hoàn tất → thu tiền.', [
  ...setup,
  req('SM-01 · Health', 'GET', '/api/health', { test: [status(200), "pm.test('Thân đúng hợp đồng R-API-15', () => pm.expect(pm.response.json()).to.eql({ status: 'ok' }));"] }),
  req('SM-02 · Báo giá thiếu danh tính', 'POST', '/api/quotes', { body: QUOTE_5KM, test: [status(401), errorCode('UNAUTHENTICATED')] }),
  ...bookingJourney('SM-03 ·', 'vixanh'),
  req('SM-03 · Hoàn tất chuyến', 'POST', '/api/trips/{{tripId}}/complete', {
    user: 'driver:D101', body: {},
    test: [status(200), "const t = pm.response.json().trip;", "pm.test('completed và paid (R-STATE-02, R-PAY-03)', () => {", "  pm.expect(t.status).to.eql('completed');", "  pm.expect(t.paymentStatus).to.eql('paid');", "});"],
  }),
  req('SM-03 · Khách đọc lại chuyến', 'GET', '/api/trips/{{tripId}}', {
    user: 'customer:C001',
    test: [status(200), "const t = pm.response.json().trip;", "pm.test('Đọc lại: completed, 35.000 đ, paid', () => {", "  pm.expect(t.status).to.eql('completed');", "  pm.expect(t.fare).to.eql(35000);", "  pm.expect(t.paymentStatus).to.eql('paid');", "});"],
  }),
]);

const sanity = folder('SYS-02 · Sanity voucher', 'Kiểm vùng thay đổi chuẩn hoá mã voucher: khoảng trắng, chữ thường, mã không tồn tại.', [
  ...setup,
  ...[['SN · Mã chuẩn GIAM20', 'GIAM20'], ['SN · Mã có khoảng trắng và chữ thường', ' giam20 ']].map(([name, code]) =>
    req(name, 'POST', '/api/quotes', {
      user: 'customer:C001', body: { ...QUOTE_5KM, voucherCode: code },
      test: [status(200), "const q = pm.response.json().quote;", "pm.test('Giảm 6.900 đ, còn 28.000 đ', () => {", "  pm.expect(q.discount).to.eql(6900);", "  pm.expect(q.total).to.eql(28000);", "});"],
    })),
  req('SN · Mã không tồn tại', 'POST', '/api/quotes', { user: 'customer:C001', body: { ...QUOTE_5KM, voucherCode: 'khongco' }, test: [status(422), errorCode('VOUCHER_NOT_FOUND')] }),
]);

const functional = folder('SYS-03 · Functional theo bảng cước', 'Chạy bằng Collection Runner với data/bang-cuoc.csv: mỗi dòng một test case, giá trị mong đợi tính tay theo R-FARE.', [
  req('FN · Báo giá theo bảng dữ liệu', 'POST', '/api/quotes', {
    user: 'customer:C001',
    body: { kind: 'ride', cityCode: 'SG', vehicle: '{{vehicle}}', distanceKm: '{{distanceKm}}' },
    test: [
      "const want = pm.iterationData.get('expected');",
      "pm.test(pm.iterationData.get('caseId') + ' · HTTP ' + pm.iterationData.get('status'), () => pm.response.to.have.status(Number(pm.iterationData.get('status'))));",
      "if (Number(pm.iterationData.get('status')) === 200) pm.test(pm.iterationData.get('caseId') + ' · cước ' + want, () => pm.expect(pm.response.json().quote.total).to.eql(Number(want)));",
      "else pm.test(pm.iterationData.get('caseId') + ' · mã lỗi ' + want, () => pm.expect(pm.response.json().error.code).to.eql(want));",
    ],
  }),
]);
// distanceKm truyền chuỗi qua biến ⇒ thân JSON cần số: sửa lại raw cho đúng kiểu.
functional.item[0].request.body.raw = '{\n  "kind": "ride",\n  "cityCode": "SG",\n  "vehicle": "{{vehicle}}",\n  "distanceKm": {{distanceKm}}\n}';

const integration = folder('SYS-04 · Tích hợp ví theo chế độ lỗi', 'Chạy bằng Collection Runner với data/che-do-vi.json: đặt chế độ ví giả rồi đi hết hành trình VíXanh, so trạng thái thanh toán.', [
  req('Reset dữ liệu mẫu', 'POST', '/api/test/reset', { test: [status(200)] }),
  req('Đặt chế độ ví theo dữ liệu', 'POST', '/api/test/payment', { body: { mode: '{{mode}}' }, test: [status(200)] }),
  ...bookingJourney('IN ·', 'vixanh'),
  req('IN · Hoàn tất và so trạng thái thanh toán', 'POST', '/api/trips/{{tripId}}/complete', {
    user: 'driver:D101', body: {},
    test: [status(200), "const t = pm.response.json().trip;", "pm.test(pm.iterationData.get('mode') + ' → ' + pm.iterationData.get('paymentStatus'), () => pm.expect(t.paymentStatus).to.eql(pm.iterationData.get('paymentStatus')));"],
  }),
]);

const authz = folder('SYS-10 · Phân quyền (lab cô lập)', 'Ma trận quyền tối thiểu trên lab: thiếu danh tính, sai vai trò, khách đọc chuyến của khách khác. Chỉ chạy trên lab được chỉ định.', [
  ...setup,
  req('Chuẩn bị · Báo giá của C001', 'POST', '/api/quotes', { user: 'customer:C001', body: QUOTE_5KM, test: [status(200), "pm.collectionVariables.set('quoteId', pm.response.json().quote.quoteId);"] }),
  req('Chuẩn bị · C001 đặt chuyến', 'POST', '/api/trips', { user: 'customer:C001', key: '{{$guid}}', body: { quoteId: '{{quoteId}}', paymentMethod: 'cash' }, test: [status(201), "pm.collectionVariables.set('tripId', pm.response.json().trip.id);"] }),
  req('SEC · Thiếu danh tính → 401', 'GET', '/api/trips/{{tripId}}', { test: [status(401), errorCode('UNAUTHENTICATED')] }),
  req('SEC · Tài xế gọi báo giá → 403', 'POST', '/api/quotes', { user: 'driver:D101', body: QUOTE_5KM, test: [status(403), errorCode('FORBIDDEN')] }),
  req('SEC · C002 đọc chuyến của C001 → 404', 'GET', '/api/trips/{{tripId}}', { user: 'customer:C002', test: [status(404), errorCode('TRIP_NOT_FOUND'), "pm.test('Không lộ dữ liệu chuyến', () => pm.expect(pm.response.json()).to.not.have.property('trip'));"] }),
  req('SEC · C001 đọc chuyến của mình → 200', 'GET', '/api/trips/{{tripId}}', { user: 'customer:C001', test: [status(200)] }),
]);

const idempotency = folder('SYS-13 · Idempotency đặt chuyến', 'Gửi lại cùng khoá cùng thân ⇒ cùng chuyến; cùng khoá khác thân ⇒ 422 (R-IDEM-02/03).', [
  ...setup,
  req('Chuẩn bị · Báo giá', 'POST', '/api/quotes', { user: 'customer:C001', body: QUOTE_5KM, test: [status(200), "pm.collectionVariables.set('quoteId', pm.response.json().quote.quoteId);", "pm.collectionVariables.set('idemKey', 'sys13-' + Date.now());"] }),
  req('DI · Lần gửi đầu', 'POST', '/api/trips', { user: 'customer:C001', key: '{{idemKey}}', body: { quoteId: '{{quoteId}}', paymentMethod: 'cash' }, test: [status(201), "pm.collectionVariables.set('tripId', pm.response.json().trip.id);"] }),
  req('DI · Gửi lại cùng khoá, cùng thân', 'POST', '/api/trips', { user: 'customer:C001', key: '{{idemKey}}', body: { quoteId: '{{quoteId}}', paymentMethod: 'cash' }, test: [status(201), "pm.test('Trả lại đúng chuyến cũ (R-IDEM-02)', () => pm.expect(pm.response.json().trip.id).to.eql(pm.collectionVariables.get('tripId')));"] }),
  req('DI · Cùng khoá, thân khác', 'POST', '/api/trips', { user: 'customer:C001', key: '{{idemKey}}', body: { quoteId: '{{quoteId}}', paymentMethod: 'vixanh' }, test: [status(422), errorCode('IDEMPOTENCY_KEY_REUSED')] }),
]);

const recovery = folder('SYS-14 · Ví mất phản hồi kéo dài (RC-03)', 'Ví giả trừ tiền nhưng mất phản hồi ở mọi lần gọi: sau 3 lần thử (R-NFR-05) chuyến ghi failed dù tiền đã bị trừ — sai lệch phải đối soát (R-PAY-04, R-TEST-03).', [
  ...setup,
  req('Ví: mất phản hồi', 'POST', '/api/test/payment', { body: { mode: 'lost_response' }, test: [status(200)] }),
  ...bookingJourney('RC ·', 'vixanh'),
  req('RC · Hoàn tất khi ví mất phản hồi', 'POST', '/api/trips/{{tripId}}/complete', {
    user: 'driver:D101', body: {},
    test: [status(200), "const t = pm.response.json().trip;", "pm.test('RC-03 · ghi failed dù ví đã trừ tiền (cần đối soát)', () => pm.expect(t.paymentStatus).to.eql('failed'));", "console.log('paymentStatus =', t.paymentStatus);"],
  }),
  req('Ví: trở lại bình thường', 'POST', '/api/test/payment', { body: { mode: 'none' }, test: [status(200)] }),
]);

const usability = folder('SYS-12 · Thông điệp lỗi cho người dùng', 'Kiểm bổ trợ cho usability: lỗi nhập quãng đường phải trả thông điệp tiếng Việt nêu rõ khoảng hợp lệ. Không thay cho phiên quan sát người dùng.', [
  req('UX · Quãng đường 0,1 km', 'POST', '/api/quotes', {
    user: 'customer:C001', body: { ...QUOTE_5KM, distanceKm: 0.1 },
    test: [
      status(422), errorCode('DISTANCE_OUT_OF_RANGE'),
      "const msg = pm.response.json().error.message;",
      "pm.test('Thông điệp nêu khoảng hợp lệ 0.5–100 km', () => pm.expect(msg).to.include('0.5 đến 100 km'));",
      "pm.test('Không lộ chi tiết kỹ thuật', () => pm.expect(msg).to.not.match(/Error|stack|undefined/));",
    ],
  }),
]);

const uatCancel = folder('SYS-15 · Supporting check UA-02 (huỷ chuyến)', 'Chạy bằng Collection Runner với data/huy-chuyen.json: huỷ sau 1 phút được miễn phí, sau 3 phút thu 10.000 đ (R-CANCEL-01/02/03/06). Đây là supporting check, không thay phiên UAT và chữ ký nghiệp vụ.', [
  req('Reset dữ liệu mẫu', 'POST', '/api/test/reset', { test: [status(200)] }),
  req('Đặt đồng hồ 09:00', 'POST', '/api/test/clock', { body: { now: '2026-11-04T09:00:00+07:00' }, test: [status(200)] }),
  req('UA-02 · Báo giá xe máy 5 km', 'POST', '/api/quotes', { user: 'customer:C001', body: QUOTE_5KM, test: [status(200), "pm.collectionVariables.set('quoteId', pm.response.json().quote.quoteId);"] }),
  req('UA-02 · Đặt chuyến tiền mặt', 'POST', '/api/trips', { user: 'customer:C001', key: '{{$guid}}', body: { quoteId: '{{quoteId}}', paymentMethod: 'cash' }, test: [status(201), "pm.collectionVariables.set('tripId', pm.response.json().trip.id);"] }),
  driverStep('UA-02 · Tài xế D101 nhận chuyến', 'accept', { etaMinutes: 5 }),
  req('UA-02 · Dời đồng hồ theo dữ liệu', 'POST', '/api/test/clock', { body: { now: '{{now}}' }, test: [status(200)] }),
  req('UA-02 · Khách huỷ chuyến', 'POST', '/api/trips/{{tripId}}/cancel', {
    user: 'customer:C001', key: '{{$guid}}', body: { reason: 'Thay đổi kế hoạch' },
    test: [
      status(200), "const b = pm.response.json();",
      "pm.test('Huỷ sau ' + pm.iterationData.get('sau') + ' → phí ' + pm.iterationData.get('fee') + ' đ (' + pm.iterationData.get('reason') + ')', () => {",
      "  pm.expect(b.trip.status).to.eql('cancelled');",
      "  pm.expect(b.cancellationFee).to.eql(Number(pm.iterationData.get('fee')));",
      "  pm.expect(b.feeReason).to.eql(pm.iterationData.get('reason'));",
      "});",
    ],
  }),
]);

const postDeploy = folder('SYS-16 · Post-deploy smoke', 'Chạy trên environment của bản triển khai: health, API kiểm thử phải đóng (R-TEST-01), báo giá đúng cước.', [
  req('DP · Health', 'GET', '/api/health', { test: [status(200)] }),
  req('DP · API kiểm thử phải đóng', 'POST', '/api/test/reset', { test: ["pm.test('R-TEST-01: /api/test/* trả 404 trên bản triển khai', () => pm.response.to.have.status(404));"] }),
  req('DP · Báo giá 5 km', 'POST', '/api/quotes', { user: 'customer:C001', body: QUOTE_5KM, test: [status(200), "pm.test('Cước 35.000 đ', () => pm.expect(pm.response.json().quote.total).to.eql(35000));"] }),
]);

const load = folder('SYS-07…09 · Request cho kịch bản tải', 'Request nguồn để dựng kịch bản load test trong CurlStudio (chuột phải → Load test). Chỉ chạy trên lab cô lập.', [
  req('PERF · Báo giá xe máy 5 km', 'POST', '/api/quotes', { user: 'customer:C001', body: QUOTE_5KM, test: [status(200), "pm.test('Cước 35.000 đ', () => pm.expect(pm.response.json().quote.total).to.eql(35000));"] }),
]);

const readTrip = (name, checks) => req(name, 'GET', '/api/trips/{{tripId}}', { user: 'customer:C001', test: [status(200), "const t = pm.response.json().trip;", ...checks] });
const journey = folder('SYS-05 · Hành trình khách–tài xế qua API', 'Phía API của system test: cùng một chuyến đi qua hai vai trò; sau mỗi chặng, khách đọc lại để đối chiếu với điều trang khách phải hiển thị (R-STATE-02, R-ACCEPT, R-API-05/10/12). Không thay kiểm tra giao diện bằng Playwright.', [
  ...setup,
  req('SY · Khách báo giá xe máy 5 km', 'POST', '/api/quotes', { user: 'customer:C001', body: QUOTE_5KM, test: [status(200), "pm.collectionVariables.set('quoteId', pm.response.json().quote.quoteId);"] }),
  req('SY · Khách đặt chuyến (vixanh)', 'POST', '/api/trips', { user: 'customer:C001', key: '{{$guid}}', body: { quoteId: '{{quoteId}}', paymentMethod: 'vixanh' }, test: [status(201), "pm.collectionVariables.set('tripId', pm.response.json().trip.id);"] }),
  req('SY · Tài xế D101 xem chuyến khả dụng', 'GET', '/api/driver/available-trips', {
    user: 'driver:D101',
    test: [status(200), "const ids = pm.response.json().trips.map((t) => t.id);", "pm.test('Chuyến vừa đặt có trong danh sách của D101 (R-ACCEPT-06)', () => pm.expect(ids).to.include(pm.collectionVariables.get('tripId')));"],
  }),
  driverStep('SY · Tài xế D101 nhận chuyến', 'accept', { etaMinutes: 5 }),
  readTrip('SY · Khách đọc lại sau khi tài xế nhận', ["pm.test('accepted, tài xế D101, ETA 5 phút', () => {", "  pm.expect(t.status).to.eql('accepted');", "  pm.expect(t.driverId).to.eql('D101');", "  pm.expect(t.etaMinutes).to.eql(5);", "});"]),
  driverStep('SY · Tài xế tới điểm đón', 'arrive'),
  driverStep('SY · Đón khách', 'pickup'),
  driverStep('SY · Bắt đầu chuyến', 'start'),
  readTrip('SY · Khách đọc lại khi đang đi', ["pm.test('in_transit', () => pm.expect(t.status).to.eql('in_transit'));"]),
  driverStep('SY · Hoàn tất chuyến', 'complete'),
  readTrip('SY · Khách đọc lại khi kết thúc', ["pm.test('completed, 35.000 đ, paid', () => {", "  pm.expect(t.status).to.eql('completed');", "  pm.expect(t.fare).to.eql(35000);", "  pm.expect(t.paymentStatus).to.eql('paid');", "});"]),
]);

const quoteTotal = (name, total, why) => req(name, 'POST', '/api/quotes', { user: 'customer:C001', body: QUOTE_5KM, test: [status(200), `pm.test('${why}', () => pm.expect(pm.response.json().quote.total).to.eql(${total}));`] });
const fareCombos = folder('SYS-06 · Hồi quy tổ hợp cước', 'Hai tổ hợp mà lượt hồi quy 44/78 bỏ sót: mưa ở giờ thường và mưa ở giờ cao điểm. Bản refactor sai trong bài cho 54.000 ở tổ hợp cao điểm + mưa; oracle tính tay theo R-FARE-04/05/08 là 52.000.', [
  req('Reset dữ liệu mẫu', 'POST', '/api/test/reset', { test: [status(200)] }),
  req('Trời mưa ở SG', 'POST', '/api/test/weather', { body: { raining: ['SG'] }, test: [status(200)] }),
  req('Đồng hồ 09:00 (giờ thường)', 'POST', '/api/test/clock', { body: { now: '2026-11-04T09:00:00+07:00' }, test: [status(200)] }),
  quoteTotal('RG-02 · Thường + mưa', 43000, 'RG-02 · 34.500 + 8.000 → 43.000 đ'),
  req('Đồng hồ 11:30 (cao điểm)', 'POST', '/api/test/clock', { body: { now: '2026-11-04T11:30:00+07:00' }, test: [status(200)] }),
  quoteTotal('RG-04 · Cao điểm + mưa', 52000, 'RG-04 · 34.500 × 1,25 + 8.000 → 52.000 đ (bản sai cho 54.000)'),
  req('Trời tạnh', 'POST', '/api/test/weather', { body: { raining: [] }, test: [status(200)] }),
  // /api/test/reset không đặt lại đồng hồ: không trả về 09:00 thì folder chạy sau (vd SYS-04) báo giá theo giờ cao điểm.
  req('Đồng hồ về 09:00', 'POST', '/api/test/clock', { body: { now: '2026-11-04T09:00:00+07:00' }, test: [status(200)] }),
]);

const collection = {
  info: {
    _postman_id: 'toi-noi-lab-sys',
    name: 'Tới Nơi lab · Kiểm thử hệ thống',
    description: 'Collection minh hoạ series Kiểm thử hệ thống Tới Nơi (SYS-01…17). Chạy trên máy chủ lab local với NODE_ENV=test; không trỏ vào hệ thống thật.',
    schema: 'https://schema.getpostman.com/json/collection/v2.1.0/collection.json',
  },
  variable: [{ key: 'quoteId', value: '' }, { key: 'tripId', value: '' }],
  item: [folder('00 · Chuẩn bị lab', 'Đưa lab về trạng thái cố định trước mỗi lượt.', setup), smoke, sanity, functional, integration, authz, usability, idempotency, recovery, uatCancel, postDeploy, load, journey, fareCombos],
};

const env = (id, name, baseUrl) => ({
  id, name,
  values: [{ key: 'baseUrl', value: baseUrl, type: 'default', enabled: true }],
  _postman_variable_scope: 'environment',
});

writeFileSync(new URL('toi-noi-lab.postman_collection.json', out), JSON.stringify(collection, null, 2) + '\n');
writeFileSync(new URL('toi-noi-lab-local.postman_environment.json', out), JSON.stringify(env('toi-noi-lab-local', 'Tới Nơi lab (local, API test bật)', 'http://localhost:3100'), null, 2) + '\n');
writeFileSync(new URL('toi-noi-prod-like.postman_environment.json', out), JSON.stringify(env('toi-noi-prod-like', 'Tới Nơi bản triển khai (prod-like)', 'http://localhost:3101'), null, 2) + '\n');
// Bảng cước: tính tay theo R-FARE-01…08 (xe máy 12.000 + 4.500/km, ô tô 25.000 + 11.000/km, làm tròn lên nghìn, tối thiểu 15.000).
writeFileSync(new URL('data/bang-cuoc.csv', out), [
  'caseId,vehicle,distanceKm,status,expected',
  'FN-T01,bike,5,200,35000',
  'FN-T02,bike,2,200,21000',
  'FN-T03,bike,0.5,200,15000',
  'FN-T04,car,5,200,80000',
  'FN-T05,car,10,200,135000',
  'FN-T06,bike,0.1,422,DISTANCE_OUT_OF_RANGE',
].join('\n') + '\n');
writeFileSync(new URL('data/che-do-vi.json', out), JSON.stringify([
  { mode: 'none', paymentStatus: 'paid' },
  { mode: 'decline', paymentStatus: 'failed' },
  { mode: 'error', paymentStatus: 'failed' },
], null, 2) + '\n');
// Bảng cố ý ghi sai FN-T02 (22.000 thay vì 21.000) để xem runner báo đỏ — dùng trong SYS-03, không phải đặc tả.
writeFileSync(new URL('data/bang-cuoc-oracle-sai.csv', out), [
  'caseId,vehicle,distanceKm,status,expected',
  'FN-T01,bike,5,200,35000',
  'FN-T02,bike,2,200,22000',
  'FN-T03,bike,0.5,200,15000',
].join('\n') + '\n');
// Một chế độ timeout: ví không trả lời, 3 lần thử × 3 giây (R-NFR-05) — dùng trong SYS-04.
writeFileSync(new URL('data/che-do-vi-timeout.json', out), JSON.stringify([{ mode: 'timeout', paymentStatus: 'failed' }], null, 2) + '\n');
writeFileSync(new URL('data/huy-chuyen.json', out), JSON.stringify([
  { sau: '1 phút', now: '2026-11-04T09:01:00+07:00', fee: 0, reason: 'grace_period' },
  { sau: '3 phút', now: '2026-11-04T09:03:00+07:00', fee: 10000, reason: 'charged' },
], null, 2) + '\n');
console.log('Đã ghi collection, 2 environment và 5 tệp dữ liệu vào', out.pathname);
