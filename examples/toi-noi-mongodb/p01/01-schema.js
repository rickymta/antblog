// P01 · 01-schema.js — collection, validator và index cho Tới Nơi (MG-003, MG-004, MG-005).
//
// MongoDB không bắt buộc schema: thiếu validator, collection nhận mọi document.
// Lab khai báo validator ngay từ đầu, gồm hai lớp:
//   * $jsonSchema: kiểu BSON, trường bắt buộc, enum, khoảng giá trị, cấm trường lạ;
//   * toán tử truy vấn ($or, $expr): quy tắc liên trường (đồ ăn ⇔ có quán …).
// validationLevel "strict" + validationAction "error": mọi insert/update đều bị kiểm.
//
// Định danh: cities dùng mã ngắn làm _id; khách, tài xế, quán, chuyến dùng UUID
// (BSON binary subtype 4) — cùng giá trị với lab PostgreSQL/SQL Server để đối chiếu.
// Chuyến có thêm `code` số nguyên tăng dần cho người đọc, cấp bằng collection `counters`.

const uuid = { bsonType: 'binData' };
const date = { bsonType: 'date' };
const cityId = { bsonType: 'string', pattern: '^[A-Z]{2}$' };

db.createCollection('cities', {
  validator: { $jsonSchema: {
    bsonType: 'object', required: ['_id', 'name', 'tz'], additionalProperties: false,
    properties: {
      _id: cityId,
      name: { bsonType: 'string', minLength: 1 },
      tz: { enum: ['Asia/Ho_Chi_Minh'] },
    },
  } },
});

db.createCollection('customers', {
  validator: { $jsonSchema: {
    bsonType: 'object', required: ['_id', 'phone', 'full_name', 'created_at'], additionalProperties: false,
    properties: {
      _id: uuid,
      phone: { bsonType: 'string', pattern: '^0[0-9]{9}$', description: 'số di động VN 10 chữ số, lưu dạng chuỗi' },
      full_name: { bsonType: 'string', minLength: 1, maxLength: 100 },
      created_at: date,
    },
  } },
});

db.createCollection('drivers', {
  validator: { $jsonSchema: {
    bsonType: 'object',
    required: ['_id', 'code', 'full_name', 'city_id', 'vehicle_type', 'status', 'created_at'],
    additionalProperties: false,
    properties: {
      _id: uuid,
      code: { bsonType: 'int', minimum: 1000, maximum: 99999 },
      full_name: { bsonType: 'string', minLength: 1, maxLength: 100 },
      city_id: cityId,
      vehicle_type: { enum: ['bike', 'car'] },
      status: { enum: ['offline', 'online', 'busy'] },
      created_at: date,
    },
  } },
});

db.createCollection('merchants', {
  validator: { $jsonSchema: {
    bsonType: 'object', required: ['_id', 'city_id', 'name', 'menu', 'created_at'], additionalProperties: false,
    properties: {
      _id: uuid,
      city_id: cityId,
      name: { bsonType: 'string', minLength: 1 },
      // Menu NHÚNG trong quán: luôn đọc cùng quán, ít món, sửa cùng lúc (MG-008).
      menu: {
        bsonType: 'array', maxItems: 200,
        items: {
          bsonType: 'object', required: ['sku', 'name', 'price'], additionalProperties: false,
          properties: {
            sku: { bsonType: 'string', pattern: '^[A-Z0-9-]{3,20}$' },
            name: { bsonType: 'string', minLength: 1 },
            price: { bsonType: 'int', minimum: 0 },
          },
        },
      },
      created_at: date,
    },
  } },
});

const STATUSES = ['requested', 'searching', 'accepted', 'picked_up', 'in_transit', 'arriving', 'completed', 'cancelled'];

db.createCollection('trips', {
  validator: {
    $and: [
      { $jsonSchema: {
        bsonType: 'object',
        required: ['_id', 'code', 'kind', 'city_id', 'customer_id', 'status', 'fare', 'requested_at', 'updated_at', 'events'],
        additionalProperties: false,
        properties: {
          _id: uuid,
          code: { bsonType: 'int', minimum: 100000 },
          kind: { enum: ['ride', 'food', 'parcel'] },
          city_id: cityId,
          customer_id: uuid,
          driver_id: uuid,
          merchant_id: uuid,
          status: { enum: STATUSES },
          // Tiền: số nguyên đồng. int hoặc long, KHÔNG double (MG-004).
          fare: { bsonType: ['int', 'long'], minimum: 0, maximum: 10000000 },
          requested_at: date,
          accepted_at: date,
          completed_at: date,
          updated_at: date,
          // Lịch sử trạng thái NHÚNG trong chuyến: có giới hạn, luôn đọc cùng chuyến (MG-006, MG-008).
          events: {
            bsonType: 'array', maxItems: 20,
            items: {
              bsonType: 'object', required: ['status', 'at'], additionalProperties: false,
              properties: { status: { enum: STATUSES }, at: date, by: { enum: ['customer', 'driver', 'system'] } },
            },
          },
        },
      } },
      // R1: đồ ăn ⇔ có quán.
      { $or: [
        { kind: 'food', merchant_id: { $exists: true } },
        { kind: { $in: ['ride', 'parcel'] }, merchant_id: { $exists: false } },
      ] },
      // R2: từ accepted tới completed phải có tài xế; requested/searching thì chưa có; cancelled tuỳ.
      { $or: [
        { status: { $in: ['requested', 'searching'] }, driver_id: { $exists: false } },
        { status: { $in: ['accepted', 'picked_up', 'in_transit', 'arriving', 'completed'] }, driver_id: { $exists: true } },
        { status: 'cancelled' },
      ] },
      // R3: có tài xế ⇔ có accepted_at.
      { $or: [
        { driver_id: { $exists: true }, accepted_at: { $exists: true } },
        { driver_id: { $exists: false }, accepted_at: { $exists: false } },
      ] },
      // R4: completed ⇔ có completed_at.
      { $or: [
        { status: 'completed', completed_at: { $exists: true } },
        { status: { $ne: 'completed' }, completed_at: { $exists: false } },
      ] },
      // R5: thứ tự thời gian — so sánh hai trường của cùng document phải dùng $expr.
      { $expr: { $and: [
        { $or: [{ $eq: [{ $type: '$accepted_at' }, 'missing'] }, { $gte: ['$accepted_at', '$requested_at'] }] },
        { $or: [{ $eq: [{ $type: '$completed_at' }, 'missing'] }, { $gte: ['$completed_at', '$accepted_at'] }] },
        { $gte: ['$updated_at', '$requested_at'] },
      ] } },
    ],
  },
  validationLevel: 'strict',
  validationAction: 'error',
});

// Bộ đếm cấp mã chuyến (MongoDB không có IDENTITY/SEQUENCE).
db.createCollection('counters');

section('Index');
// _id luôn có index unique sẵn. Các index unique còn lại giữ "khoá tự nhiên".
db.customers.createIndex({ phone: 1 }, { unique: true, name: 'uq_customers_phone' });
db.drivers.createIndex({ code: 1 }, { unique: true, name: 'uq_drivers_code' });
db.trips.createIndex({ code: 1 }, { unique: true, name: 'uq_trips_code' });
// Index phục vụ truy vấn của P01 (Phần 3 đo và chọn index bài bản).
db.trips.createIndex({ customer_id: 1, requested_at: -1 }, { name: 'ix_trips_customer_requested' });
db.trips.createIndex({ driver_id: 1, status: 1 }, { name: 'ix_trips_driver_status' });

const idx = ['cities', 'customers', 'drivers', 'merchants', 'trips'].flatMap((c) =>
  db.getCollection(c).getIndexes().map((i) => ({ collection: c, name: i.name, key: i.key, unique: i.unique === true || i.name === '_id_' })));
table(idx, ['collection', 'name', 'key', 'unique']);
expectValue('số index (kể cả _id)', idx.length, 10);

section('Collection và validator');
table(db.getCollectionInfos({ type: 'collection' })
  .sort((a, b) => a.name.localeCompare(b.name))
  .map((c) => ({ name: c.name, co_validator: !!c.options.validator, validationLevel: c.options.validationLevel ?? '(mặc định)', validationAction: c.options.validationAction ?? '(mặc định)' })),
  ['name', 'co_validator', 'validationLevel', 'validationAction']);
expectValue('collection có validator', db.getCollectionInfos().filter((c) => c.options.validator).length, 5);
