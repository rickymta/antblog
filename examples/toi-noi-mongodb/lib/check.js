// lib/check.js — nạp trước mọi file của lab (run.mjs truyền nó cho mongosh).
// Không thuộc Tới Nơi: chỉ là công cụ in kết quả và kiểm PASS/FAIL.

// In mảng document thành bảng "cột|cột" (giống output của lab SQL Server).
// Giá trị: Date → giờ VN "YYYY-MM-DD HH:mm:ss +07:00", UUID → chuỗi, null/thiếu → NULL / (thiếu).
globalThis.fmt = function fmt(v) {
  if (v === undefined) return '(thiếu)';
  if (v === null) return 'NULL';
  if (v instanceof Date) {
    const t = new Date(v.getTime() + 7 * 3600 * 1000).toISOString();
    return t.slice(0, 10) + ' ' + t.slice(11, 19) + ' +07:00';
  }
  if (v && v._bsontype === 'Binary' && v.sub_type === 4) return v.toUUID().toString();
  if (v && v._bsontype === 'ObjectId') return v.toHexString();
  if (v && (v._bsontype === 'Decimal128' || v._bsontype === 'Long' || v._bsontype === 'Int32' || v._bsontype === 'Double')) return v.toString();
  if (Array.isArray(v) || (typeof v === 'object')) return EJSON.stringify(v, { relaxed: true });
  return String(v);
};

globalThis.table = function table(rows, cols) {
  rows = Array.isArray(rows) ? rows : rows.toArray();
  cols = cols ?? [...new Set(rows.flatMap((r) => Object.keys(r)))];
  print(cols.join('|'));
  print(cols.map((c) => '-'.repeat(c.length)).join('|'));
  for (const r of rows) print(cols.map((c) => fmt(c.split('.').reduce((o, k) => (o == null ? undefined : o[k]), r))).join('|'));
  print(`(${rows.length} document)`);
};

globalThis.section = (title) => print(`\n/* ${title} */`);

globalThis.expectValue = function expectValue(label, actual, expected) {
  const a = fmt(actual);
  const e = String(expected);
  print(a === e ? `PASS ${label} = ${a}` : `FAIL ${label}: nhận ${a}, mong đợi ${e}`);
};

// Chạy fn; PASS khi fn ném lỗi có `code` mong đợi (và chứa `hint` trong thông điệp/chi tiết nếu có).
// Mã lỗi hay gặp: 121 DocumentValidationFailure, 11000 DuplicateKey.
globalThis.expectError = function expectError(label, fn, code, hint) {
  try {
    fn();
  } catch (e) {
    const info = e.errInfo ?? e.writeErrors?.[0]?.errInfo ?? {};
    const detail = EJSON.stringify(info, { relaxed: true });
    let ok = e.code === code;
    let shown = hint;
    if (typeof hint === 'number') {
      // Số = chỉ số điều kiện của $and gốc trong validator phải là điều kiện DUY NHẤT không thoả.
      const failed = (info.details?.clausesNotSatisfied ?? []).map((c) => c.index);
      ok = ok && failed.length === 1 && failed[0] === hint;
      shown = `điều kiện #${failed.join(',#')}`;
    } else if (hint) {
      ok = ok && `${e.message} ${detail}`.includes(hint);
    }
    print(ok ? `PASS ${label} → lỗi ${e.code}${shown ? ` (${shown})` : ''}` : `FAIL ${label}: lỗi ${e.code} ${shown ?? ''} ${e.message}`);
    return e;
  }
  print(`FAIL ${label}: không có lỗi, mong đợi ${code}`);
  return null;
};

// Lấy chuỗi ngắn mô tả quy tắc validation bị vi phạm từ errInfo.details (MongoDB ≥ 5.0).
globalThis.ruleFailed = function ruleFailed(e) {
  const d = e?.errInfo?.details ?? e?.writeErrors?.[0]?.errInfo?.details;
  return d ? EJSON.stringify(d.schemaRulesNotSatisfied ?? d, { relaxed: true }) : '(không có chi tiết)';
};

// Ngày giờ VN → Date (UTC bên trong). vn('2026-11-02 07:58') hoặc vn('2026-11-02 07:58:30').
globalThis.vn = (s) => new Date(s.replace(' ', 'T') + (s.length === 16 ? ':00' : '') + '+07:00');
