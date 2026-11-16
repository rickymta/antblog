// P01 · 07-checks.js — bất biến liên collection, kiểm toàn bộ validator, kiểm kê (MG-008, MG-010).
//
// Validator chỉ nhìn MỘT document. Mọi luật cần nhìn document khác (tham chiếu,
// trạng thái tài xế vs chuyến, bộ đếm) phải kiểm bằng truy vấn — ở đây là
// aggregation, mỗi kiểm tra trả về danh sách vi phạm và mong đợi rỗng.

function check(label, coll, pipeline) {
  const bad = db.getCollection(coll).aggregate(pipeline).toArray();
  expectValue(label, bad.length, 0);
  if (bad.length) table(bad);
}
// Bước $lookup chỉ lấy _id của bên được tham chiếu; mảng rỗng = mồ côi.
const missing = (from, local) => [
  { $match: { [local]: { $exists: true } } },
  { $lookup: { from, localField: local, foreignField: '_id', as: 'ref', pipeline: [{ $project: { _id: 1 } }] } },
  { $match: { ref: { $size: 0 } } },
  { $project: { _id: 0, code: 1, [local]: 1 } },
];
const ACTIVE = ['accepted', 'picked_up', 'in_transit', 'arriving'];

section('Tham chiếu (thay cho khoá ngoại)');
check('K1 chuyến trỏ tới khách không tồn tại', 'trips', missing('customers', 'customer_id'));
check('K2 chuyến trỏ tới tài xế không tồn tại', 'trips', missing('drivers', 'driver_id'));
check('K3 chuyến trỏ tới quán không tồn tại', 'trips', missing('merchants', 'merchant_id'));
for (const c of ['trips', 'drivers', 'merchants']) check(`K4 ${c}.city_id không có trong cities`, c, missing('cities', 'city_id'));

section('Luật nghiệp vụ liên collection');
check('K5 quán khác thành phố với chuyến đồ ăn', 'trips', [
  { $match: { kind: 'food' } },
  { $lookup: { from: 'merchants', localField: 'merchant_id', foreignField: '_id', as: 'm' } },
  { $match: { $expr: { $ne: ['$city_id', { $first: '$m.city_id' }] } } },
  { $project: { _id: 0, code: 1, city_id: 1, merchant_city: { $first: '$m.city_id' } } },
]);
check('K6 tài xế khác thành phố với chuyến', 'trips', [
  { $match: { driver_id: { $exists: true } } },
  { $lookup: { from: 'drivers', localField: 'driver_id', foreignField: '_id', as: 'd' } },
  { $match: { $expr: { $ne: ['$city_id', { $first: '$d.city_id' }] } } },
  { $project: { _id: 0, code: 1, city_id: 1, driver_city: { $first: '$d.city_id' } } },
]);
check('K7 tài xế: busy ⇔ đúng một chuyến đang chạy', 'drivers', [
  { $lookup: { from: 'trips', localField: '_id', foreignField: 'driver_id', as: 'act',
    pipeline: [{ $match: { status: { $in: ACTIVE } } }, { $project: { code: 1 } }] } },
  { $project: { _id: 0, code: 1, status: 1, n_active: { $size: '$act' } } },
  { $match: { $or: [{ n_active: { $gt: 1 } }, { status: 'busy', n_active: 0 }, { status: { $ne: 'busy' }, n_active: { $gt: 0 } }] } },
]);

section('Luật trong một document mà validator khó diễn đạt');
check('K8 sự kiện cuối khác trạng thái hiện tại', 'trips', [
  { $match: { 'events.0': { $exists: true } } },
  { $match: { $expr: { $ne: [{ $last: '$events.status' }, '$status'] } } },
  { $project: { _id: 0, code: 1, status: 1 } },
]);
check('K9 sự kiện không theo thứ tự thời gian', 'trips', [
  { $match: { 'events.1': { $exists: true } } },
  { $match: { $expr: { $ne: ['$events.at', { $sortArray: { input: '$events.at', sortBy: 1 } }] } } },
  { $project: { _id: 0, code: 1 } },
]);
const maxCode = db.trips.find().sort({ code: -1 }).limit(1).next().code;
const seq = db.counters.findOne({ _id: 'trip_code' }).seq;
expectValue('K10 bộ đếm ≥ mã chuyến lớn nhất', seq >= maxCode, true);

section('Toàn bộ document thoả validator hiện tại (validate)');
const rows = ['cities', 'customers', 'drivers', 'merchants', 'trips'].map((c) => {
  const v = db.getCollection(c).validate();
  return { collection: c, valid: v.valid, nrecords: v.nrecords, nNonCompliantDocuments: v.nNonCompliantDocuments };
});
table(rows);
expectValue('K11 tổng document không thoả validator', rows.reduce((s, r) => s + r.nNonCompliantDocuments, 0), 0);

section('Index unique giữ khoá tự nhiên');
const uniq = ['customers', 'drivers', 'trips'].flatMap((c) => db.getCollection(c).getIndexes().filter((i) => i.unique).map((i) => `${c}.${i.name}`));
expectValue('K12 index unique ngoài _id', uniq.join(', '), 'customers.uq_customers_phone, drivers.uq_drivers_code, trips.uq_trips_code');

section('Đối soát nhanh: doanh thu chuyến hoàn thành theo thành phố');
table(db.trips.aggregate([
  { $match: { status: 'completed' } },
  { $group: { _id: '$city_id', so_chuyen: { $sum: 1 }, doanh_thu: { $sum: '$fare' } } },
  { $lookup: { from: 'cities', localField: '_id', foreignField: '_id', as: 'c' } },
  { $project: { _id: 0, city: '$_id', ten: { $first: '$c.name' }, so_chuyen: 1, doanh_thu: 1, kieu_tong: { $type: '$doanh_thu' } } },
  { $sort: { city: 1 } },
]), ['city', 'ten', 'so_chuyen', 'doanh_thu', 'kieu_tong']);
const total = db.trips.aggregate([{ $match: { status: 'completed' } }, { $group: { _id: null, s: { $sum: '$fare' } } }]).next().s;
expectValue('tổng doanh thu chuyến hoàn thành (đ)', total, 397000);

section('Kiểm kê cuối P01');
table([Object.fromEntries(['cities', 'customers', 'drivers', 'merchants', 'trips', 'counters'].map((c) => [c, db.getCollection(c).countDocuments()]))]);
const stats = db.trips.stats();
table([{ collection: 'trips', count: stats.count, avgObjSize: stats.avgObjSize, nindexes: stats.nindexes }]);
