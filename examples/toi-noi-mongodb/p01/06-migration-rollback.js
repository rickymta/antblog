// P01 · 06-migration-rollback.js — hoàn tác 06-migration.js (MG-009). Chạy lặp được.
//
// Thứ tự ngược với migration và KHÔNG được đảo:
//   1. nới validator (biển số không bắt buộc) — nếu không, bước 3 bị chính validator chặn;
//   2. sao lưu biển số sang drivers_plate_backup (upsert theo _id, chạy lại không nhân đôi);
//   3. $unset vehicle_plate, schema_version;
//   4. đặt lại validator v1 (không có hai trường mới, additionalProperties: false).
// Tài xế 8008 được tạo bởi phiên bản mới — giữ lại, chỉ bỏ hai trường.

const cur = db.getCollectionInfos({ name: 'drivers' })[0].options.validator.$jsonSchema;
const v1 = EJSON.parse(EJSON.stringify(cur));
delete v1.properties.vehicle_plate;
delete v1.properties.schema_version;
v1.required = v1.required.filter((f) => f !== 'vehicle_plate' && f !== 'schema_version');

const lenient = EJSON.parse(EJSON.stringify(cur));
lenient.required = v1.required;
db.runCommand({ collMod: 'drivers', validator: { $jsonSchema: lenient }, validationLevel: 'strict', validationAction: 'error' });

const backup = db.drivers.find({ vehicle_plate: { $exists: true } }, { vehicle_plate: 1, code: 1 }).toArray();
for (const d of backup) {
  db.drivers_plate_backup.updateOne({ _id: d._id }, { $set: { code: d.code, vehicle_plate: d.vehicle_plate, saved_at: new Date() } }, { upsert: true });
}
const unset = db.drivers.updateMany({}, { $unset: { vehicle_plate: '', schema_version: '' } });
db.runCommand({ collMod: 'drivers', validator: { $jsonSchema: v1 }, validationLevel: 'strict', validationAction: 'error' });

expectValue('R1 còn tài xế có vehicle_plate', db.drivers.countDocuments({ vehicle_plate: { $exists: true } }), 0);
expectValue('R2 số biển số đã sao lưu', db.drivers_plate_backup.countDocuments(), 8);
expectValue('R3 validator không còn vehicle_plate', 'vehicle_plate' in db.getCollectionInfos({ name: 'drivers' })[0].options.validator.$jsonSchema.properties, false);
expectValue('R4 drivers không thoả validator v1', db.drivers.validate().nNonCompliantDocuments, 0);
print(`(lần này $unset sửa ${unset.modifiedCount} document)`);
