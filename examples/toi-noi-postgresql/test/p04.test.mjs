// P04 · Giao dịch, MVCC và đồng thời — chạy trên PostgreSQL 18 THẬT qua pg/run-pg.mjs và
// pg/run-dong-thoi.mjs.
//
// Phần 1 (luôn chạy, không cần PostgreSQL): chỉ thị mới của runner (cho-xong, expect-mat-ket-noi),
// che PID/xid trong thông báo của tệp nhiều phiên, bộ thử lại giaoDich(), mã thoát khi KHÔNG có PostgreSQL.
// Phần 2 (chỉ khi có PostgreSQL ≥ 18; không có ⇒ SKIP): một lần `node pg/run-pg.mjs p04
// p04/thu-nghiem/manifest.json --json …` rồi kiểm từng tệp bài, từng thí nghiệm, OUTPUT.txt, dọn cluster.
// Phần 3 (chỉ khi có PostgreSQL): phép thử ĐỒNG THỜI cỡ nhỏ `node pg/run-dong-thoi.mjs p04/dong-thoi.mjs
// --nho --json …` — chỉ kiểm BẤT BIẾN "phải đúng" (số đếm dao động, không so).
import { test, describe, before } from 'node:test';
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, readFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { findPgBin } from '../pg/lib/bin.mjs';
import { parseDirectives } from '../pg/lib/sqlsplit.mjs';
import { normalizeForCompare } from '../pg/lib/output.mjs';
import { Unit, formatSummary, newSummary } from '../pg/lib/runner.mjs';
import { giaoDich, BoDem, LoiSql, rng } from '../pg/lib/dong-thoi.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

// ---------------------------------------------------------------------------
// Phần 1 — không cần PostgreSQL
test('parseDirectives: lab: cho-xong và expect-mat-ket-noi', () => {
  const d = parseDirectives(['-- @B', '-- lab: cho-xong', '-- expect-mat-ket-noi', '-- expect-blocked', 'COMMIT;'].join('\n'));
  assert.equal(d.lab.session, 'B');
  assert.equal(d.lab.waitDone, true);
  assert.equal(d.exp.lost, true);
  assert.equal(d.exp.blocked, true);
});

test('che PID/xid/thời gian trong thông báo deadlock của tệp nhiều phiên', () => {
  const u = new Unit({}, 'db', '');
  u.multi = true;
  u.pidLabels.set(16468, 'B');
  u.pidLabels.set(31412, 'A');
  assert.equal(
    u.maskMulti('DETAIL:  Process 16468 waits for ShareLock on transaction 758; blocked by process 31412.'),
    'DETAIL:  Process [B] waits for ShareLock on transaction <XID>; blocked by process [A].',
  );
  assert.equal(u.maskMulti('LOG:  process 16468 detected deadlock while waiting for ShareLock on transaction 758 after 201.681 ms'),
    'LOG:  process [B] detected deadlock while waiting for ShareLock on transaction <XID> after T ms');
  assert.equal(u.maskMulti('DETAIL:  Process holding the lock: 31412. Wait queue: 16468, 999.'),
    'DETAIL:  Process holding the lock: [A]. Wait queue: [B], [pid khác].');
  u.multi = false; // tệp một phiên (P03): không đổi gì
  assert.equal(u.maskMulti('Process 16468 transaction 758'), 'Process 16468 transaction 758');
});

test('formatSummary: dòng "mất kết nối" chỉ xuất hiện khi có kiểm (P03 không đổi)', () => {
  const s = newSummary();
  assert.doesNotMatch(formatSummary(s), /mất kết nối/);
  s.lostChecks = 2;
  assert.match(formatSummary(s), /Kiểm mất kết nối \(expect-mat-ket-noi\): 2/);
});

test('giaoDich(): thử lại TOÀN BỘ giao dịch khi 40001, dừng khi hết lượt', async () => {
  // Phiên giả: COMMIT lần 1 và 2 lỗi 40001, lần 3 thành công.
  const log = [];
  let commits = 0;
  const w = {
    async q(sql) { log.push(sql); return { error: false, rows: [], raw: {} }; },
    async qx(sql) {
      log.push(sql);
      if (sql.startsWith('COMMIT') && ++commits < 3) {
        throw new LoiSql({ sqlstate: '40001', stderr: 'ERROR:  40001: could not serialize access' }, sql);
      }
      return { rows: [] };
    },
  };
  const dem = new BoDem();
  let goi = 0;
  const r = await giaoDich(w, async () => { goi++; return { x: 1 }; }, { muc: 'SERIALIZABLE', toi_da: 5, dem, co_so_ms: 0, ngau_nhien: rng(1) });
  assert.equal(r.ok, true);
  assert.equal(r.lan, 3);
  assert.equal(goi, 3, 'thân giao dịch phải chạy lại từ đầu mỗi lần');
  assert.deepEqual(dem.obj(), { loi_40001: 2, thu_lai: 2 });
  assert.equal(log.filter((x) => x.startsWith('BEGIN ISOLATION LEVEL SERIALIZABLE')).length, 3);
  commits = -100; // luôn lỗi ⇒ hết lượt
  const dem2 = new BoDem();
  const r2 = await giaoDich(w, async () => ({}), { toi_da: 2, dem: dem2, co_so_ms: 0 });
  assert.equal(r2.ok, false);
  assert.equal(r2.sqlstate, '40001');
  assert.deepEqual(dem2.obj(), { het_luot_thu: 1, loi_40001: 2, thu_lai: 1 });
});

test('run-pg.mjs và run-dong-thoi.mjs: KHÔNG có PostgreSQL (PG_BIN sai) ⇒ thoát mã 3', () => {
  const env = { ...process.env, PG_BIN: path.join(root, 'khong-co-thu-muc-nay') };
  const a = spawnSync(process.execPath, ['pg/run-pg.mjs', 'p04'], { cwd: root, encoding: 'utf8', env });
  assert.equal(a.status, 3, a.stderr);
  const b = spawnSync(process.execPath, ['pg/run-dong-thoi.mjs', 'p04/dong-thoi.mjs', '--nho'], { cwd: root, encoding: 'utf8', env });
  assert.equal(b.status, 3, b.stderr);
  assert.match(b.stderr, /PG_BIN=.*không chứa psql/);
});

// ---------------------------------------------------------------------------
// Phần 2 + 3 — PostgreSQL thật
const pg = findPgBin();
const lab = JSON.parse(readFileSync(path.join(root, 'p04/lab.json'), 'utf8'));
const tn = JSON.parse(readFileSync(path.join(root, 'p04/thu-nghiem/manifest.json'), 'utf8'));
const skip = pg.error ? `bỏ qua — ${pg.error}` : false;

function chay(args) {
  return new Promise((resolve, reject) => {
    const p = spawn(process.execPath, args, { cwd: root });
    let stdout = '';
    let stderr = '';
    p.stdout.setEncoding('utf8');
    p.stderr.setEncoding('utf8');
    p.stdout.on('data', (d) => { stdout += d; });
    p.stderr.on('data', (d) => { stderr += d; });
    p.on('error', reject);
    p.on('exit', (code) => resolve({ code, stdout, stderr }));
  });
}

describe('P04 trên PostgreSQL thật — tệp bài + thí nghiệm', { skip }, () => {
  let run;
  before(async () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'toinoi-test-p04-'));
    const json = path.join(dir, 'ket-qua.json');
    const out = await chay(['pg/run-pg.mjs', 'p04', 'p04/thu-nghiem/manifest.json', '--json', json]);
    run = { ...out, res: existsSync(json) ? JSON.parse(readFileSync(json, 'utf8')) : null };
    rmSync(dir, { recursive: true, force: true });
  }, { timeout: 15 * 60 * 1000 });

  test('runner thoát 0 và có tệp kết quả JSON', () => {
    assert.ok(run.res, run.stderr.slice(-2000));
    assert.equal(run.code, 0, run.stdout.split('\n').filter((l) => l.includes('FAIL')).join('\n') || run.stderr.slice(-2000));
  });

  test('nền p04 (lược đồ P01 + P02 + p04/00, fixture nhỏ) ĐẠT', () => {
    assert.equal(run.res.bases.p04.ok, true, JSON.stringify(run.res.bases.p04.failures));
  });

  for (const e of lab.tep) {
    test(`${e.bai} · p04/${e.file}`, () => {
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

  test('đầu ra khớp p04/OUTPUT.txt (lịch chạy nhiều phiên tất định sau chuẩn hoá)', () => {
    const cut = (s) => s.split(/\n-- \[lab\] ===== (?:p04\/thu-nghiem\/|Tổng kết)/)[0];
    const got = normalizeForCompare(cut(run.stdout));
    const want = normalizeForCompare(cut(readFileSync(path.join(root, 'p04/OUTPUT.txt'), 'utf8')));
    if (got !== want) {
      const g = got.split('\n');
      const w = want.split('\n');
      const k = g.findIndex((l, i) => l !== w[i]);
      assert.fail(`khác ở dòng ${k + 1}:\n  OUTPUT.txt: ${w[k]}\n  lần chạy:   ${g[k]}\n(chạy \`npm run p04:output\` nếu thay đổi là có chủ đích)`);
    }
  });

  test('cluster tạm đã được dừng và xoá', () => {
    const m = /cluster (.+?), cổng/.exec(run.stderr);
    assert.ok(m, 'không thấy đường dẫn cluster trong stderr');
    assert.equal(existsSync(m[1]), false, `còn thư mục ${m[1]}`);
  });
});

describe('P04 trên PostgreSQL thật — phép thử đồng thời cỡ nhỏ (chỉ kiểm bất biến)', { skip }, () => {
  let run;
  const spec = 'p04/dong-thoi.mjs';
  before(async () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'toinoi-test-p04dt-'));
    const json = path.join(dir, 'dong-thoi.json');
    const out = await chay(['pg/run-dong-thoi.mjs', spec, '--nho', '--json', json]);
    run = { ...out, res: existsSync(json) ? JSON.parse(readFileSync(json, 'utf8')) : null };
    rmSync(dir, { recursive: true, force: true });
  }, { timeout: 10 * 60 * 1000 });

  test('lệnh thoát 0 (mọi bất biến "phải đúng" đều đúng)', () => {
    assert.ok(run.res, run.stderr.slice(-2000));
    assert.equal(run.code, 0, run.stderr.split('\n').filter((l) => /KHÔNG ĐẠT|LỖI/.test(l)).join('\n') || run.stderr.slice(-2000));
  });

  const ids = ['K040-a', 'K040-b', 'K040-c', 'K040-d', 'K040-e', 'K036-skip', 'K036-cho', 'K036-nowait',
    'K037-theo-chieu', 'K037-theo-id', 'K038-rr', 'K038-ser', 'K038-ser-index', 'K039'];
  for (const id of ids) {
    test(`${id}: bất biến "phải đúng" đều đúng`, () => {
      const kb = run.res.kich_ban.find((x) => x.id === id);
      assert.ok(kb, 'không có kịch bản này trong kết quả');
      for (const lan of kb.lan) {
        assert.equal(lan.loi, undefined, lan.loi);
        for (const b of lan.bat_bien.filter((x) => x.phai_dung)) {
          assert.equal(b.dat, true, `${b.ten}: ${b.gia_tri} (mong đợi ${b.mong_doi})`);
        }
      }
    });
  }

  test('cluster tạm của phép thử đồng thời đã được dừng và xoá', () => {
    const m = /cluster (.+?), cổng/.exec(run.stderr);
    assert.ok(m, 'không thấy đường dẫn cluster trong stderr');
    assert.equal(existsSync(m[1]), false, `còn thư mục ${m[1]}`);
  });
});
