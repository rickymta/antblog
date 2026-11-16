// P01 · 06-migration.js — thêm biển số xe cho tài xế khi không có ALTER TABLE (MG-009).
//
// Trong MongoDB "schema" là (1) hình dạng document đang có trên đĩa và (2) validator.
// Đổi validator KHÔNG sửa document cũ; sửa document cũ KHÔNG đổi validator.
// Vì vậy migration đi theo expand → backfill → contract, mỗi bước kiểm được:
//   1. expand  : validator cho phép trường mới (chưa bắt buộc) + schema_version
//   2. backfill: ghi biển số cho tài xế cũ theo lô, chuẩn hoá dữ liệu nguồn bẩn
//   3. kiểm    : đếm document KHÔNG thoả validator đích ($nor) phải = 0
//   4. contract: validator đích bắt buộc trường mới, strict + error
// Hoàn tác: 06-migration-rollback.js.

const D = (n) => UUID(`d0000000-0000-4000-8000-00000000${n}`);
const PLATE = { bsonType: 'string', pattern: '^[0-9]{2}[A-Z][A-Z0-9]?-[0-9]{3}\\.[0-9]{2}$', description: 'biển số dạng 59X3-123.45 / 51G-123.45' };

// Đọc validator hiện tại từ catalog (không chép tay lại 01-schema.js).
const v1 = db.getCollectionInfos({ name: 'drivers' })[0].options.validator.$jsonSchema;
const clone = (o) => EJSON.parse(EJSON.stringify(o));

function schemaV2({ requirePlate }) {
  const s = clone(v1);
  s.properties.vehicle_plate = PLATE;
  s.properties.schema_version = { bsonType: 'int', enum: [2] };
  if (requirePlate) s.required = [...s.required, 'vehicle_plate', 'schema_version'];
  return { $jsonSchema: s };
}

section('Bước 1 · expand: cho phép vehicle_plate, chưa bắt buộc');
db.runCommand({ collMod: 'drivers', validator: schemaV2({ requirePlate: false }), validationLevel: 'strict', validationAction: 'error' });
// Phiên bản ứng dụng cũ (chưa biết biển số) vẫn ghi được:
db.drivers.updateOne({ code: 8003 }, { $set: { status: 'online' } });
print('PASS app cũ cập nhật tài xế 8003 không có biển số');
// Phiên bản mới ghi tài xế có biển số:
db.drivers.insertOne({
  _id: D(8008), code: 8008, full_name: 'Kiều Thị Sen', city_id: 'HN', vehicle_type: 'car', status: 'offline',
  created_at: vn('2026-11-03 09:30'), vehicle_plate: '30K-456.78', schema_version: 2,
});
expectError('biển số sai định dạng bị chặn ngay từ expand', () => db.drivers.updateOne({ code: 8008 }, { $set: { vehicle_plate: '30K45678' } }), 121, 'pattern');

section('Bước 2 · backfill theo lô (bulkWrite), dữ liệu nguồn có dòng bẩn');
// Biển số lấy từ hệ thống đăng ký xe cũ. Dòng 8004 viết thường, có khoảng trắng.
const source = [
  [8001, '59X3-123.45'], [8002, '59X1-678.90'], [8003, '29B1-246.80'],
  [8004, ' 30h-135.79 '], [8005, '43D1-112.23'], [8006, '43A-998.87'], [8007, '59X2-555.01'],
];
// ordered:false ⇒ server chạy HẾT lô, dòng lỗi không chặn các dòng sau; mongosh
// ném MongoBulkWriteError ở cuối, kèm số đã ghi và danh sách lỗi theo vị trí.
try {
  db.drivers.bulkWrite(
    source.map(([code, plate]) => ({ updateOne: {
      filter: { code, vehicle_plate: { $exists: false } },
      update: { $set: { vehicle_plate: plate, schema_version: 2 } },
    } })),
    { ordered: false },
  );
  print('FAIL lô đầu không có lỗi, mong đợi dòng 8004 bị chặn');
} catch (e) {
  expectValue('lần 1: số tài xế đã ghi biển số', e.result.modifiedCount, 6);
  expectValue('lần 1: lỗi tại vị trí (mã lỗi)', e.writeErrors.map((w) => `${w.index} (${w.code})`).join(', '), '3 (121)');
}
const left = db.drivers.find({ vehicle_plate: { $exists: false } }, { _id: 0, code: 1 }).toArray().map((d) => d.code);
expectValue('còn thiếu biển số', left.join(','), '8004');
// Xử lý riêng dòng lỗi: chuẩn hoá rồi ghi lại.
for (const [code, raw] of source.filter(([c]) => left.includes(c))) {
  const plate = raw.trim().toUpperCase();
  db.drivers.updateOne({ code, vehicle_plate: { $exists: false } }, { $set: { vehicle_plate: plate, schema_version: 2 } });
}
expectValue('8004 sau chuẩn hoá', db.drivers.findOne({ code: 8004 }).vehicle_plate, '30H-135.79');
// Chạy lại backfill lần 2 không sửa gì: bộ lọc $exists:false làm nó idempotent.
const again = db.drivers.bulkWrite(source.map(([code, plate]) => ({ updateOne: {
  filter: { code, vehicle_plate: { $exists: false } }, update: { $set: { vehicle_plate: plate.trim().toUpperCase(), schema_version: 2 } },
} }))).modifiedCount;
expectValue('backfill lần 2 (idempotent)', again, 0);

section('Bước 3 · document không thoả validator đích');
const target = schemaV2({ requirePlate: true });
expectValue('drivers không thoả validator đích', db.drivers.countDocuments({ $nor: [target] }), 0);

section('validationAction "warn": ghi được, chỉ để lại log');
db.runCommand({ collMod: 'drivers', validator: target, validationLevel: 'strict', validationAction: 'warn' });
const warned = { _id: UUID(), code: 8099, full_name: 'Thiếu Biển Số', city_id: 'SG', vehicle_type: 'bike', status: 'offline', created_at: vn('2026-11-03 09:40') };
db.drivers.insertOne(warned);
const logLine = db.adminCommand({ getLog: 'global' }).log.map((l) => JSON.parse(l)).reverse()
  .find((l) => l.id === 20294 && l.attr?.namespace === 'toinoi.drivers');
expectValue('log id 20294 "Document would fail validation"', logLine?.msg, 'Document would fail validation');
db.drivers.deleteOne({ _id: warned._id });

section('Bước 4 · contract: bắt buộc biển số, strict + error');
db.runCommand({ collMod: 'drivers', validator: target, validationLevel: 'strict', validationAction: 'error' });
expectError('tài xế mới thiếu biển số', () => db.drivers.insertOne(warned), 121, 'vehicle_plate');
expectError('$unset biển số của 8001', () => db.drivers.updateOne({ code: 8001 }, { $unset: { vehicle_plate: '' } }), 121, 'vehicle_plate');

section('validate: kiểm toàn bộ collection với validator hiện tại');
const v = db.drivers.validate();
table([{ valid: v.valid, nrecords: v.nrecords, nNonCompliantDocuments: v.nNonCompliantDocuments }]);
expectValue('nNonCompliantDocuments', v.nNonCompliantDocuments, 0);

section('Tài xế sau migration');
table(db.drivers.find({}, { _id: 0, code: 1, full_name: 1, vehicle_type: 1, vehicle_plate: 1, schema_version: 1 }).sort({ code: 1 }),
  ['code', 'full_name', 'vehicle_type', 'vehicle_plate', 'schema_version']);
