// Kiểm tra lab P02 bằng node:test — chạy: npm test
//
// P02 chạy trên lược đồ P01: p01/00-reset.sql + p01/01-schema.sql rồi cả thư
// mục p02 (đúng như `npm run p02`). Mỗi ca dùng một phiên PGlite MỚI.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { canonicalResult, createDb, expandPaths, parseExpectations, resultMd5, runFiles } from '../run.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const P02 = join(ROOT, 'p02');
const BASE = [join(ROOT, 'p01', '00-reset.sql'), join(ROOT, 'p01', '01-schema.sql')];
const quiet = { write: () => {}, baseDir: ROOT };
const files = () => [...BASE, ...expandPaths([P02])];

/** Đếm chỉ thị trong các file p02; expect-line đếm theo CÂU LỆNH (nhóm dòng liền nhau). */
function countDirectives(kind) {
  let n = 0;
  for (const f of expandPaths([P02])) {
    const text = readFileSync(f, 'utf8');
    if (kind === 'line') {
      // Mỗi khối các dòng `-- expect-line:` liền nhau (có thể xen chú thích) là một lần kiểm.
      const lines = text.split('\n');
      let inBlock = false;
      for (const l of lines) {
        const isLine = /^\s*--\s*expect-line\s*:/.test(l);
        if (isLine && !inBlock) n++;
        if (isLine) inBlock = true;
        else if (!/^\s*--/.test(l)) inBlock = false;
      }
    } else {
      n += (text.match(new RegExp(`^\\s*--\\s*expect-${kind}\\s*:`, 'gm')) || []).length;
    }
  }
  return n;
}

test('thứ tự file khi chạy cả thư mục p02', () => {
  const names = expandPaths([P02]).map((f) => f.split(/[\\/]/).pop());
  assert.deepEqual(names, [
    '00-mo-rong-schema.sql',
    '01-fixture.sql',
    '02-kiem-tra-fixture.sql',
    '11-null-case-coalesce.sql',
    '12-aggregate-group-by.sql',
    '13-join-cardinality.sql',
    '14-subquery-exists-cte.sql',
    '15-window-function.sql',
    '16-date-time-timezone.sql',
    '17-upsert-on-conflict.sql',
    '18-pagination-keyset.sql',
    '19-cay-khu-vuc.sql',
    '19-chu-trinh.sql',
    '20-doi-soat-thanh-toan.sql',
  ]);
});

test('p02 chạy trọn trên PGlite mới: 0 FAIL, mọi chỉ thị đều được chấm, đáp án đúng', async () => {
  const db = await createDb();
  try {
    const summary = await runFiles(db, files(), quiet);
    assert.deepEqual(summary.failures, [], 'không được có FAIL');

    // Không chỉ thị nào bị bỏ sót.
    assert.equal(summary.expectedErrors, countDirectives('error'));
    assert.equal(summary.rowChecks, countDirectives('rows'));
    assert.equal(summary.valueChecks, countDirectives('value'));
    assert.equal(summary.lineChecks, countDirectives('line'));
    assert.equal(summary.md5Checks, countDirectives('md5'));
    assert.ok(summary.lineChecks >= 100, `phải có nhiều kiểm nội dung (đang có ${summary.lineChecks})`);

    // Mọi câu trả bảng kết quả trong p02 đều có ít nhất một kỳ vọng.
    const unguarded = summary.results.filter(
      (r) => r.file.startsWith('p02/') && r.fields && Object.keys(r.expect).length === 0,
    );
    assert.deepEqual(unguarded.map((r) => `${r.file}:${r.line}`), []);

    // Mỗi bài có đủ câu lệnh có kỳ vọng.
    for (const n of ['11', '12', '13', '14', '15', '16', '17', '18', '19', '20']) {
      const guarded = summary.results.filter((r) => r.file.startsWith(`p02/${n}-`) && Object.keys(r.expect).length > 0);
      assert.ok(guarded.length >= 5, `bài PG-0${n} cần ≥ 5 câu có kỳ vọng (đang có ${guarded.length})`);
    }

    // ----- Đáp án then chốt, tính độc lập bằng JS/SQL riêng -----
    const one = async (sql) => (await db.query(sql)).rows[0];

    assert.deepEqual(await one(`
      SELECT (SELECT count(*) FROM toinoi.trips)::int        AS trips,
             (SELECT count(*) FROM toinoi.trip_events)::int  AS events,
             (SELECT count(*) FROM toinoi.trip_ratings)::int AS ratings,
             (SELECT count(*) FROM toinoi.payments)::int     AS payments,
             (SELECT count(*) FROM toinoi.areas)::int        AS areas,
             (SELECT count(*) FROM toinoi.menu_items)::int   AS menu_items`),
    { trips: 3174, events: 20685, ratings: 1891, payments: 2826, areas: 34, menu_items: 7 });

    // Doanh thu tháng 10 và tỉ lệ huỷ toàn hệ thống.
    assert.deepEqual(await one(`
      SELECT sum(fare) FILTER (WHERE status = 'completed')::int AS doanh_thu,
             count(*) FILTER (WHERE status = 'cancelled')::int  AS huy,
             count(*) FILTER (WHERE status IN ('completed', 'cancelled'))::int AS da_ket_thuc
      FROM toinoi.trips`), { doanh_thu: 182287000, huy: 461, da_ket_thuc: 3168 });

    // PG-011: avg bỏ qua NULL; COALESCE(stars, 0) kéo điểm xuống.
    assert.deepEqual(await one(`
      SELECT round(avg(stars), 2)::text AS dung, round(avg(coalesce(stars, 0)), 2)::text AS sai
      FROM toinoi.trip_ratings`), { dung: '4.22', sai: '3.88' });

    // PG-014: bẫy NOT IN + NULL.
    assert.equal((await db.query(`SELECT d.code FROM toinoi.drivers d
      WHERE d.id NOT IN (SELECT driver_id FROM toinoi.trips)`)).rows.length, 0);
    assert.deepEqual((await db.query(`SELECT d.code FROM toinoi.drivers d
      WHERE NOT EXISTS (SELECT 1 FROM toinoi.trips t WHERE t.driver_id = d.id) ORDER BY d.code`)).rows.map((r) => r.code),
    [8219, 8220]);

    // PG-016: ngày 15/10 giờ VN = 106 chuyến; ngày bão 20/10 Đà Nẵng 0 chuyến.
    assert.equal((await one(`SELECT count(*)::int AS n FROM toinoi.trips
      WHERE requested_at >= '2026-10-15 00:00+07' AND requested_at < '2026-10-16 00:00+07'`)).n, 106);
    assert.equal((await one(`SELECT count(*)::int AS n FROM toinoi.trips
      WHERE city_id = 3 AND (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = '2026-10-20'`)).n, 0);

    // PG-018: hai nhóm trùng giờ đặt của khách doanh nghiệp (6 và 4 chuyến).
    assert.deepEqual((await db.query(`SELECT count(*)::int AS n FROM toinoi.trips
      WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
      GROUP BY requested_at HAVING count(*) > 1 ORDER BY 1 DESC`)).rows.map((r) => r.n), [6, 4]);

    // PG-019: sau 19-chu-trinh (ROLLBACK) cây lành lại.
    assert.deepEqual(await one(`SELECT count(*)::int AS n FROM toinoi.areas WHERE id IN (21, 22) AND parent_id = 2`), { n: 2 });

    // PG-020: đúng 7 chuyến lệch, viết lại quy tắc theo cách khác (từng chuyến
    // tính trong JS) để đối chiếu với đáp án SQL.
    const rows = (await db.query(`
      SELECT t.code::int AS code, t.status, t.fare,
             coalesce(json_agg(json_build_object('k', p.kind, 's', p.status, 'a', p.amount))
                      FILTER (WHERE p.id IS NOT NULL), '[]') AS gd
      FROM toinoi.trips t LEFT JOIN toinoi.payments p ON p.trip_id = t.id
      WHERE t.status IN ('completed', 'cancelled')
      GROUP BY t.code, t.status, t.fare`)).rows;
    const lech = [];
    let tongChenh = 0;
    for (const r of rows) {
      const ok = r.gd.filter((g) => g.s === 'succeeded');
      const net = ok.reduce((s, g) => s + (g.k === 'refund' ? -g.a : g.a), 0);
      const phaiThu = r.status === 'completed' ? r.fare : 0;
      if (net !== phaiThu) { lech.push(r.code); tongChenh += net - phaiThu; }
    }
    lech.sort((a, b) => a - b);
    assert.deepEqual(lech, [100724, 101084, 101310, 101987, 102443, 102642, 102739]);
    assert.equal(tongChenh, 268000);

    // Đáp án SQL của 20 (câu liệt kê chênh lệch) khớp danh sách trên.
    const answer = summary.results.find((r) => r.file === 'p02/20-doi-soat-thanh-toan.sql'
      && /WHERE s\.loai_lech IS NOT NULL/.test(r.sql));
    assert.ok(answer, 'không tìm thấy câu đáp án của PG-020');
    assert.deepEqual(answer.rows.map((r) => Number(r[0])), lech);
    assert.deepEqual(answer.rows.map((r) => r[8]), [
      '3 THIẾU THANH TOÁN', '3 THIẾU THANH TOÁN', '4 THU TRÙNG', '1 HOÀN KHÔNG CÓ THU',
      '5 LỆCH SỐ TIỀN', '5 LỆCH SỐ TIỀN', '2 THU CHUYẾN HUỶ',
    ]);
  } finally {
    await db.close();
  }
});

test('fixture P02 tất định: hai lần sinh cho cùng dấu vân tay dữ liệu', async () => {
  const fingerprint = async () => {
    const db = await createDb();
    try {
      const s = await runFiles(db, [...BASE, join(P02, '00-mo-rong-schema.sql'), join(P02, '01-fixture.sql')], quiet);
      assert.deepEqual(s.failures, []);
      const q = async (sql) => resultMd5((await db.query(sql, [], { rowMode: 'array' })).rows.map((r) => r.map((v) => (v === null ? null : String(v)))));
      return [
        await q(`SELECT code, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                        extract(epoch FROM requested_at), extract(epoch FROM accepted_at),
                        extract(epoch FROM completed_at), extract(epoch FROM updated_at), pickup_area_id
                 FROM toinoi.trips ORDER BY code`),
        await q(`SELECT id, trip_id, status, extract(epoch FROM occurred_at) FROM toinoi.trip_events ORDER BY id`),
        await q(`SELECT id, trip_id, kind, method, status, amount, provider_ref, extract(epoch FROM paid_at)
                 FROM toinoi.payments ORDER BY id`),
        await q(`SELECT trip_id, stars, tip, extract(epoch FROM rated_at) FROM toinoi.trip_ratings ORDER BY trip_id`),
      ];
    } finally {
      await db.close();
    }
  };
  assert.deepEqual(await fingerprint(), await fingerprint());
});

test('runner: expect-value / expect-line / expect-md5 bắt đúng sai lệch', async () => {
  // Hàm dạng chuẩn dùng cho md5.
  assert.equal(canonicalResult([['1', null], [' a ', 'b']]), '1 | NULL\na | b');
  assert.deepEqual(parseExpectations('-- expect-value: 42\n-- expect-line: a |b\n-- expect-line:  c | NULL \nSELECT 1'), {
    value: '42', lines: ['a | b', 'c | NULL'],
  });

  const dir = mkdtempSync(join(tmpdir(), 'toinoi-p02-runner-'));
  try {
    const bad = join(dir, '99-bad.sql');
    const md5Ok = resultMd5([['1', 'x'], ['2', null]]);
    writeFileSync(bad, [
      '-- (1) đúng giá trị ⇒ PASS',
      '-- expect-value: 42',
      'SELECT 42;',
      '-- (2) sai giá trị ⇒ FAIL',
      '-- expect-value: 41',
      'SELECT 42;',
      '-- (3) expect-value trên kết quả 2 dòng ⇒ FAIL',
      '-- expect-value: 1',
      'SELECT * FROM (VALUES (1), (2)) AS v (n);',
      '-- (4) đúng nội dung, NULL viết là NULL ⇒ PASS',
      '-- expect-line: 1 | x',
      '-- expect-line: 2 | NULL',
      "SELECT * FROM (VALUES (1, 'x'), (2, NULL)) AS v (n, s) ORDER BY n;",
      '-- (5) sai một ô ⇒ FAIL',
      '-- expect-line: 1 | x',
      '-- expect-line: 2 | y',
      "SELECT * FROM (VALUES (1, 'x'), (2, NULL)) AS v (n, s) ORDER BY n;",
      '-- (6) thiếu dòng kỳ vọng ⇒ FAIL',
      '-- expect-line: 1 | x',
      "SELECT * FROM (VALUES (1, 'x'), (2, NULL)) AS v (n, s) ORDER BY n;",
      '-- (7) đúng md5 ⇒ PASS',
      `-- expect-md5: ${md5Ok}`,
      "SELECT * FROM (VALUES (1, 'x'), (2, NULL)) AS v (n, s) ORDER BY n;",
      '-- (8) sai md5 ⇒ FAIL',
      '-- expect-md5: 00000000000000000000000000000000',
      "SELECT * FROM (VALUES (1, 'x'), (2, NULL)) AS v (n, s) ORDER BY n;",
      '-- (9) CREATE TABLE AS in thẻ SELECT n như psql, và expect-rows đọc được thẻ đó ⇒ PASS',
      '-- expect-rows: 2',
      'CREATE TEMP TABLE t AS SELECT * FROM (VALUES (1), (2)) AS v (n);',
      '',
    ].join('\n'));
    const db = await createDb();
    let out = '';
    try {
      const s = await runFiles(db, [bad], { write: (x) => { out += x; }, baseDir: dir });
      const reasons = s.failures.map((f) => f.reason);
      assert.equal(s.failures.length, 5, reasons.join('\n'));
      assert.match(reasons[0], /mong đợi giá trị «41», nhận «42»/);
      assert.match(reasons[1], /expect-value cần đúng 1 dòng × 1 cột, nhận 2 dòng × 1 cột/);
      assert.match(reasons[2], /khác ở dòng 2: mong đợi «2 \| y», nhận «2 \| NULL»/);
      assert.match(reasons[3], /mong đợi 1 dòng, nhận 2/);
      assert.match(reasons[4], /mong đợi md5 0{32}, nhận [0-9a-f]{32}/);
      assert.equal(s.valueChecks, 3);
      assert.equal(s.lineChecks, 3);
      assert.equal(s.md5Checks, 2);
      assert.match(out, /^SELECT 2$/m, 'CREATE TABLE AS phải in thẻ SELECT 2');
    } finally {
      await db.close();
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('p02/OUTPUT.txt khớp kết quả chạy hiện tại (chạy `npm run p02:output` sau khi sửa SQL)', () => {
  const out = execFileSync(process.execPath, [join(ROOT, 'run.mjs'), 'p01/00-reset.sql', 'p01/01-schema.sql', 'p02'],
    { cwd: ROOT, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
  const saved = readFileSync(join(P02, 'OUTPUT.txt'), 'utf8').replace(/\r\n/g, '\n');
  assert.equal(out, saved);
});

test('p02/00 báo lỗi rõ ràng nếu chạy khi chưa có lược đồ P01', async () => {
  const db = await createDb();
  try {
    await db.exec('DROP SCHEMA IF EXISTS toinoi CASCADE; CREATE SCHEMA toinoi;');
    const s = await runFiles(db, [join(P02, '00-mo-rong-schema.sql')], quiet);
    const first = s.results.find((r) => r.error);
    assert.ok(first, 'phải có lỗi');
    assert.equal(first.error.code, 'P0001');
    assert.match(first.error.message, /Chưa có lược đồ P01/);
  } finally {
    await db.close();
  }
});
