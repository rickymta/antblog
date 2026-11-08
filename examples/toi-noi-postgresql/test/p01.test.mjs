// Kiểm tra lab P01 bằng node:test — chạy: npm test
//
// Mỗi ca dùng một phiên PGlite MỚI (database trống trong bộ nhớ).

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createDb, expandPaths, parseExpectations, runFiles, splitSql } from '../run.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const P01 = join(ROOT, 'p01');
const file = (name) => join(P01, name);
const quiet = { write: () => {}, baseDir: ROOT };

async function runP01(extra = []) {
  const db = await createDb();
  try {
    const summary = await runFiles(db, [...expandPaths([P01]), ...extra], quiet);
    return { db, summary };
  } catch (e) {
    await db.close();
    throw e;
  }
}

/** Đếm chỉ thị expect-* trong một file SQL (để đối chiếu runner không bỏ sót). */
function countDirectives(name, kind) {
  const re = new RegExp(`^\\s*--\\s*expect-${kind}\\s*:`, 'gm');
  return (readFileSync(file(name), 'utf8').match(re) || []).length;
}

test('thứ tự file khi chạy cả thư mục p01: 00 → 07, bỏ *-rollback.sql', () => {
  const names = expandPaths([P01]).map((f) => f.split(/[\\/]/).pop());
  assert.deepEqual(names, [
    '00-reset.sql',
    '01-schema.sql',
    '02-fixture-valid.sql',
    '03-fixture-invalid.sql',
    '04-lifecycle.sql',
    '05-select.sql',
    '06-migration.sql',
    '07-checks.sql',
  ]);
});

test('p01 chạy 00 → 07 trên PGlite mới: không lỗi ngoài kỳ vọng, mọi kỳ vọng đạt', async () => {
  const { db, summary } = await runP01();
  try {
    assert.deepEqual(summary.failures, [], 'không được có FAIL');

    // Mọi chỉ thị expect-error trong các file đều đã được đánh giá và khớp.
    const files = expandPaths([P01]).map((f) => f.split(/[\\/]/).pop());
    const expectedErrors = files.reduce((n, f) => n + countDirectives(f, 'error'), 0);
    const expectedRows = files.reduce((n, f) => n + countDirectives(f, 'rows'), 0);
    assert.equal(summary.expectedErrors, expectedErrors);
    assert.equal(summary.rowChecks, expectedRows);
    assert.ok(expectedErrors >= 30, `03 phải có đủ ca dữ liệu sai (đang có ${expectedErrors})`);

    // 03: mỗi câu lỗi đều lỗi đúng như mong đợi — không câu nào "lọt".
    const invalid = summary.results.filter((r) => r.file.endsWith('03-fixture-invalid.sql') && r.expect.error);
    assert.equal(invalid.length, countDirectives('03-fixture-invalid.sql', 'error'));
    for (const r of invalid) assert.ok(r.error && r.ok, `${r.file}:${r.line}`);
    // Đủ năm nhóm lỗi chính: NOT NULL, FK, UNIQUE, CHECK, sai định dạng.
    const codes = new Set(invalid.map((r) => r.error.code));
    for (const c of ['23502', '23503', '23505', '23514', '22P02', '22007', '22008', '22003', '428C9']) {
      assert.ok(codes.has(c), `thiếu ca lỗi ${c}`);
    }

    // 07: mọi truy vấn kiểm tra bất biến trả 0 dòng.
    const checks = summary.results.filter((r) => r.file.endsWith('07-checks.sql') && r.expect.rows === 0);
    assert.ok(checks.length >= 12, `07 phải có ít nhất 12 truy vấn kiểm tra (đang có ${checks.length})`);
    for (const r of checks) assert.equal(r.rows.length, 0, `${r.file}:${r.line} phải trả 0 dòng`);

    // Số dòng cuối cùng.
    const counts = (await db.query(`
      SELECT (SELECT count(*) FROM toinoi.cities)::int      AS cities,
             (SELECT count(*) FROM toinoi.customers)::int   AS customers,
             (SELECT count(*) FROM toinoi.drivers)::int     AS drivers,
             (SELECT count(*) FROM toinoi.merchants)::int   AS merchants,
             (SELECT count(*) FROM toinoi.trips)::int       AS trips,
             (SELECT count(*) FROM toinoi.trip_events)::int AS trip_events`)).rows[0];
    assert.deepEqual(counts, { cities: 3, customers: 6, drivers: 7, merchants: 3, trips: 16, trip_events: 7 });

    // Fixture phủ đủ 8 trạng thái và 3 loại dịch vụ.
    const statuses = (await db.query('SELECT DISTINCT status FROM toinoi.trips')).rows.length;
    const kinds = (await db.query('SELECT DISTINCT kind FROM toinoi.trips')).rows.length;
    assert.equal(statuses, 8);
    assert.equal(kinds, 3);

    // Chuyến của 04 hoàn thành với 7 dòng lịch sử; bản nháp đã bị xoá.
    const life = (await db.query(`
      SELECT t.code::int AS code, t.status, count(e.*)::int AS events
      FROM toinoi.trips t LEFT JOIN toinoi.trip_events e ON e.trip_id = t.id
      WHERE t.id IN ('b0000000-0000-4000-8000-000000000001', 'b0000000-0000-4000-8000-000000000002')
      GROUP BY t.code, t.status`)).rows;
    assert.deepEqual(life, [{ code: 100029, status: 'completed', events: 7 }]);

    // Sau 06: biển số bắt buộc, CHECK định dạng đã VALIDATE.
    const col = (await db.query(`SELECT is_nullable FROM information_schema.columns
      WHERE table_schema = 'toinoi' AND table_name = 'drivers' AND column_name = 'vehicle_plate'`)).rows;
    assert.deepEqual(col, [{ is_nullable: 'NO' }]);
  } finally {
    await db.close();
  }
});

test('06-migration-rollback.sql gỡ sạch migration; chạy lại 06 và 07 vẫn đạt', async () => {
  const db = await createDb();
  try {
    const upTo06 = expandPaths([P01]).filter((f) => !f.endsWith('07-checks.sql'));
    const files = [...upTo06, file('06-migration-rollback.sql'), file('06-migration-rollback.sql'),
      file('06-migration.sql'), file('07-checks.sql')];
    const summary = await runFiles(db, files, quiet);
    assert.deepEqual(summary.failures, []);
    // Bản sao lưu biển số có đủ 7 tài xế (chạy rollback hai lần không nhân đôi).
    const backup = (await db.query('SELECT count(*)::int AS n FROM toinoi.drivers_vehicle_plate_backup')).rows[0].n;
    assert.equal(backup, 7);
  } finally {
    await db.close();
  }
});

test('runner tự kiểm: bắt được lỗi mong đợi bị thiếu, sai mã, sai tên, sai số dòng, lỗi ngoài kỳ vọng', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'toinoi-runner-'));
  try {
    const bad = join(dir, '99-bad.sql');
    writeFileSync(bad, [
      'SET search_path TO toinoi;',
      '-- (1) mong đợi lỗi trùng số điện thoại nhưng số này chưa ai dùng ⇒ câu lệnh chạy được ⇒ FAIL',
      '-- expect-error: 23505 uq_customers_phone',
      "INSERT INTO customers (id, phone, full_name, created_at) VALUES ('f0000000-0000-4000-8000-000000000099', '0900000099', 'Khách Mới', '2026-11-03 09:00:00+07');",
      '-- (2) đúng là lỗi trùng, nhưng ghi sai mã SQLSTATE ⇒ FAIL',
      '-- expect-error: 23514 uq_customers_phone',
      "INSERT INTO customers (id, phone, full_name, created_at) VALUES ('f0000000-0000-4000-8000-000000000098', '0900000001', 'Trùng', '2026-11-03 09:00:00+07');",
      '-- (3) đúng mã, sai tên ràng buộc ⇒ FAIL',
      '-- expect-error: 23505 pk_customers',
      "INSERT INTO customers (id, phone, full_name, created_at) VALUES ('f0000000-0000-4000-8000-000000000097', '0900000001', 'Trùng', '2026-11-03 09:00:00+07');",
      '-- (4) sai số dòng ⇒ FAIL',
      '-- expect-rows: 99',
      'SELECT code FROM trips;',
      '-- (5) lỗi không có chỉ thị ⇒ FAIL',
      'SELECT 1 / 0;',
      '-- (6) khớp ⇒ PASS, runner vẫn chạy tiếp sau các lỗi',
      '-- expect-error: 23505 uq_customers_phone',
      "INSERT INTO customers (id, phone, full_name, created_at) VALUES ('f0000000-0000-4000-8000-000000000096', '0900000001', 'Trùng', '2026-11-03 09:00:00+07');",
      '',
    ].join('\n'));
    const db = await createDb();
    try {
      await runFiles(db, [file('00-reset.sql'), file('01-schema.sql'), file('02-fixture-valid.sql')], quiet);
      const summary = await runFiles(db, [bad], quiet);
      const reasons = summary.failures.map((f) => f.reason);
      assert.equal(summary.failures.length, 5, reasons.join('\n'));
      assert.match(reasons[0], /nhưng câu lệnh chạy thành công/);
      assert.match(reasons[1], /mong đợi lỗi 23514 .*nhận 23505/);
      assert.match(reasons[2], /mong đợi lỗi 23505 pk_customers, nhận 23505 uq_customers_phone/);
      assert.match(reasons[3], /mong đợi 99 dòng, nhận 15/);
      assert.match(reasons[4], /lỗi ngoài kỳ vọng 22012/);
      assert.equal(summary.results.at(-1).ok, true, 'ca (6) phải PASS');
    } finally {
      await db.close();
    }

    // Qua dòng lệnh: có FAIL ⇒ exit code 1.
    const r = (() => {
      try {
        execFileSync(process.execPath, [join(ROOT, 'run.mjs'), file('00-reset.sql'), file('01-schema.sql'),
          file('02-fixture-valid.sql'), bad, '--no-echo'], { cwd: ROOT, stdio: 'pipe' });
        return 0;
      } catch (e) {
        return e.status;
      }
    })();
    assert.equal(r, 1);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('splitSql: chuỗi, chú thích, dollar quote, định danh có dấu chấm phẩy', () => {
  const sql = [
    "SELECT 'a;b', \"x;y\" FROM t; -- chú thích; có chấm phẩy",
    '/* khối /* lồng */ ; */ SELECT 2;',
    "DO $$ BEGIN RAISE NOTICE 'x;y'; END $$;",
    "SELECT $tag$ ; $$ ; $tag$;",
    "SELECT E'it\\'s;';",
    '-- chỉ có chú thích ở cuối',
  ].join('\n');
  const parts = splitSql(sql).map((s) => s.text);
  assert.equal(parts.length, 5);
  assert.match(parts[2], /^DO \$\$/);
  assert.throws(() => splitSql('SELECT 1;\n\\d trips\n'), /lệnh meta của psql/);
  assert.throws(() => splitSql('SELECT 1;\n-- expect-rows: 1\n'), /không đứng trước câu lệnh nào/);
  assert.deepEqual(parseExpectations('-- mô tả\n-- expect-error: 23502 full_name\nINSERT …'), {
    error: { code: '23502', name: 'full_name' },
  });
  assert.deepEqual(parseExpectations('-- expect-rows: 0\nSELECT 1'), { rows: 0 });
});

test('p01/OUTPUT.txt khớp kết quả chạy hiện tại (chạy `npm run p01:output` sau khi sửa SQL)', () => {
  const out = execFileSync(process.execPath, [join(ROOT, 'run.mjs'), 'p01'], { cwd: ROOT, encoding: 'utf8' });
  const saved = readFileSync(join(P01, 'OUTPUT.txt'), 'utf8').replace(/\r\n/g, '\n');
  assert.equal(out, saved);
});
