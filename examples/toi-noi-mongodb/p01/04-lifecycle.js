// P01 · 04-lifecycle.js — insert, update, delete trên một chuyến mới (MG-006).
//
// Ý chính: trạng thái và lịch sử nằm trong CÙNG một document, nên một updateOne
// đổi trạng thái và $push sự kiện là một thao tác nguyên tử — không cần transaction.
// Bộ lọc của update chứa trạng thái hiện tại ⇒ chuyển trạng thái sai thứ tự
// (hoặc hai tài xế tranh nhau) khớp 0 document, không ghi gì.

const C = (n) => UUID(`c0000000-0000-4000-8000-00000000000${n}`);
const D = (n) => UUID(`d0000000-0000-4000-8000-00000000${n}`);

section('Thêm tài xế mới 8007 (insertOne)');
const ins = db.drivers.insertOne({
  _id: D(8007), code: 8007, full_name: 'Mai Văn Quý', city_id: 'SG',
  vehicle_type: 'bike', status: 'online', created_at: vn('2026-11-03 08:31'),
});
expectValue('acknowledged', ins.acknowledged, true);
expectValue('insertedId', ins.insertedId, 'd0000000-0000-4000-8000-000000008007');

section('Cấp mã chuyến bằng bộ đếm (findOneAndUpdate + $inc, trả bản SAU khi sửa)');
const counter = db.counters.findOneAndUpdate({ _id: 'trip_code' }, { $inc: { seq: 1 } }, { returnDocument: 'after', upsert: true });
const code = counter.seq;
expectValue('mã chuyến mới', code, 100015);

section('Khách 1 đặt xe ở SG lúc 08:40');
const id = UUID('70000000-0000-4000-8000-000000100015');
const t0 = vn('2026-11-03 08:40');
db.trips.insertOne({
  _id: id, code, kind: 'ride', city_id: 'SG', customer_id: C(1), status: 'requested',
  fare: 41000, requested_at: t0, updated_at: t0, events: [{ status: 'requested', at: t0, by: 'customer' }],
});

// Một bước chuyển trạng thái: chỉ khớp khi trạng thái hiện tại đúng `from`.
function step(from, to, at, by, extra = {}) {
  const when = vn(at);
  return db.trips.updateOne(
    { _id: id, status: from },
    { $set: { status: to, updated_at: when, ...extra }, $push: { events: { status: to, at: when, by } } },
  );
}

let r = step('requested', 'searching', '2026-11-03 08:40:05', 'system');
expectValue('requested → searching: matched/modified', `${r.matchedCount}/${r.modifiedCount}`, '1/1');

section('Hai tài xế cùng bấm nhận (8007 trước, 8003 sau)');
r = step('searching', 'accepted', '2026-11-03 08:41', 'driver', { driver_id: D(8007), accepted_at: vn('2026-11-03 08:41') });
expectValue('8007 nhận: matched', r.matchedCount, 1);
r = step('searching', 'accepted', '2026-11-03 08:41:01', 'driver', { driver_id: D(8003), accepted_at: vn('2026-11-03 08:41:01') });
expectValue('8003 nhận muộn: matched (không ghi gì)', r.matchedCount, 0);
expectValue('tài xế của chuyến', db.trips.findOne({ _id: id }).driver_id, 'd0000000-0000-4000-8000-000000008007');

// Tài xế chuyển busy: document KHÁC, thao tác riêng — không nguyên tử cùng chuyến (Phần 4).
db.drivers.updateOne({ _id: D(8007), status: 'online' }, { $set: { status: 'busy' } });

section('Đi tiếp, bước cuối dùng findOneAndUpdate để nhận document sau khi sửa');
step('accepted', 'picked_up', '2026-11-03 08:47', 'driver');
step('picked_up', 'in_transit', '2026-11-03 08:47:30', 'driver');
step('in_transit', 'arriving', '2026-11-03 09:08', 'system');
const done = db.trips.findOneAndUpdate(
  { _id: id, status: 'arriving' },
  { $set: { status: 'completed', completed_at: vn('2026-11-03 09:11'), updated_at: vn('2026-11-03 09:11') },
    $push: { events: { status: 'completed', at: vn('2026-11-03 09:11'), by: 'driver' } } },
  { returnDocument: 'after', projection: { _id: 0, code: 1, status: 1, fare: 1, completed_at: 1, n_events: { $size: '$events' } } },
);
table([done], ['code', 'status', 'fare', 'completed_at', 'n_events']);
db.drivers.updateOne({ _id: D(8007), status: 'busy' }, { $set: { status: 'online' } });

section('Lịch sử trạng thái nhúng trong chuyến');
table(db.trips.aggregate([
  { $match: { _id: id } }, { $unwind: { path: '$events', includeArrayIndex: 'i' } },
  { $project: { _id: 0, i: 1, status: '$events.status', at: '$events.at', by: '$events.by' } },
]), ['i', 'status', 'at', 'by']);
const t = db.trips.findOne({ _id: id });
expectValue('số sự kiện', t.events.length, 7);
expectValue('phút từ đặt tới hoàn thành', (t.completed_at - t.requested_at) / 60000, 31);

section('Cập nhật sai thứ tự: không lỗi, chỉ khớp 0 document');
r = step('searching', 'cancelled', '2026-11-03 09:12', 'customer');
expectValue('huỷ chuyến đã hoàn thành: matched', r.matchedCount, 0);

section('updateMany và upsert');
r = db.drivers.updateMany({ city_id: 'DN', status: 'offline' }, { $set: { status: 'offline' } });
expectValue('updateMany: matched/modified (giá trị không đổi ⇒ không sửa)', `${r.matchedCount}/${r.modifiedCount}`, '1/0');
r = db.counters.updateOne({ _id: 'parcel_code' }, { $setOnInsert: { seq: 500000 } }, { upsert: true });
expectValue('upsert tạo bộ đếm mới: upsertedCount', r.upsertedCount, 1);
db.counters.deleteOne({ _id: 'parcel_code' });

section('replaceOne thay CẢ document: quên trường là mất trường');
const before = db.customers.findOne({ _id: C(6) });
expectError('replaceOne thiếu created_at', () => db.customers.replaceOne({ _id: C(6) }, { phone: before.phone, full_name: 'Đặng Ngọc Hà' }), 121, 'created_at');

section('Xoá: không có khoá ngoại chặn');
// Khách 6 có chuyến 100007 và 100014. deleteOne vẫn thành công ⇒ hai chuyến mồ côi.
const removed = db.customers.findOneAndDelete({ _id: C(6) });
expectValue('đã xoá khách', removed.full_name, 'Đặng Ngọc Hà');
const orphans = db.trips.countDocuments({ customer_id: C(6) });
expectValue('chuyến trỏ tới khách vừa xoá', orphans, 2);
// findOneAndDelete trả lại document đã xoá ⇒ khôi phục được nguyên trạng.
db.customers.insertOne(removed);
expectValue('khôi phục khách 6', db.customers.countDocuments({ _id: C(6) }), 1);

// Mẫu xoá an toàn ở tầng ứng dụng: kiểm tham chiếu trước (vẫn có khe hở đồng thời — Phần 4).
function deleteCustomerIfUnused(cid) {
  if (db.trips.countDocuments({ customer_id: cid }, { limit: 1 }) > 0) return 'bị chặn: còn chuyến';
  return `đã xoá ${db.customers.deleteOne({ _id: cid }).deletedCount}`;
}
expectValue('xoá an toàn khách 6', deleteCustomerIfUnused(C(6)), 'bị chặn: còn chuyến');

section('Số document sau vòng đời');
table([{ customers: db.customers.countDocuments(), drivers: db.drivers.countDocuments(), trips: db.trips.countDocuments(), counter: db.counters.findOne({ _id: 'trip_code' }).seq }]);
