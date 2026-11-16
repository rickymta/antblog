// P01 · thu-nghiem/mg004-kieu-bson.js — kiểu BSON cho tiền, thời gian, chuỗi, định danh (MG-004).
// Chạy: npm run p01:thu-nghiem  (00-reset rồi file này; dùng collection nháp `lab_types`).

const c = db.lab_types;
c.drop();

section('1. Số trong mongosh: literal nguyên → int, ngoài 32 bit → double (KHÔNG phải long)');
c.insertOne({ _id: 1, nho: 100000, lon: 2 ** 31, le: 45000.5, long: Long('3000000000'), int_ep: Int32(7), dec: Decimal128('0.1') });
table(c.aggregate([{ $project: { _id: 0, nho: { $type: '$nho' }, lon: { $type: '$lon' }, le: { $type: '$le' }, long: { $type: '$long' }, int_ep: { $type: '$int_ep' }, dec: { $type: '$dec' } } }]));
expectValue('2**31 lưu thành', c.aggregate([{ $project: { t: { $type: '$lon' } } }]).next().t, 'double');

section('2. Cộng tiền: double sai số, int/long/Decimal128 chính xác');
c.insertMany([
  { _id: 'd1', kind: 'double', v: 0.1 }, { _id: 'd2', kind: 'double', v: 0.2 },
  { _id: 'm1', kind: 'decimal', v: Decimal128('0.1') }, { _id: 'm2', kind: 'decimal', v: Decimal128('0.2') },
]);
table(c.aggregate([{ $match: { kind: { $exists: true } } }, { $group: { _id: '$kind', tong: { $sum: '$v' } } }, { $sort: { _id: 1 } },
  { $project: { _id: 0, kieu: '$_id', tong: { $toString: '$tong' }, bang_0_3: { $eq: ['$tong', { $cond: [{ $eq: ['$_id', 'decimal'] }, Decimal128('0.3'), 0.3] }] } } }]));
expectValue('double: 0.1 + 0.2 == 0.3', c.aggregate([{ $match: { kind: 'double' } }, { $group: { _id: null, s: { $sum: '$v' } } }]).next().s === 0.3, false);

section('3. $sum của int tự nâng lên long khi tràn 32 bit');
c.insertMany([{ _id: 'big1', g: 2000000000 }, { _id: 'big2', g: 2000000000 }]);
const big = c.aggregate([{ $match: { g: { $exists: true } } }, { $group: { _id: null, s: { $sum: '$g' } } }, { $project: { s: 1, t: { $type: '$s' } } }]).next();
table([{ tong: big.s, kieu: big.t }]);
expectValue('kiểu của tổng hai int 2 tỷ', big.t, 'long');

section('4. Date: luôn là thời điểm UTC; múi giờ chỉ là cách hiển thị');
// Container chạy TZ=Asia/Ho_Chi_Minh. Chuỗi không có múi giờ là BẪY:
const a = new Date('2026-11-02T07:58:00');      // không múi giờ ⇒ hiểu theo giờ của tiến trình mongosh
const b = new Date('2026-11-02T07:58:00+07:00'); // rõ ràng
const z = ISODate('2026-11-02T07:58:00');       // ISODate không múi giờ ⇒ UTC
table([{ cach_viet: "new Date('…07:58:00')", utc: a.toISOString() }, { cach_viet: "new Date('…07:58:00+07:00')", utc: b.toISOString() }, { cach_viet: "ISODate('…07:58:00')", utc: z.toISOString() }], ['cach_viet', 'utc']);
expectValue('ISODate không múi giờ lệch 7 giờ so với giờ VN', (z - b) / 3600000, 7);
c.insertOne({ _id: 'time', at: b });
table(c.aggregate([{ $match: { _id: 'time' } }, { $project: { _id: 0,
  utc: { $dateToString: { date: '$at', format: '%Y-%m-%d %H:%M' } },
  gio_vn: { $dateToString: { date: '$at', format: '%Y-%m-%d %H:%M', timezone: 'Asia/Ho_Chi_Minh' } },
  ngay_vn: { $dateTrunc: { date: '$at', unit: 'day', timezone: 'Asia/Ho_Chi_Minh' } },
} }]), ['utc', 'gio_vn', 'ngay_vn']);
expectValue('Date độ chính xác mili giây', new Date('2026-11-02T07:58:00.123456+07:00').getUTCMilliseconds(), 123);

section('5. Chuỗi: UTF-8; độ dài theo ký tự khác theo byte');
c.insertOne({ _id: 'str', s: 'Đà Nẵng', s2: 'Đà Nẵng'.normalize('NFD') });
table(c.aggregate([{ $match: { _id: 'str' } }, { $project: { _id: 0,
  cp: { $strLenCP: '$s' }, bytes: { $strLenBytes: '$s' }, nfd_cp: { $strLenCP: '$s2' }, bang_nhau: { $eq: ['$s', '$s2'] } } }]));
expectValue('NFC và NFD của "Đà Nẵng" bằng nhau khi so byte', c.findOne({ _id: 'str' }).s === c.findOne({ _id: 'str' }).s2, false);

section('6. Định danh: ObjectId có thời điểm tạo bên trong; UUID phải là binary subtype 4');
const oid = ObjectId('6ae93b700000000000000001');
table([{ objectid: oid.toHexString(), thoi_diem: oid.getTimestamp(), bytes: 12 }]);
c.insertOne({ _id: 'ids', u: UUID('c0000000-0000-4000-8000-000000000001'), s: 'c0000000-0000-4000-8000-000000000001' });
table(c.aggregate([{ $match: { _id: 'ids' } }, { $project: { _id: 0, u: { $type: '$u' }, u_bytes: { $binarySize: '$u' }, s: { $type: '$s' }, s_bytes: { $strLenBytes: '$s' } } }]));
expectValue('UUID lưu 16 byte, chuỗi UUID 36 byte', `${c.aggregate([{ $match: { _id: 'ids' } }, { $project: { n: { $binarySize: '$u' } } }]).next().n}/36`, '16/36');
expectValue('so UUID với chuỗi', c.countDocuments({ u: 'c0000000-0000-4000-8000-000000000001' }), 0);

section('7. So sánh khác kiểu: thứ tự kiểu BSON, không phải lỗi');
c.insertMany([{ _id: 'f1', fare: 45000 }, { _id: 'f2', fare: '45000' }, { _id: 'f3', fare: Long('45000') }, { _id: 'f4', fare: 45000.0 }]);
expectValue('{fare: 45000} khớp int, long, double', c.countDocuments({ fare: 45000 }), 3);
expectValue('{fare: {$gt: 0}} bỏ qua chuỗi', c.countDocuments({ fare: { $gt: 0 } }), 3);
expectValue('{fare: {$type: "string"}}', c.countDocuments({ fare: { $type: 'string' } }), 1);

c.drop();
