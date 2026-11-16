// P01 · 02-fixture-valid.js — bộ dữ liệu nhỏ, cố định, hợp lệ (MG-003, MG-005, MG-006, MG-007, MG-010).
// Cùng dữ liệu với lab PostgreSQL và SQL Server để đối chiếu ba engine.
//
// Mọi tên, số điện thoại và tên quán đều HƯ CẤU (090000000x là số giả).
//
// Để kết quả in ra giống nhau ở mọi lần chạy:
//   * khách, tài xế, quán, chuyến dùng UUID ghi sẵn (c… = khách, d…800x = tài xế
//     mã 800x, e… = quán, 7…100000 ↔ chuyến mã 100000). Đuôi trùng mã CHỈ để dễ
//     đọc; trong hệ thật id không liên quan gì tới code;
//   * mọi mốc thời gian viết theo giờ VN qua hàm vn() (lib/check.js), lưu thành
//     BSON Date (UTC);
//   * số nguyên viết trong mongosh được lưu là int (Int32) — MG-004 nói khi nào không.
//
// Ảnh chụp dữ liệu lúc 2026-11-03 08:30 (giờ VN): ngày 02/11 đã xong, sáng 03/11
// còn 6 chuyến đang chạy ở 6 trạng thái khác nhau. Lịch sử `events` của 15 chuyến
// này để rỗng (dữ liệu chuyển từ hệ cũ chỉ có trạng thái cuối); 04-lifecycle.js
// ghi lịch sử đầy đủ cho chuyến mới.

const C = (n) => UUID(`c0000000-0000-4000-8000-00000000000${n}`);
const D = (n) => UUID(`d0000000-0000-4000-8000-00000000${n}`);
const M = (n) => UUID(`e0000000-0000-4000-8000-00000000000${n}`);
const T = (n) => UUID(`70000000-0000-4000-8000-000000${n}`);

let r = db.cities.insertMany([
  { _id: 'SG', name: 'TP. Hồ Chí Minh', tz: 'Asia/Ho_Chi_Minh' },
  { _id: 'HN', name: 'Hà Nội', tz: 'Asia/Ho_Chi_Minh' },
  { _id: 'DN', name: 'Đà Nẵng', tz: 'Asia/Ho_Chi_Minh' },
]);
expectValue('cities sau khi nạp', Object.keys(r.insertedIds).length, 3);

r = db.customers.insertMany([
  { _id: C(1), phone: '0900000001', full_name: 'Nguyễn Văn An', created_at: vn('2026-09-01 08:00') },
  { _id: C(2), phone: '0900000002', full_name: 'Trần Thị Bình', created_at: vn('2026-09-03 19:15') },
  { _id: C(3), phone: '0900000003', full_name: 'Lê Hoàng Cường', created_at: vn('2026-09-10 12:00') },
  { _id: C(4), phone: '0900000004', full_name: 'Phạm Thu Dung', created_at: vn('2026-10-01 07:30') },
  { _id: C(5), phone: '0900000005', full_name: 'Võ Minh Đức', created_at: vn('2026-10-15 21:45') },
  { _id: C(6), phone: '0900000006', full_name: 'Đặng Ngọc Hà', created_at: vn('2026-11-01 10:00') },
]);
expectValue('customers sau khi nạp', Object.keys(r.insertedIds).length, 6);

// Trạng thái tài xế khớp ảnh chụp: ai đang có chuyến chạy thì `busy`.
r = db.drivers.insertMany([
  { _id: D(8001), code: 8001, full_name: 'Bùi Văn Khoa', city_id: 'SG', vehicle_type: 'bike', status: 'busy', created_at: vn('2026-08-01 09:00') },
  { _id: D(8002), code: 8002, full_name: 'Hồ Thị Lan', city_id: 'SG', vehicle_type: 'bike', status: 'busy', created_at: vn('2026-08-02 09:00') },
  { _id: D(8003), code: 8003, full_name: 'Ngô Quang Minh', city_id: 'HN', vehicle_type: 'bike', status: 'online', created_at: vn('2026-08-05 09:00') },
  { _id: D(8004), code: 8004, full_name: 'Đỗ Thanh Nam', city_id: 'HN', vehicle_type: 'car', status: 'busy', created_at: vn('2026-08-20 09:00') },
  { _id: D(8005), code: 8005, full_name: 'Lý Thị Oanh', city_id: 'DN', vehicle_type: 'bike', status: 'busy', created_at: vn('2026-09-01 09:00') },
  { _id: D(8006), code: 8006, full_name: 'Trịnh Văn Phúc', city_id: 'DN', vehicle_type: 'car', status: 'offline', created_at: vn('2026-09-15 09:00') },
]);
expectValue('drivers sau khi nạp', Object.keys(r.insertedIds).length, 6);

// Menu nhúng trong quán: giá là int (đồng).
r = db.merchants.insertMany([
  { _id: M(1), city_id: 'SG', name: 'Cơm Tấm Phố Lab', created_at: vn('2026-07-01 08:00'), menu: [
    { sku: 'CT-SUON', name: 'Cơm tấm sườn', price: 45000 },
    { sku: 'CT-BSC', name: 'Cơm tấm sườn bì chả', price: 59000 },
  ] },
  { _id: M(2), city_id: 'HN', name: 'Bún Chả Ngõ Mẫu', created_at: vn('2026-07-01 08:00'), menu: [
    { sku: 'BC-THUONG', name: 'Bún chả suất thường', price: 50000 },
    { sku: 'BC-NEM', name: 'Nem rán (2 chiếc)', price: 20000 },
  ] },
  { _id: M(3), city_id: 'DN', name: 'Mì Quảng Thử Nghiệm', created_at: vn('2026-07-01 08:00'), menu: [
    { sku: 'MQ-GA', name: 'Mì Quảng gà', price: 40000 },
    { sku: 'MQ-TOM', name: 'Mì Quảng tôm thịt', price: 45000 },
  ] },
]);
expectValue('merchants sau khi nạp', Object.keys(r.insertedIds).length, 3);

// 15 chuyến, đủ 8 trạng thái và 3 loại dịch vụ. Trường không áp dụng thì KHÔNG có
// trong document (không ghi null): chuyến chưa có tài xế không có driver_id.
// Hai chuyến 100003 và 100004 CÙNG giờ đặt 12:00 (dùng ở 05-find.js).
// 100007 đặt 23:50 ngày 02/11, 100008 đặt 00:10 ngày 03/11 giờ VN (vẫn là ngày
// 02/11 theo UTC) — dùng để thử lọc theo ngày.
function trip(code, kind, city, cust, status, fare, req, more = {}) {
  // Thứ tự khoá trong object = thứ tự trường trong document BSON.
  return {
    _id: T(code), code, kind, city_id: city, customer_id: C(cust),
    ...(more.driver ? { driver_id: D(more.driver) } : {}),
    ...(more.merchant ? { merchant_id: M(more.merchant) } : {}),
    status, fare, requested_at: vn(req),
    ...(more.driver ? { accepted_at: vn(more.acc) } : {}),
    ...(more.done ? { completed_at: vn(more.done) } : {}),
    updated_at: vn(more.upd ?? more.done ?? more.acc ?? req),
    events: [],
  };
}
r = db.trips.insertMany([
  trip(100000, 'ride', 'SG', 1, 'completed', 45000, '2026-11-02 07:58', { driver: 8001, acc: '2026-11-02 08:00:30', done: '2026-11-02 08:24:10' }),
  trip(100001, 'parcel', 'HN', 3, 'completed', 38000, '2026-11-02 09:15', { driver: 8003, acc: '2026-11-02 09:17', done: '2026-11-02 10:02' }),
  trip(100002, 'food', 'SG', 2, 'completed', 62000, '2026-11-02 11:05', { driver: 8002, merchant: 1, acc: '2026-11-02 11:06', done: '2026-11-02 11:41' }),
  // 100003: đồ ăn bị huỷ SAU khi tài xế đã nhận (vẫn giữ tài xế và accepted_at)
  trip(100003, 'food', 'HN', 3, 'cancelled', 71000, '2026-11-02 12:00', { driver: 8003, merchant: 2, acc: '2026-11-02 12:01', upd: '2026-11-02 12:04' }),
  trip(100004, 'food', 'DN', 5, 'completed', 55000, '2026-11-02 12:00', { driver: 8005, merchant: 3, acc: '2026-11-02 12:02', done: '2026-11-02 12:35' }),
  trip(100005, 'ride', 'HN', 4, 'completed', 126000, '2026-11-02 17:30', { driver: 8004, acc: '2026-11-02 17:31', done: '2026-11-02 18:05' }),
  // 100006: đặt xe bị huỷ TRƯỚC khi có người nhận (không tài xế)
  trip(100006, 'ride', 'SG', 1, 'cancelled', 52000, '2026-11-02 18:00', { upd: '2026-11-02 18:03' }),
  trip(100007, 'parcel', 'SG', 6, 'completed', 30000, '2026-11-02 23:50', { driver: 8002, acc: '2026-11-02 23:52', done: '2026-11-03 00:20' }),
  // 100008: đặt xe miễn phí (khuyến mãi) — cước 0 đ là hợp lệ
  trip(100008, 'ride', 'SG', 2, 'completed', 0, '2026-11-03 00:10', { driver: 8001, acc: '2026-11-03 00:11', done: '2026-11-03 00:30' }),
  // 100009..100014: sáu chuyến đang chạy lúc 08:30 ngày 03/11
  trip(100009, 'food', 'SG', 2, 'picked_up', 64000, '2026-11-03 08:00', { driver: 8002, merchant: 1, acc: '2026-11-03 08:02', upd: '2026-11-03 08:15' }),
  trip(100010, 'ride', 'HN', 3, 'in_transit', 98000, '2026-11-03 08:05', { driver: 8004, acc: '2026-11-03 08:06', upd: '2026-11-03 08:13' }),
  trip(100011, 'parcel', 'DN', 5, 'arriving', 27000, '2026-11-03 08:10', { driver: 8005, acc: '2026-11-03 08:12', upd: '2026-11-03 08:12:30' }),
  trip(100012, 'ride', 'SG', 1, 'accepted', 39000, '2026-11-03 08:20', { driver: 8001, acc: '2026-11-03 08:21' }),
  trip(100013, 'food', 'HN', 4, 'searching', 85000, '2026-11-03 08:25', { merchant: 2, upd: '2026-11-03 08:25:10' }),
  trip(100014, 'ride', 'SG', 6, 'requested', 48000, '2026-11-03 08:29:30'),
]);
expectValue('trips sau khi nạp', Object.keys(r.insertedIds).length, 15);

// Bộ đếm mã chuyến bắt đầu từ mã lớn nhất đã có.
db.counters.insertOne({ _id: 'trip_code', seq: 100014 });
expectValue('counters.trip_code', db.counters.findOne({ _id: 'trip_code' }).seq, 100014);

section('Một chuyến nhìn dưới dạng document (100004)');
printjson(db.trips.findOne({ code: 100004 }));

section('Số chuyến theo trạng thái và loại (đủ 8 trạng thái)');
table(db.trips.aggregate([
  { $group: {
    _id: '$status',
    ride: { $sum: { $cond: [{ $eq: ['$kind', 'ride'] }, 1, 0] } },
    food: { $sum: { $cond: [{ $eq: ['$kind', 'food'] }, 1, 0] } },
    parcel: { $sum: { $cond: [{ $eq: ['$kind', 'parcel'] }, 1, 0] } },
    tong: { $sum: 1 },
  } },
  { $sort: { _id: 1 } },
]), ['_id', 'ride', 'food', 'parcel', 'tong']);
expectValue('số trạng thái khác nhau', db.trips.distinct('status').length, 8);
