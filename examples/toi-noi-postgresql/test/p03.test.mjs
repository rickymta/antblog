// P03 · Lưu trữ, index và kế hoạch thực thi — chạy trên PostgreSQL 18 THẬT qua pg/run-pg.mjs.
//
// Phần 1 (luôn chạy, không cần PostgreSQL): kiểm các mảnh của runner (tách câu kiểu psql,
// đọc chỉ thị, đọc bảng aligned, chuẩn hoá) và mã thoát khi KHÔNG có PostgreSQL.
// Phần 2 (chỉ khi tìm thấy PostgreSQL ≥ 18; không có ⇒ SKIP, không FAIL): một lần chạy
// `node pg/run-pg.mjs p03 p03/thu-nghiem/manifest.json --json …` (nền dựng MỘT lần, ~3 phút),
// rồi kiểm từng tệp bài, từng thí nghiệm, OUTPUT.txt và việc dọn cluster tạm.
import { test, describe, before } from 'node:test';
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, readFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { findPgBin } from '../pg/lib/bin.mjs';
import { splitPsql, parseDirectives } from '../pg/lib/sqlsplit.mjs';
import { parseAligned, transformStderr, normalizeForCompare, normalizeOutput, VOLATILE_BEGIN, VOLATILE_END } from '../pg/lib/output.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

// ---------------------------------------------------------------------------
// Phần 1 — không cần PostgreSQL
test('splitPsql: câu SQL, lệnh meta riêng dòng, \\gset cuối câu, dollar quote', () => {
  const steps = splitPsql([
    '-- expect-value: 1',
    'SELECT 1;',
    '\\timing on',
    "SELECT id AS x FROM t WHERE a = 'b;c' \\gset",
    'DO $$BEGIN PERFORM 1; END$$;',
    'SELECT E\'\\\\\';',
  ].join('\n'));
  assert.deepEqual(steps.map((s) => s.kind), ['sql', 'meta', 'sql', 'sql', 'sql']);
  assert.match(steps[2].text, /\\gset$/);
  assert.equal(parseDirectives(steps[0].text).exp.value, '1');
});

test('parseDirectives: chỉ thị mới của runner PostgreSQL thật', () => {
  const d = parseDirectives([
    '-- @B', '-- expect-blocked', '-- expect-range: 1 2.5', '-- expect-match: Seq Scan', '-- expect-no-match: Index',
    '-- lab: bien-dong', '-- lab: nhat-ky', '-- lab: cho-ms 500', '-- lab: restart', 'SELECT 1;',
  ].join('\n'));
  assert.equal(d.lab.session, 'B');
  assert.equal(d.exp.blocked, true);
  assert.deepEqual(d.exp.range, [1, 2.5]);
  assert.equal(d.exp.match[0].source, 'Seq Scan');
  assert.equal(d.exp.noMatch[0].source, 'Index');
  assert.deepEqual([d.lab.volatile, d.lab.serverLog, d.lab.waitMs, d.lab.restart], [true, true, 500, true]);
  assert.throws(() => parseDirectives('-- expect-sai: 1\nSELECT 1;'), /không hiểu/);
  assert.throws(() => parseDirectives('-- lab: khong-co\nSELECT 1;'), /không hiểu/);
});

test('parseAligned: đọc bảng psql (cột số căn phải, chữ có dấu, NULL, ô nhiều dòng)', () => {
  const t = parseAligned([' code |  ten  | ghi_chu', '------+-------+---------', '    1 | Đặng  | NULL', '   20 | Võ Ba | x|y', '(2 rows)', ''].join('\n'));
  assert.deepEqual(t.headers, ['code', 'ten', 'ghi_chu']);
  assert.deepEqual(t.rows, [['1', 'Đặng', 'NULL'], ['20', 'Võ Ba', 'x|y']]);
  const m = parseAligned([' a | b', '---+---', ' 1 | x+', '   | y', '(1 row)', ''].join('\n'));
  assert.deepEqual(m.rows, [['1', 'x\ny']]);
});

test('transformStderr: lỗi VERBOSITY verbose → dạng ngắn + dòng [lab] có tên ràng buộc', () => {
  const r = transformStderr([
    'ERROR:  23505: duplicate key value violates unique constraint "uq_a"',
    'DETAIL:  Key (a)=(1) already exists.',
    'SCHEMA NAME:  toinoi', 'TABLE NAME:  t', 'CONSTRAINT NAME:  uq_a',
    'LOCATION:  _bt_check_unique, nbtinsert.c:673',
  ].join('\n'));
  assert.equal(r.error.code, '23505');
  assert.equal(r.error.constraint, 'uq_a');
  assert.equal(r.text, [
    'ERROR:  duplicate key value violates unique constraint "uq_a"',
    'DETAIL:  Key (a)=(1) already exists.',
    '-- [lab] SQLSTATE 23505 · constraint uq_a · table t',
  ].join('\n'));
});

test('chuẩn hoá: thời gian đo bị che; khối biến động và ước lượng bị bỏ khi so khớp', () => {
  assert.equal(normalizeOutput(' Seq Scan on t (actual time=0.010..12.345 rows=3.00 loops=1)\n Execution Time: 12.5 ms'),
    ' Seq Scan on t (actual time=T..T rows=3.00 loops=1)\n Execution Time: T ms');
  const a = normalizeForCompare(['x (cost=1.00..2.00 rows=10 width=4)', VOLATILE_BEGIN, ' 12345', VOLATILE_END, '   Buffers: shared hit=3 read=7'].join('\n'));
  const b = normalizeForCompare(['x (cost=9.00..9.50 rows=99 width=4)', VOLATILE_BEGIN, ' 1', ' 2', VOLATILE_END, '   Buffers: shared hit=5 read=5'].join('\n'));
  assert.equal(a, b);
});

test('run-pg.mjs: KHÔNG có PostgreSQL (PG_BIN sai) ⇒ báo rõ và thoát mã 3', () => {
  const r = spawnSync(process.execPath, ['pg/run-pg.mjs', 'p03'], {
    cwd: root, encoding: 'utf8', env: { ...process.env, PG_BIN: path.join(root, 'khong-co-thu-muc-nay') },
  });
  assert.equal(r.status, 3, r.stderr);
  assert.match(r.stderr, /PG_BIN=.*không chứa psql/);
});

// ---------------------------------------------------------------------------
// Phần 2 — PostgreSQL thật
const pg = findPgBin();
const lab = JSON.parse(readFileSync(path.join(root, 'p03/lab.json'), 'utf8'));
const tn = JSON.parse(readFileSync(path.join(root, 'p03/thu-nghiem/manifest.json'), 'utf8'));

describe('P03 trên PostgreSQL thật', { skip: pg.error ? `bỏ qua — ${pg.error}` : false }, () => {
  let run;
  before(async () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'toinoi-test-p03-'));
    const json = path.join(dir, 'ket-qua.json');
    const out = await new Promise((resolve, reject) => {
      const p = spawn(process.execPath, ['pg/run-pg.mjs', 'p03', 'p03/thu-nghiem/manifest.json', '--json', json], { cwd: root });
      let stdout = '';
      let stderr = '';
      p.stdout.setEncoding('utf8');
      p.stderr.setEncoding('utf8');
      p.stdout.on('data', (d) => { stdout += d; });
      p.stderr.on('data', (d) => { stderr += d; });
      p.on('error', reject);
      p.on('exit', (code) => resolve({ code, stdout, stderr }));
    });
    run = { ...out, res: existsSync(json) ? JSON.parse(readFileSync(json, 'utf8')) : null };
    rmSync(dir, { recursive: true, force: true });
  }, { timeout: 30 * 60 * 1000 });

  test('runner thoát 0 và có tệp kết quả JSON', () => {
    assert.ok(run.res, run.stderr.slice(-2000));
    assert.equal(run.code, 0, run.stdout.split('\n').filter((l) => l.includes('FAIL')).join('\n') || run.stderr.slice(-2000));
  });

  test('nền p03 (P01 + P02 lược đồ, fixture 1 triệu chuyến, bộ dữ liệu bài cũ) ĐẠT', () => {
    assert.equal(run.res.bases.p03.ok, true, JSON.stringify(run.res.bases.p03.failures));
  });

  for (const e of lab.tep) {
    test(`${e.bai} · p03/${e.file}`, () => {
      const r = run.res.entries.find((x) => x.file === e.file && !x.manifest?.includes('thu-nghiem'));
      assert.ok(r, 'không có kết quả cho tệp này');
      assert.equal(r.ok, true, r.failures.map((f) => `${f.file}:${f.line} ${f.reason}`).join('\n'));
    });
  }

  for (const e of tn.tep) {
    test(`thí nghiệm ${e.bai} · ${e.file}`, () => {
      const r = run.res.entries.find((x) => x.file === e.file && x.manifest?.includes('thu-nghiem'));
      assert.ok(r, 'không có kết quả cho tệp này');
      assert.equal(r.ok, true, r.failures.map((f) => `${f.file}:${f.line} ${f.reason}`).join('\n'));
    });
  }

  test('đầu ra khớp p03/OUTPUT.txt (sau chuẩn hoá: bỏ thời gian, ước lượng, khối biến động)', () => {
    // Phần của lab = từ đầu tới trước tệp thí nghiệm đầu tiên; OUTPUT.txt bỏ khối Tổng kết.
    const cut = (s) => s.split(/\n-- \[lab\] ===== (?:p03\/thu-nghiem\/|Tổng kết)/)[0];
    const got = normalizeForCompare(cut(run.stdout));
    const want = normalizeForCompare(cut(readFileSync(path.join(root, 'p03/OUTPUT.txt'), 'utf8')));
    if (got !== want) {
      const g = got.split('\n');
      const w = want.split('\n');
      const k = g.findIndex((l, i) => l !== w[i]);
      assert.fail(`khác ở dòng ${k + 1}:\n  OUTPUT.txt: ${w[k]}\n  lần chạy:   ${g[k]}\n(chạy \`npm run p03:output\` nếu thay đổi là có chủ đích)`);
    }
  });

  test('cluster tạm đã được dừng và xoá', () => {
    const m = /cluster (.+?), cổng/.exec(run.stderr);
    assert.ok(m, 'không thấy đường dẫn cluster trong stderr');
    assert.equal(existsSync(m[1]), false, `còn thư mục ${m[1]}`);
  });
});
