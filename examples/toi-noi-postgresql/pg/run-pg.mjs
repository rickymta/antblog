#!/usr/bin/env node
// Runner lab P03+ trên PostgreSQL THẬT (không cần dependency npm).
//
// Mỗi lần chạy: initdb một cluster tạm trong thư mục tạm của hệ điều hành
// (toinoi-pg-<ngẫu nhiên>), khởi động ở cổng trống chỉ trên localhost, chạy tệp
// SQL qua psql, in kết quả kiểu psql + chấm chỉ thị expect-*, rồi dừng và xoá
// cluster (kể cả khi lỗi/Ctrl-C). Xem README.md gốc của lab, mục «P03+».
//
//   node pg/run-pg.mjs p03                         # nền + mọi tệp bài của p03/lab.json
//   node pg/run-pg.mjs p03/thu-nghiem/manifest.json # tệp thí nghiệm, mỗi tệp một bản sao nền
//   node pg/run-pg.mjs p03 --only 23               # chỉ mục có tên chứa "23"
//   node pg/run-pg.mjs --nen p03/lab.json:p03 tam.sql  # chạy tệp lẻ trên bản sao nền p03
//   node pg/run-pg.mjs a.sql b.sql                 # tệp lẻ trên một database trống, chung phiên
//
// Mã thoát: 0 ĐẠT · 1 có FAIL · 2 lỗi dùng/runner · 3 không có PostgreSQL.
import { readFileSync, readdirSync, statSync, writeFileSync, existsSync } from 'node:fs';
import { basename, dirname, join, resolve, relative } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { findPgBin } from './lib/bin.mjs';
import { createCluster, attachCluster } from './lib/cluster.mjs';
import { PsqlSession } from './lib/session.mjs';
import { Unit, newSummary, formatSummary } from './lib/runner.mjs';

export const LAB_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const EXIT_NO_PG = 3;

function usage() {
  return `Cách dùng: node pg/run-pg.mjs [tùy chọn] <thư-mục-có-lab.json | manifest.json | tệp.sql …>
  --only <chuỗi>          chỉ chạy mục manifest có tên tệp chứa chuỗi (lặp được)
  --nen <manifest>:<khoá> chạy các tệp .sql lẻ trên bản sao nền <khoá> của manifest
  --an-nen                không in đầu ra của các tệp nền (vẫn chạy và chấm)
  --no-echo               không in lại câu lệnh, chỉ in kết quả
  --json <tệp>            ghi kết quả từng mục (ĐẠT/FAIL) ra tệp JSON
  --keep                  giữ cluster chạy sau khi xong (để gỡ lỗi; tự dừng + xoá sau)
  --attach <thư-mục>      GỠ LỖI: dùng lại cluster đã --keep (dùng lại database nền nếu đã có)`;
}

/** Mở rộng thư mục thành các tệp NN-*.sql theo tên (bỏ *-rollback.sql). */
function expandSql(p) {
  if (statSync(p).isDirectory()) {
    return readdirSync(p).filter((f) => /^\d\d-.*\.sql$/.test(f) && !/-rollback\.sql$/.test(f)).sort().map((f) => join(p, f));
  }
  return [p];
}

function loadManifest(p) {
  const m = JSON.parse(readFileSync(p, 'utf8'));
  if (!m.nen || !Array.isArray(m.tep)) throw new Error(`${p}: manifest cần "nen" (object) và "tep" (mảng)`);
  return { ...m, path: p, dir: dirname(p) };
}

export async function main(argv, { write = (s) => process.stdout.write(s), say = (s) => process.stderr.write(s + '\n') } = {}) {
  const opts = { only: [], echo: true, showBase: true, json: null, keep: false, nen: null, attach: null };
  const targets = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--only') opts.only.push(argv[++i]);
    else if (a === '--nen') opts.nen = argv[++i];
    else if (a === '--an-nen') opts.showBase = false;
    else if (a === '--no-echo') opts.echo = false;
    else if (a === '--json') opts.json = argv[++i];
    else if (a === '--keep') opts.keep = true;
    else if (a === '--attach') opts.attach = argv[++i];
    else if (a === '-h' || a === '--help') { write(usage() + '\n'); return 0; }
    else if (a.startsWith('--')) { say(`Tùy chọn không hiểu: ${a}\n${usage()}`); return 2; }
    else targets.push(a);
  }
  if (targets.length === 0) { say(usage()); return 2; }

  const pg = findPgBin();
  if (pg.error) {
    say(`-- [lab] ${pg.error}`);
    say('-- [lab] P03+ cần PostgreSQL 18 thật (initdb, pg_ctl, psql). P01–P02 vẫn chạy được bằng `npm run p01` / `npm run p02` (PGlite).');
    return EXIT_NO_PG;
  }

  const t0 = Date.now();
  const cluster = opts.attach ? attachCluster(pg, resolve(opts.attach)) : await createCluster(pg, { keep: opts.keep, say });
  const summary = newSummary();
  const ctx = { cluster, write, echo: opts.echo, baseDir: LAB_ROOT, summary };
  const results = { ok: false, pg: { version: pg.version, dir: pg.dir }, entries: [], bases: {} };
  let admin;
  try {
    admin = await new PsqlSession(cluster, 'postgres', 'admin').open();
    const ver = (await admin.run('SELECT current_setting(\'server_version\') AS v \\gset\n\\echo :v')).stdout.trim();
    write(`-- [lab] Engine: PostgreSQL ${ver} (server thật, cluster tạm do pg/run-pg.mjs tạo), client psql ${pg.version}\n`);
    write(`-- [lab] Cluster: UTF8, locale provider builtin C.UTF-8, TimeZone Asia/Ho_Chi_Minh, autovacuum off, jit off, không song song — cấu hình đầy đủ ở pg/lib/cluster.mjs\n`);
    write(`-- [lab] Phiên psql: -X, ON_ERROR_STOP=0, VERBOSITY=verbose (in lại dạng ngắn), \\pset null NULL\n`);
    say(`-- [lab] PostgreSQL ${ver} từ ${pg.dir} (${pg.source}); cluster ${cluster.root}, cổng ${cluster.port}`);

    const bases = new Map(); // khoá nội dung → tên database nền
    let counter = 0;
    const ensureBase = async (key, files, manifestDir) => {
      const abs = files.map((f) => resolve(LAB_ROOT, f));
      const sig = abs.join('|');
      if (bases.has(sig)) return bases.get(sig);
      const db = `nen_${key.replace(/[^A-Za-z0-9_]/g, '_').toLowerCase()}`;
      const tb = Date.now();
      if (cluster.attached) {
        const ex = await admin.run(`SELECT count(*) AS n FROM pg_database WHERE datname = '${db}' \\gset\n\\echo :n`);
        if (ex.stdout.trim() === '1') {
          say(`-- [lab] --attach: dùng lại database nền có sẵn ${db} (không chạy lại tệp nền)`);
          const info = { db, failures: [] };
          bases.set(sig, info);
          return info;
        }
      }
      await admin.run(`CREATE DATABASE ${db};`);
      const unit = new Unit({ ...ctx, write: opts.showBase ? write : () => {} }, db, `nền ${key} (database ${db})`);
      for (const f of abs) await unit.runFile(f);
      await unit.closeAll();
      const secs = ((Date.now() - tb) / 1000).toFixed(1);
      say(`-- [lab] nền ${key}: dựng xong sau ${secs} s`);
      results.bases[key] = { db, seconds: Number(secs), ok: unit.failures.length === 0, failures: unit.failures };
      const info = { db, failures: unit.failures };
      bases.set(sig, info);
      return info;
    };
    const runOnClone = async (base, file, title, entry) => {
      counter++;
      const db = `bai_${String(counter).padStart(2, '0')}`;
      const tc = Date.now();
      if (cluster.attached) await admin.run(`DROP DATABASE IF EXISTS ${db} WITH (FORCE);`);
      const r = await admin.run(`CREATE DATABASE ${db} TEMPLATE ${base.db} STRATEGY = FILE_COPY;`);
      if (r.error) throw new Error(`Không tạo được bản sao ${db}: ${r.stderr}`);
      const unit = new Unit(ctx, db, `${title} (database ${db})`);
      await unit.runFile(file);
      await unit.closeAll();
      await admin.run(`DROP DATABASE ${db} WITH (FORCE);`);
      const ok = unit.failures.length === 0 && base.failures.length === 0;
      results.entries.push({ ...entry, ok, seconds: (Date.now() - tc) / 1000, failures: [...base.failures, ...unit.failures] });
      say(`-- [lab] ${ok ? 'ĐẠT' : 'FAIL'} ${relative(LAB_ROOT, file).replace(/\\/g, '/')} (${((Date.now() - tc) / 1000).toFixed(1)} s)`);
    };

    // Gom các tệp .sql lẻ liên tiếp thành một đơn vị.
    let loose = [];
    const flushLoose = async () => {
      if (loose.length === 0) return;
      const files = loose;
      loose = [];
      if (opts.nen) {
        const k = opts.nen.lastIndexOf(':');
        if (k <= 0) throw new Error('--nen cần dạng <manifest.json>:<khoá>');
        const m = loadManifest(resolve(opts.nen.slice(0, k)));
        const key = opts.nen.slice(k + 1);
        if (!m.nen[key]) throw new Error(`manifest ${m.path} không có nền "${key}"`);
        const base = await ensureBase(key, m.nen[key], m.dir);
        for (const f of files) await runOnClone(base, f, `bản sao nền ${key}`, { file: relative(LAB_ROOT, f).replace(/\\/g, '/'), nen: key });
      } else {
        counter++;
        const db = `lab_${String(counter).padStart(2, '0')}`;
        await admin.run(`DROP DATABASE IF EXISTS ${db} WITH (FORCE);`);
        await admin.run(`CREATE DATABASE ${db};`);
        const unit = new Unit(ctx, db, '');
        for (const f of files) await unit.runFile(f);
        await unit.closeAll();
        results.entries.push({ files: files.map((f) => relative(LAB_ROOT, f).replace(/\\/g, '/')), ok: unit.failures.length === 0, failures: unit.failures });
      }
    };

    for (const t of targets) {
      const p = resolve(t);
      if (!existsSync(p)) throw new Error(`Không có tệp/thư mục: ${t}`);
      const manifestPath = statSync(p).isDirectory() && existsSync(join(p, 'lab.json')) ? join(p, 'lab.json')
        : p.endsWith('.json') ? p : null;
      if (!manifestPath) { loose.push(...expandSql(p)); continue; }
      await flushLoose();
      const m = loadManifest(manifestPath);
      const entries = m.tep.filter((e) => opts.only.length === 0 || opts.only.some((o) => e.file.includes(o)));
      if (entries.length === 0) say(`-- [lab] ${t}: không mục nào khớp --only`);
      for (const e of entries) {
        if (!m.nen[e.nen]) throw new Error(`${manifestPath}: mục ${e.file} dùng nền "${e.nen}" không khai báo`);
        const base = await ensureBase(e.nen, m.nen[e.nen], m.dir);
        const file = resolve(m.dir, e.file);
        await runOnClone(base, file, `${e.bai ? e.bai + ' · ' : ''}bản sao nền ${e.nen}`, { file: e.file, bai: e.bai ?? null, nen: e.nen, manifest: relative(LAB_ROOT, manifestPath).replace(/\\/g, '/') });
      }
    }
    await flushLoose();
  } catch (e) {
    say(`-- [lab] Lỗi runner: ${e?.stack ?? e}`);
    summary.failures.push({ file: '(runner)', line: 0, reason: String(e?.message ?? e) });
  } finally {
    write(formatSummary(summary));
    results.ok = summary.failures.length === 0;
    results.summary = { ...summary, failures: summary.failures.length };
    results.seconds = (Date.now() - t0) / 1000;
    if (opts.json) writeFileSync(opts.json, JSON.stringify(results, null, 2));
    try { await admin?.close(); } catch { /* bỏ qua */ }
    if (opts.keep) {
      say(`-- [lab] --keep: cluster VẪN CHẠY. Kết nối: psql -h localhost -p ${cluster.port} -U postgres -d postgres`);
      say(`-- [lab] Dừng và xoá khi xong: pg_ctl stop -D "${cluster.dataDir}" -m fast  rồi xoá thư mục ${cluster.root}`);
    } else {
      cluster.destroySync();
    }
    say(`-- [lab] Tổng thời gian: ${((Date.now() - t0) / 1000).toFixed(1)} s`);
  }
  return summary.failures.length === 0 ? 0 : 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.stdout.on('error', (e) => { if (e.code === 'EPIPE') process.exit(0); else throw e; });
  main(process.argv.slice(2)).then(
    (code) => { process.exitCode = code; },
    (e) => { console.error(e); process.exitCode = 2; },
  );
}
