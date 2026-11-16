// P01 · 03-fixture-invalid.js — dữ liệu sai bị chặn ở đâu, và cái gì LỌT (MG-004, MG-005, MG-010).
//
// Mỗi ca vi phạm đúng MỘT quy tắc, so với một document hợp lệ làm mẫu.
//   121   DocumentValidationFailure — validator ($jsonSchema hoặc quy tắc liên trường)
//   11000 DuplicateKey — index unique (kể cả _id)
// Phần cuối là các "bẫy": MongoDB KHÔNG báo lỗi, dữ liệu sai lọt vào. Lab ghi
// nhận rồi dọn đi; 07-checks.js là lưới bắt những gì validator không bắt được.

const C = (n) => UUID(`c0000000-0000-4000-8000-00000000000${n}`);
const D = (n) => UUID(`d0000000-0000-4000-8000-00000000${n}`);
const M = (n) => UUID(`e0000000-0000-4000-8000-00000000000${n}`);

// Document mẫu hợp lệ; mỗi ca sửa đúng một chỗ.
const goodCustomer = () => ({ _id: UUID(), phone: '0900000099', full_name: 'Khách Thử', created_at: vn('2026-11-03 08:00') });
const goodDriver = () => ({ _id: UUID(), code: 8099, full_name: 'Tài Xế Thử', city_id: 'SG', vehicle_type: 'bike', status: 'online', created_at: vn('2026-11-03 08:00') });
const goodTrip = () => ({
  _id: UUID(), code: 199999, kind: 'ride', city_id: 'SG', customer_id: C(1), status: 'requested',
  fare: 45000, requested_at: vn('2026-11-03 08:40'), updated_at: vn('2026-11-03 08:40'), events: [],
});
const without = (o, k) => { const c = { ...o }; delete c[k]; return c; };

// Mẫu phải hợp lệ thật (rồi xoá) — nếu không, mọi ca bên dưới đều vô nghĩa.
for (const [coll, doc] of [['customers', goodCustomer()], ['drivers', goodDriver()], ['trips', goodTrip()]]) {
  db.getCollection(coll).insertOne(doc);
  db.getCollection(coll).deleteOne({ _id: doc._id });
}
print('PASS ba document mẫu hợp lệ');

section('customers');
expectError('C1 thiếu phone', () => db.customers.insertOne(without(goodCustomer(), 'phone')), 121, 'phone');
expectError('C2 phone 9 chữ số', () => db.customers.insertOne({ ...goodCustomer(), phone: '090000009' }), 121, 'pattern');
expectError('C3 phone là số, không phải chuỗi', () => db.customers.insertOne({ ...goodCustomer(), phone: 900000099 }), 121, 'bsonType');
expectError('C4 phone trùng khách 1', () => db.customers.insertOne({ ...goodCustomer(), phone: '0900000001' }), 11000, 'uq_customers_phone');
expectError('C5 full_name rỗng', () => db.customers.insertOne({ ...goodCustomer(), full_name: '' }), 121, 'minLength');
expectError('C6 created_at là chuỗi ngày', () => db.customers.insertOne({ ...goodCustomer(), created_at: '2026-11-03T08:00:00+07:00' }), 121, 'created_at');
expectError('C7 trường lạ (email)', () => db.customers.insertOne({ ...goodCustomer(), email: 'a@example.test' }), 121, 'additionalProperties');
expectError('C8 _id trùng khách 1', () => db.customers.insertOne({ ...goodCustomer(), _id: C(1) }), 11000, '_id_');

section('drivers');
expectError('D1 code trùng 8001', () => db.drivers.insertOne({ ...goodDriver(), code: 8001 }), 11000, 'uq_drivers_code');
expectError('D2 vehicle_type truck', () => db.drivers.insertOne({ ...goodDriver(), vehicle_type: 'truck' }), 121, 'enum');
expectError('D3 status viết hoa ONLINE', () => db.drivers.insertOne({ ...goodDriver(), status: 'ONLINE' }), 121, 'enum');
expectError('D4 code là double 8099.5', () => db.drivers.insertOne({ ...goodDriver(), code: 8099.5 }), 121, 'bsonType');
expectError('D5 city_id viết thường', () => db.drivers.insertOne({ ...goodDriver(), city_id: 'sg' }), 121, 'pattern');

section('trips: kiểu và giá trị');
expectError('T1 fare âm', () => db.trips.insertOne({ ...goodTrip(), fare: -1000 }), 121, 'minimum');
expectError('T2 fare có phần lẻ 45000.5', () => db.trips.insertOne({ ...goodTrip(), fare: 45000.5 }), 121, 'bsonType');
expectError('T3 fare là chuỗi "45000"', () => db.trips.insertOne({ ...goodTrip(), fare: '45000' }), 121, 'bsonType');
expectError('T4 fare là Decimal128', () => db.trips.insertOne({ ...goodTrip(), fare: Decimal128('45000') }), 121, 'bsonType');
expectError('T5 kind taxi', () => db.trips.insertOne({ ...goodTrip(), kind: 'taxi' }), 121, 'enum');
expectError('T6 status done', () => db.trips.insertOne({ ...goodTrip(), status: 'done' }), 121, 'enum');
expectError('T7 code dưới 100000', () => db.trips.insertOne({ ...goodTrip(), code: 99999 }), 121, 'minimum');
expectError('T8 code trùng 100004', () => db.trips.insertOne({ ...goodTrip(), code: 100004 }), 11000, 'uq_trips_code');
expectError('T9 customer_id là chuỗi UUID', () => db.trips.insertOne({ ...goodTrip(), customer_id: 'c0000000-0000-4000-8000-000000000001' }), 121, 'customer_id');
expectError('T10 thiếu events', () => db.trips.insertOne(without(goodTrip(), 'events')), 121, 'events');
expectError('T11 event có status lạ', () => db.trips.insertOne({ ...goodTrip(), events: [{ status: 'xyz', at: vn('2026-11-03 08:40') }] }), 121, 'enum');
expectError('T12 driver_id null thay vì vắng', () => db.trips.insertOne({ ...goodTrip(), driver_id: null }), 121, 'driver_id');

section('trips: quy tắc liên trường');
expectError('R1a food không có quán', () => db.trips.insertOne({ ...goodTrip(), kind: 'food' }), 121, 1);
expectError('R1b ride có quán', () => db.trips.insertOne({ ...goodTrip(), merchant_id: M(1) }), 121, 1);
expectError('R2a requested mà đã có tài xế', () => db.trips.insertOne({ ...goodTrip(), driver_id: D(8003), accepted_at: vn('2026-11-03 08:41') }), 121, 2);
expectError('R2b accepted mà chưa có tài xế', () => db.trips.insertOne({ ...goodTrip(), status: 'accepted' }), 121, 2);
expectError('R3 có tài xế, thiếu accepted_at', () => db.trips.insertOne({ ...goodTrip(), status: 'accepted', driver_id: D(8003) }), 121, 3);
expectError('R4 completed thiếu completed_at', () => db.trips.insertOne({ ...goodTrip(), status: 'completed', driver_id: D(8003), accepted_at: vn('2026-11-03 08:41') }), 121, 4);
expectError('R5 hoàn thành trước khi nhận', () => db.trips.insertOne({ ...goodTrip(), status: 'completed', driver_id: D(8003), accepted_at: vn('2026-11-03 08:41'), completed_at: vn('2026-11-03 08:30') }), 121, 5);

section('Chi tiết lỗi validation (errInfo.details) của R4');
const e = expectError('R4 lần nữa, đọc chi tiết', () => db.trips.insertOne({ ...goodTrip(), status: 'completed', driver_id: D(8003), accepted_at: vn('2026-11-03 08:41') }), 121);
const det = e.errInfo.details;
print('failingDocumentId có mặt (ở errInfo, không ở details): ' + (e.errInfo.failingDocumentId !== undefined));
print('operatorName gốc: ' + det.operatorName);
print('các điều kiện $and không thoả: ' + det.clausesNotSatisfied.map((c) => `#${c.index}`).join(', '));

section('update cũng bị kiểm');
expectError('U1 $inc làm fare âm (chuyến 100014)', () => db.trips.updateOne({ code: 100014 }, { $inc: { fare: -100000 } }), 121, 'minimum');
expectError('U2 $set completed cho chuyến đang chạy 100010', () => db.trips.updateOne({ code: 100010 }, { $set: { status: 'completed' } }), 121, 4);
expectError('U3 $set trùng phone khách 2 cho khách 3', () => db.customers.updateOne({ _id: C(3) }, { $set: { phone: '0900000002' } }), 11000, 'uq_customers_phone');
expectError('U4 $unset trường bắt buộc full_name', () => db.drivers.updateOne({ code: 8006 }, { $unset: { full_name: '' } }), 121, 'full_name');

section('Bẫy: KHÔNG có lỗi, dữ liệu sai lọt vào');
// B1: không có khoá ngoại — customer_id trỏ tới khách không tồn tại vẫn được nhận.
let x = { ...goodTrip(), customer_id: UUID('c0000000-0000-4000-8000-0000000000ff') };
db.trips.insertOne(x);
expectValue('B1 chuyến của khách không tồn tại đã vào', db.trips.countDocuments({ _id: x._id }), 1);
db.trips.deleteOne({ _id: x._id });

// B2: city_id 'XX' đúng pattern nhưng không có thành phố XX.
x = { ...goodDriver(), city_id: 'XX' };
db.drivers.insertOne(x);
expectValue('B2 tài xế ở thành phố không tồn tại đã vào', db.drivers.countDocuments({ _id: x._id }), 1);
db.drivers.deleteOne({ _id: x._id });

// B3: ghi vào collection gõ sai tên — MongoDB tự tạo collection mới, không báo gì.
db.trip.insertOne(goodTrip());
expectValue('B3 collection "trip" (thiếu s) được tự tạo', db.getCollectionNames().includes('trip'), true);
db.trip.drop();

// B4: bypassDocumentValidation bỏ qua validator (cần quyền bypassDocumentValidation).
x = { ...goodTrip(), fare: -5 };
db.trips.insertOne(x, { bypassDocumentValidation: true });
expectValue('B4 fare âm lọt qua bypass', db.trips.countDocuments({ _id: x._id }), 1);
db.trips.deleteOne({ _id: x._id });

// B5: tài xế 8003 "online" nhưng được gán vào chuyến đang chạy — trạng thái nằm ở
// collection khác, validator của một collection không nhìn thấy collection khác.
// (Không ghi thử: 07-checks.js kiểm bất biến này trên dữ liệu thật.)

section('Dữ liệu sau các ca sai: không đổi');
table([{
  cities: db.cities.countDocuments(), customers: db.customers.countDocuments(), drivers: db.drivers.countDocuments(),
  merchants: db.merchants.countDocuments(), trips: db.trips.countDocuments(),
  fare_100014: db.trips.findOne({ code: 100014 }).fare, status_100010: db.trips.findOne({ code: 100010 }).status,
}]);
expectValue('số chuyến', db.trips.countDocuments(), 15);
