/**
 * TCD-02 — năm test case viết tốt cho R-ACCEPT (tài xế nhận chuyến) và bộ sưu tập test case viết dở.
 * Kết quả mong đợi suy ra từ SPEC v1.1, không chép từ việc chạy hệ thống.
 */
import type { TestCaseDoc } from './test-case-schema';

const ENV = 'lab toi-noi-testing, Node 22, createApp + kho trong bộ nhớ, FixedClock, id tuần tự; gọi qua supertest';
const VERSION = 'SPEC v1.1 · fare-2026.1';

/** Tiền điều kiện chung: chuyến SG xe máy ở `searching` (đặt bằng API, không trỏ tới test case khác). */
const BASE_PRECONDITIONS = [
  'Hệ thống vừa reset về dữ liệu mẫu (SPEC Phụ lục A)',
  'Đồng hồ hệ thống cố định ở 09:00:00 giờ VN ngày 04/11/2026 (`2026-11-04T02:00:00.000Z`)',
  'Khách C001 đã tạo chuyến T1 từ báo giá `ride`, SG, `bike`, 5 km; T1 đang ở `searching`, chưa có tài xế',
];

export const GOOD: TestCaseDoc[] = [
  {
    id: 'TC-ACCEPT-001',
    title: 'Tài xế online cùng thành phố, đúng loại xe nhận được chuyến đang tìm tài xế',
    objective: 'Chứng minh tài xế thoả mọi điều kiện của R-ACCEPT-01 nhận được chuyến và các trường của phân công được ghi đúng',
    priority: 'cao',
    traceability: { requirements: ['R-ACCEPT-01', 'R-ACCEPT-04', 'R-ACCEPT-05', 'R-STATE-07'], conditions: ['TCOND-ACCEPT-01', 'TCOND-ACCEPT-09'] },
    basisVersion: VERSION,
    environment: ENV,
    preconditions: [...BASE_PRECONDITIONS, 'Tài xế D101 (SG, `bike`) đang `online`'],
    testData: [{ id: 'TD-01', description: 'ETA hợp lệ ở giữa khoảng 1–60', values: { driver: 'D101', etaMinutes: 7 } }],
    steps: [
      { action: 'Gửi `POST /api/trips/T1/accept` với header `x-user: driver:D101`, thân `{"etaMinutes":7}`', data: 'TD-01' },
      { action: 'Đọc trạng thái tài xế D101 trong kho dữ liệu' },
    ],
    expectedResults: [
      'HTTP 200; `trip.status` = `accepted`; `trip.driverId` = `D101`',
      '`trip.etaMinutes` = 7; `trip.acceptedAt` = `2026-11-04T02:00:00.000Z`',
      'Tài xế D101 ở trạng thái `busy`',
    ],
    postconditions: ['T1 ở `accepted` với D101; D101 `busy` — dọn bằng reset'],
  },
  {
    id: 'TC-ACCEPT-002',
    title: 'Tài xế offline nhận chuyến bị từ chối với DRIVER_OFFLINE',
    objective: 'Chứng minh tài xế đang offline không nhận được chuyến và chuyến không bị thay đổi',
    priority: 'cao',
    traceability: { requirements: ['R-ACCEPT-01', 'R-ACCEPT-02'], conditions: ['TCOND-ACCEPT-02'] },
    basisVersion: VERSION,
    environment: ENV,
    preconditions: [...BASE_PRECONDITIONS, 'Tài xế D104 (SG, `bike`) đang `offline`'],
    testData: [{ id: 'TD-02', description: 'tài xế offline cùng thành phố, đúng loại xe', values: { driver: 'D104', etaMinutes: 5 } }],
    steps: [
      { action: 'Gửi `POST /api/trips/T1/accept` với header `x-user: driver:D104`, thân `{"etaMinutes":5}`', data: 'TD-02' },
      { action: 'Đọc lại chuyến T1 và trạng thái tài xế D104 trong kho dữ liệu' },
    ],
    expectedResults: [
      'HTTP 409; `error.code` = `DRIVER_OFFLINE`',
      'T1 vẫn ở `searching`, `driverId` = null, `acceptedAt` = null',
      'D104 vẫn `offline`',
    ],
    postconditions: ['Không có thay đổi dữ liệu'],
  },
  {
    id: 'TC-ACCEPT-003',
    title: 'Tài xế khác cả thành phố lẫn loại xe nhận được lỗi CITY_MISMATCH',
    objective: 'Chứng minh khi vi phạm hai điều kiện, lỗi thành phố được báo trước lỗi loại xe theo thứ tự R-ACCEPT-02',
    priority: 'trung-binh',
    traceability: { requirements: ['R-ACCEPT-02'], conditions: ['TCOND-ACCEPT-08'] },
    basisVersion: VERSION,
    environment: ENV,
    preconditions: [...BASE_PRECONDITIONS, 'Tài xế D202 (HN, `car`) đang `online`'],
    testData: [{ id: 'TD-03', description: 'khác thành phố (HN ≠ SG) và khác loại xe (car ≠ bike)', values: { driver: 'D202', etaMinutes: 5 } }],
    steps: [{ action: 'Gửi `POST /api/trips/T1/accept` với header `x-user: driver:D202`, thân `{"etaMinutes":5}`', data: 'TD-03' }],
    expectedResults: ['HTTP 422; `error.code` = `CITY_MISMATCH` (không phải `VEHICLE_MISMATCH`)', 'T1 vẫn ở `searching`, `driverId` = null'],
    postconditions: ['Không có thay đổi dữ liệu'],
  },
  {
    id: 'TC-ACCEPT-004',
    title: 'Tài xế thứ hai nhận chuyến đã có người nhận bị từ chối với TRIP_ALREADY_TAKEN',
    objective: 'Chứng minh chuyến đã được nhận thì tài xế khác không giành được và vẫn online',
    priority: 'cao',
    traceability: { requirements: ['R-ACCEPT-02', 'R-ACCEPT-03'], conditions: ['TCOND-ACCEPT-06'] },
    basisVersion: VERSION,
    environment: ENV,
    preconditions: [
      ...BASE_PRECONDITIONS.slice(0, 2),
      'Khách C001 đã tạo chuyến T1 từ báo giá `ride`, SG, `bike`, 5 km; tài xế D101 đã nhận T1 với ETA 7 phút (T1 ở `accepted`)',
      'Tài xế D102 (SG, `bike`) đang `online`',
    ],
    testData: [{ id: 'TD-04', description: 'tài xế thứ hai đủ điều kiện', values: { driver: 'D102', etaMinutes: 4 } }],
    steps: [
      { action: 'Gửi `POST /api/trips/T1/accept` với header `x-user: driver:D102`, thân `{"etaMinutes":4}`', data: 'TD-04' },
      { action: 'Đọc lại chuyến T1 và trạng thái tài xế D102 trong kho dữ liệu' },
    ],
    expectedResults: ['HTTP 409; `error.code` = `TRIP_ALREADY_TAKEN`', 'T1 vẫn ở `accepted` với `driverId` = `D101`, `etaMinutes` = 7', 'D102 vẫn `online`'],
    postconditions: ['T1 ở `accepted` với D101 — dọn bằng reset'],
  },
  {
    id: 'TC-ACCEPT-005',
    title: 'Tài xế gửi lại lệnh nhận cho chuyến mình đang giữ nhận lại phân công cũ',
    objective: 'Chứng minh lệnh nhận lặp lại của chính tài xế đang giữ chuyến là idempotent theo R-ACCEPT-07',
    priority: 'trung-binh',
    traceability: { requirements: ['R-ACCEPT-07', 'R-IDEM-06'], conditions: ['TCOND-ACCEPT-10'] },
    basisVersion: VERSION,
    environment: ENV,
    preconditions: [
      ...BASE_PRECONDITIONS.slice(0, 2),
      'Khách C001 đã tạo chuyến T1 từ báo giá `ride`, SG, `bike`, 5 km; tài xế D101 đã nhận T1 lúc 09:00:00 với ETA 7 phút',
      'Đồng hồ đã chạy tới 09:01:00 giờ VN',
    ],
    testData: [{ id: 'TD-05', description: 'ETA khác lần đầu, để thấy giá trị cũ được giữ', values: { driver: 'D101', etaMinutes: 15 } }],
    steps: [{ action: 'Gửi `POST /api/trips/T1/accept` với header `x-user: driver:D101`, thân `{"etaMinutes":15}`', data: 'TD-05' }],
    expectedResults: [
      'HTTP 200; `trip.status` = `accepted`; `trip.driverId` = `D101`',
      '`trip.etaMinutes` = 7 (không phải 15); `trip.acceptedAt` = `2026-11-04T02:00:00.000Z` (không phải 09:01)',
    ],
    postconditions: ['T1 ở `accepted` với D101 — dọn bằng reset'],
  },
];

const base = GOOD[1]!; // TC-ACCEPT-002 làm khuôn: mỗi bản dở chỉ hỏng đúng một chỗ

/** Bộ sưu tập test case viết dở: mỗi mục kèm các luật lint mà nó PHẢI vi phạm. */
export const BAD: { label: string; doc: TestCaseDoc; violates: string[] }[] = [
  {
    label: 'kết quả mong đợi mơ hồ',
    doc: { ...base, id: 'TC-ACCEPT-901', expectedResults: ['Hệ thống hoạt động đúng'] },
    violates: ['TC-L01', 'TC-L02'],
  },
  {
    label: 'kết quả mong đợi không có giá trị quan sát',
    doc: { ...base, id: 'TC-ACCEPT-902', expectedResults: ['Tài xế nhận được thông báo lỗi phù hợp'] },
    violates: ['TC-L02'],
  },
  {
    label: 'thiếu tiền điều kiện',
    doc: { ...base, id: 'TC-ACCEPT-903', preconditions: [] },
    violates: ['TC-L03'],
  },
  {
    label: 'nhiều mục tiêu',
    doc: { ...base, id: 'TC-ACCEPT-904', objective: 'Kiểm tra tài xế offline bị từ chối và kiểm tra tài xế busy bị từ chối' },
    violates: ['TC-L04'],
  },
  {
    label: 'phụ thuộc test case khác',
    doc: { ...base, id: 'TC-ACCEPT-905', preconditions: ['Đã chạy xong TC-ACCEPT-001 (T1 có tài xế)'] },
    violates: ['TC-L05'],
  },
  {
    label: 'kết quả không tất định',
    doc: { ...base, id: 'TC-ACCEPT-906', expectedResults: ['HTTP 409 hoặc 422 tuỳ trạng thái tài xế, có thể kèm `DRIVER_OFFLINE`'] },
    violates: ['TC-L06'],
  },
  {
    label: 'truy vết tới yêu cầu không tồn tại',
    doc: { ...base, id: 'TC-ACCEPT-907', traceability: { requirements: ['R-ACCEPT-09'], conditions: [] } },
    violates: ['TC-L07'],
  },
];
