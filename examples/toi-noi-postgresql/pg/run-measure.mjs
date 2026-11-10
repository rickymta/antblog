#!/usr/bin/env node
// Lệnh ĐO của lab P03+ (không nằm trong `npm test`). Đọc tệp đặc tả (vd. p03/do-dac.mjs),
// dựng cluster tạm + nền như `pg/run-pg.mjs`, đo lặp và GHI kết quả (Markdown) ra tệp
// mà đặc tả chỉ định (vd. p03/DO-DAC.md), kèm cấu hình máy và cluster.
//
//   node pg/run-measure.mjs p03/do-dac.mjs                # đo tất cả, ghi p03/DO-DAC.md
//   node pg/run-measure.mjs p03/do-dac.mjs --only PG-029  # chỉ nhóm có mã/bài chứa chuỗi
//   node pg/run-measure.mjs p03/do-dac.mjs --lap 3        # đổi số lần đo (thử nhanh)
//   node pg/run-measure.mjs p03/do-dac.mjs --ra tam.md    # ghi ra tệp khác
//
// Mã thoát: 0 xong · 1 có nhóm lỗi · 2 lỗi dùng · 3 không có PostgreSQL.
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve, relative } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { findPgBin } from './lib/bin.mjs';
import { createCluster, attachCluster, LAB_SETTINGS } from './lib/cluster.mjs';
import { PsqlSession } from './lib/session.mjs';
import { Unit, newSummary } from './lib/runner.mjs';
import { machineInfo, cpuLoad } from './lib/machine.mjs';
import { measureOne, runPgbench, runSql, tempStats, writeScript } from './lib/measure.mjs';

const LAB_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const say = (s) => process.stderr.write(s + '\n');

function fmt(x, d = 2) {
  if (x === null || x === undefined || Number.isNaN(x)) return '—';
  return Number(x).toLocaleString('vi-VN', { minimumFractionDigits: d, maximumFractionDigits: d });
}

async function main(argv) {
  const opts = { only: [], lap: null, ra: null, attach: null };
  let specPath = null;
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--only') opts.only.push(argv[++i]);
    else if (argv[i] === '--lap') opts.lap = Number(argv[++i]);
    else if (argv[i] === '--ra') opts.ra = argv[++i];
    else if (argv[i] === '--attach') opts.attach = argv[++i];
    else specPath = argv[i];
  }
  if (!specPath) { say('Cách dùng: node pg/run-measure.mjs <tệp-đặc-tả.mjs> [--only <chuỗi>] [--lap n] [--ra tệp.md]'); return 2; }
  const spec = (await import(pathToFileURL(resolve(specPath)).href)).default;
  const pg = findPgBin();
  if (pg.error) { say(`-- [lab] ${pg.error}`); return 3; }
  const defaults = { lap: opts.lap ?? spec.lap ?? 7, khoi_dong: spec.khoi_dong ?? 1 };
  const t0 = Date.now();
  const may = machineInfo();
  may.cpu_truoc = cpuLoad();
  say(`-- [lab] Tải CPU toàn máy trước khi đo: ${may.cpu_truoc ?? '?'} %`);
  if (may.cpu_truoc > 20) say('-- [lab] CẢNH BÁO: máy đang bận (> 20 % CPU) — số đo sẽ nhiễu; nên đo lại khi máy rảnh.');
  const cluster = opts.attach ? attachCluster(pg, resolve(opts.attach)) : await createCluster(pg, { say });
  const ketQua = [];
  const pgb = [];
  let loi = 0;
  let admin;
  try {
    admin = await new PsqlSession(cluster, 'postgres', 'admin').open();
    const ver = (await admin.run('SHOW server_version;')).stdout.match(/\d+\.\d+/)?.[0];
    // --- nền
    const man = JSON.parse(readFileSync(resolve(LAB_ROOT, spec.nen.manifest), 'utf8'));
    const files = man.nen[spec.nen.khoa].map((f) => resolve(LAB_ROOT, f));
    let nenDb = `nen_do_${spec.nen.khoa}`;
    // --attach (gỡ lỗi): dùng lại database nền do `run-pg.mjs --keep` để lại nếu có.
    if (cluster.attached && (await admin.run(`SELECT count(*) FROM pg_database WHERE datname = 'nen_${spec.nen.khoa}';`)).stdout.match(/\s1\s/)) {
      nenDb = `nen_${spec.nen.khoa}`;
    }
    const exists = (await admin.run(`SELECT count(*) FROM pg_database WHERE datname = '${nenDb}';`)).stdout.match(/\s(\d)\s/)?.[1] === '1';
    if (!(cluster.attached && exists)) {
      const tb = Date.now();
      await admin.run(`DROP DATABASE IF EXISTS ${nenDb} WITH (FORCE);`);
      await admin.run(`CREATE DATABASE ${nenDb};`);
      const summary = newSummary();
      const unit = new Unit({ cluster, write: () => {}, echo: false, baseDir: LAB_ROOT, summary }, nenDb, '');
      for (const f of files) await unit.runFile(f);
      await unit.closeAll();
      if (summary.failures.length) throw new Error(`nền lỗi: ${JSON.stringify(summary.failures.slice(0, 3))}`);
      say(`-- [lab] nền ${spec.nen.khoa}: ${((Date.now() - tb) / 1000).toFixed(1)} s`);
    }
    // --- các nhóm đo
    let k = 0;
    const match = (g) => opts.only.length === 0 || opts.only.some((o) => `${g.id} ${g.bai}`.includes(o));
    for (const g of spec.nhom.filter(match)) {
      k++;
      const db = `do_${String(k).padStart(2, '0')}`;
      await admin.run(`DROP DATABASE IF EXISTS ${db} WITH (FORCE);`);
      await admin.run(`CREATE DATABASE ${db} TEMPLATE ${nenDb} STRATEGY = FILE_COPY;`);
      const sess = await new PsqlSession(cluster, db, 'do').open();
      await sess.run('\\pset format unaligned');
      await sess.run('\\pset tuples_only on');
      const tg = Date.now();
      say(`-- [lab] nhóm ${g.id} (${g.bai}) …`);
      try {
        await runSql(sess, 'SET search_path TO toinoi, public;\n' + (g.chuan_bi ?? ''), g.id);
        if (g.pgbench) {
          const script = writeScript(cluster, `${g.id.replace(/[^A-Za-z0-9]/g, '_')}.sql`, g.pgbench.script);
          for (const cfg of g.pgbench.cau_hinh) {
            const before = await tempStats(sess);
            const r = await runPgbench(cluster, db, sess, { ...cfg, giay: cfg.giay ?? g.pgbench.giay }, script);
            const after = await tempStats(sess);
            pgb.push({ nhom: g, cfg, ...r, temp_files: after.files - before.files, temp_mb: (after.bytes - before.bytes) / 1048576 });
            say(`-- [lab]   ${cfg.nhan}: ${r.tps} tps`);
          }
        }
        for (const m of g.do ?? []) {
          const r = await measureOne(sess, { ...m, lap: opts.lap ?? m.lap }, defaults);
          r.nhom = g;
          ketQua.push(r);
          say(`-- [lab]   ${m.id}: trung vị ${r.trung_vi.toFixed(2)} ms (${r.n} lần)`);
        }
      } catch (e) {
        loi++;
        say(`-- [lab] LỖI nhóm ${g.id}: ${e.message}`);
        ketQua.push({ id: g.id, nhom: g, loi: e.message });
      }
      await sess.close();
      await admin.run(`DROP DATABASE IF EXISTS ${db} WITH (FORCE);`);
      say(`-- [lab]   (${((Date.now() - tg) / 1000).toFixed(1)} s)`);
    }
    // --- ghi kết quả
    const ra = resolve(LAB_ROOT, opts.ra ?? spec.ra);
    may.cpu_sau = cpuLoad();
    writeFileSync(ra, render(spec, may, ver, pg, ketQua, pgb, defaults, (Date.now() - t0) / 1000));
    say(`-- [lab] Đã ghi ${relative(LAB_ROOT, ra)} (${((Date.now() - t0) / 1000).toFixed(0)} s)`);
  } finally {
    try { await admin?.close(); } catch { /* bỏ qua */ }
    cluster.destroySync();
  }
  return loi ? 1 : 0;
}

function render(spec, may, ver, pg, kq, pgb, defaults, giay) {
  const L = [];
  L.push(`# ${spec.tieu_de}`);
  L.push('');
  L.push(`> Tệp này do \`npm run ${spec.lenh}\` (\`node pg/run-measure.mjs ${spec.tep}\`) GHI RA — đừng sửa tay; chạy lại lệnh để đo lại.`);
  L.push(`> Số dưới đây là đo THẬT trên máy ghi ở mục 1, một lần chạy lệnh đo; trên máy khác, con số tuyệt đối sẽ khác — so TỈ LỆ và XU HƯỚNG.`);
  L.push('');
  L.push('## 1. Máy và cấu hình');
  L.push('');
  L.push('| Mục | Giá trị |');
  L.push('|---|---|');
  L.push(`| Thời điểm đo | ${new Date().toISOString().replace('T', ' ').slice(0, 16)} UTC |`);
  L.push(`| CPU | ${may.cpu} · ${may.nhan} luồng logic |`);
  L.push(`| RAM | ${may.ram_gb} GB |`);
  L.push(`| Hệ điều hành | ${may.he_dieu_hanh} |`);
  L.push(`| Ổ đĩa chứa cluster tạm | ${may.o_dia} |`);
  L.push(`| PostgreSQL | ${ver} (${pg.dir}) |`);
  L.push(`| Node.js | ${may.node} |`);
  L.push(`| Cluster tạm | ${Object.entries(LAB_SETTINGS).filter(([k]) => !['log_line_prefix', 'lc_messages', 'datestyle', 'log_timezone'].includes(k)).map(([k, v]) => `${k}=${String(v).replace(/'/g, '')}`).join(', ')} |`);
  L.push(`| Lặp | ${defaults.khoi_dong} lần khởi động (bỏ) + ${defaults.lap} lần đo (trừ khi phép đo ghi khác) |`);
  L.push(`| Tổng thời gian lệnh đo | ${fmt(giay / 60, 1)} phút |`);
  L.push(`| Tải CPU toàn máy (trung bình 3 s) | trước khi đo ${may.cpu_truoc ?? '?'} % · sau khi đo ${may.cpu_sau ?? '?'} % — cao nghĩa là có việc khác chạy song song, số đo nhiễu |`);
  L.push('');
  L.push(spec.phuong_phap.trim());
  L.push('');
  L.push('## 2. Kết quả theo bài');
  const nhoms = [];
  for (const r of kq) if (!nhoms.includes(r.nhom)) nhoms.push(r.nhom);
  for (const g of nhoms) {
    L.push('');
    L.push(`### ${g.bai} · ${g.ten}`);
    L.push('');
    if (g.mo_ta) { L.push(g.mo_ta.trim()); L.push(''); }
    const rs = kq.filter((r) => r.nhom === g);
    if (rs.some((r) => r.loi)) { L.push(`**LỖI khi đo:** ${rs.find((r) => r.loi).loi}`); continue; }
    L.push('| Mã | Phép đo | Cách đo · cache | Trung vị (ms) | Min | Max | N | Buffers (trung vị) | Kế hoạch / ghi chú |');
    L.push('|---|---|---|---:|---:|---:|---:|---:|---|');
    for (const r of rs) {
      const kh = [...r.ke_hoach, ...r.ghi_chu_sort].join('; ') || '—';
      L.push(`| ${r.id} | ${r.ten} | ${r.cach_do} · ${r.cache} | ${fmt(r.trung_vi)} | ${fmt(r.min)} | ${fmt(r.max)} | ${r.n} | ${r.buffers === null ? '—' : fmt(r.buffers, 0)} | ${kh} |`);
    }
    L.push('');
    L.push('<details><summary>Từng lần đo (ms)</summary>');
    L.push('');
    for (const r of rs) L.push(`- ${r.id}: ${r.cac_lan.map((x) => fmt(x)).join(' · ')}`);
    L.push('');
    L.push('</details>');
  }
  const pgNhom = [];
  for (const r of pgb) if (!pgNhom.includes(r.nhom)) pgNhom.push(r.nhom);
  for (const g0 of pgNhom) {
    L.push('');
    L.push(`### ${g0.bai} · ${g0.ten}`);
    L.push('');
    if (g0.mo_ta) { L.push(g0.mo_ta.trim()); L.push(''); }
    L.push('```sql');
    L.push(g0.pgbench.script.trim());
    L.push('```');
    L.push('');
    L.push('| Cấu hình | Phiên đồng thời | tps | Độ trễ TB (ms) | Giao dịch | File tạm | Dữ liệu tạm (MB) | Đỉnh bộ nhớ riêng các backend (MB) |');
    L.push('|---|---:|---:|---:|---:|---:|---:|---:|');
    for (const r of pgb.filter((x) => x.nhom === g0)) {
      L.push(`| ${r.cfg.nhan} | ${r.cfg.clients} | ${fmt(r.tps, 1)} | ${fmt(r.latency_ms, 1)} | ${r.giao_dich ?? '—'} | ${r.temp_files} | ${fmt(r.temp_mb, 0)} | ${r.dinh_bo_nho_rieng_mb ?? 'chưa đo'} |`);
    }
  }
  L.push('');
  L.push('## 3. Giới hạn của các số đo');
  L.push('');
  L.push(spec.gioi_han.trim());
  L.push('');
  return L.join('\n');
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv.slice(2)).then(
    (code) => { process.exitCode = code; },
    (e) => { console.error(e); process.exitCode = 2; },
  );
}
