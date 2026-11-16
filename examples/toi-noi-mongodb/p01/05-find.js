// P01 · 05-find.js — find: lọc, projection, sắp xếp, phân trang, null/thiếu, collation (MG-007).

const C = (n) => UUID(`c0000000-0000-4000-8000-00000000000${n}`);
const D = (n) => UUID(`d0000000-0000-4000-8000-00000000${n}`);

section('Chuyến của "ngày 02/11" theo giờ VN: khoảng nửa mở [00:00, 00:00 hôm sau)');
const from = vn('2026-11-02 00:00'), to = vn('2026-11-03 00:00');
table(db.trips.find(
  { requested_at: { $gte: from, $lt: to } },
  { _id: 0, code: 1, status: 1, requested_at: 1 },
).sort({ requested_at: 1, code: 1 }), ['code', 'status', 'requested_at']);
expectValue('chuyến ngày 02/11 giờ VN', db.trips.countDocuments({ requested_at: { $gte: from, $lt: to } }), 8);

section('Sai: lấy "ngày" theo UTC (mặc định của $dateToString)');
const utcDay = db.trips.countDocuments({ $expr: { $eq: [{ $dateToString: { date: '$requested_at', format: '%Y-%m-%d' } }, '2026-11-02'] } });
const vnDay = db.trips.countDocuments({ $expr: { $eq: [{ $dateToString: { date: '$requested_at', format: '%Y-%m-%d', timezone: 'Asia/Ho_Chi_Minh' } }, '2026-11-02'] } });
table([{ theo_utc: utcDay, theo_gio_vn: vnDay }]);
expectValue('ngày UTC đếm thừa 100008 (00:10 ngày 03/11 giờ VN)', `${utcDay} vs ${vnDay}`, '9 vs 8');

section('Sai: so Date với chuỗi — không lỗi, 0 kết quả');
expectValue('requested_at >= "2026-11-02"', db.trips.countDocuments({ requested_at: { $gte: '2026-11-02' } }), 0);
section('Sai: so UUID với chuỗi — không lỗi, 0 kết quả');
expectValue('customer_id = chuỗi UUID', db.trips.countDocuments({ customer_id: 'c0000000-0000-4000-8000-000000000001' }), 0);
expectValue('customer_id = UUID(...)', db.trips.countDocuments({ customer_id: C(1) }), 4);

section('Sắp xếp có giá trị trùng: 100003 và 100004 cùng 12:00');
table(db.trips.find({ requested_at: vn('2026-11-02 12:00') }, { _id: 0, code: 1, requested_at: 1 }).sort({ requested_at: 1, code: 1 }), ['code', 'requested_at']);
print('Chỉ sort theo requested_at thì thứ tự hai chuyến này không được đảm bảo; thêm code làm khoá phụ.');

section('Phân trang: trang 2, mỗi trang 5 chuyến, mới nhất trước');
table(db.trips.find({}, { _id: 0, code: 1, requested_at: 1 }).sort({ requested_at: -1, code: -1 }).skip(5).limit(5), ['code', 'requested_at']);

section('Projection: trường lồng nhau và cắt mảng');
table(db.merchants.find({}, { _id: 0, name: 1, 'menu.name': 1 }).sort({ name: 1 }), ['name', 'menu']);
table(db.trips.find({ code: 100015 }, { _id: 0, code: 1, events: { $slice: -2 } }), ['code', 'events']);

section('Lọc theo phần tử mảng');
expectValue('quán có món dưới 45.000 đ', db.merchants.countDocuments({ 'menu.price': { $lt: 45000 } }), 2);
expectValue('chuyến có sự kiện accepted bởi driver', db.trips.countDocuments({ events: { $elemMatch: { status: 'accepted', by: 'driver' } } }), 1);

section('null và trường vắng');
// Lab không bao giờ ghi driver_id: null (validator cấm) — chuyến chưa có tài xế đơn giản là không có trường.
table(db.trips.find({ driver_id: null }, { _id: 0, code: 1, status: 1 }).sort({ code: 1 }), ['code', 'status']);
expectValue('{driver_id: null} khớp cả trường vắng', db.trips.countDocuments({ driver_id: null }), 3);
expectValue('{driver_id: {$exists: false}}', db.trips.countDocuments({ driver_id: { $exists: false } }), 3);
// Ngược với NOT IN của SQL: $nin/$ne KHỚP cả document không có trường.
expectValue('$ne tài xế 8001 (gồm cả 3 chuyến chưa có tài xế)', db.trips.countDocuments({ driver_id: { $ne: D(8001) } }), 13);
expectValue('$ne 8001 và có tài xế', db.trips.countDocuments({ driver_id: { $ne: D(8001), $exists: true } }), 10);

section('Chuỗi tiếng Việt: sắp xếp và tìm không phân biệt dấu/hoa thường');
const names = (opt) => db.customers.find({}, { _id: 0, full_name: 1 }).sort({ full_name: 1 }).collation(opt).toArray().map((c) => c.full_name).join(' · ');
print('so byte (mặc định): ' + db.customers.find({}, { _id: 0, full_name: 1 }).sort({ full_name: 1 }).toArray().map((c) => c.full_name).join(' · '));
print('collation vi      : ' + names({ locale: 'vi' }));
expectValue('Đặng đứng cuối khi so byte', db.customers.find().sort({ full_name: 1 }).toArray().at(-1).full_name, 'Đặng Ngọc Hà');
expectValue('collation vi: Đ đứng sau D, trước L ⇒ Đặng đầu danh sách', db.customers.find().sort({ full_name: 1 }).collation({ locale: 'vi' }).toArray()[0].full_name, 'Đặng Ngọc Hà');
// Tìm: collation chỉ có tác dụng khi truyền vào truy vấn (hoặc đặt mặc định cho collection/index).
const count = (q, loc, s) => db.customers.countDocuments({ full_name: q }, loc ? { collation: { locale: loc, strength: s } } : {});
table([
  { truy_van: 'nguyen van an', khong_collation: count('nguyen van an'), vi_1: count('nguyen van an', 'vi', 1), en_1: count('nguyen van an', 'en', 1) },
  { truy_van: 'nguyên văn an', khong_collation: count('nguyên văn an'), vi_1: count('nguyên văn an', 'vi', 1), en_1: count('nguyên văn an', 'en', 1) },
  { truy_van: 'NGUYỄN VĂN AN', khong_collation: count('NGUYỄN VĂN AN'), vi_1: count('NGUYỄN VĂN AN', 'vi', 1), en_1: count('NGUYỄN VĂN AN', 'en', 1) },
  { truy_van: 'dang ngoc ha', khong_collation: count('dang ngoc ha'), vi_1: count('dang ngoc ha', 'vi', 1), en_1: count('dang ngoc ha', 'en', 1) },
], ['truy_van', 'khong_collation', 'vi_1', 'en_1']);
// Tiếng Việt: ă â ê ô ơ ư đ là CHỮ CÁI riêng (khác biệt cấp 1); chỉ dấu thanh và hoa/thường bị bỏ ở strength 1.
expectValue('vi strength 1: "nguyen van an" KHÔNG khớp (văn ≠ van)', count('nguyen van an', 'vi', 1), 0);
expectValue('vi strength 1: bỏ dấu thanh và hoa/thường', count('nguyên văn an', 'vi', 1), 1);
expectValue('en strength 1: bỏ mọi dấu ⇒ tìm không dấu', count('nguyen van an', 'en', 1), 1);
expectValue('regex /^Nguy/ (phân biệt dấu)', db.customers.countDocuments({ full_name: /^Nguy/ }), 1);

section('Đếm: countDocuments vs estimatedDocumentCount');
table([{ countDocuments: db.trips.countDocuments({}), estimatedDocumentCount: db.trips.estimatedDocumentCount() }]);
