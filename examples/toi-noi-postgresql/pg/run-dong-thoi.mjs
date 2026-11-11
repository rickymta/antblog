#!/usr/bin/env node
// Lệnh PHÉP THỬ ĐỒNG THỜI (P04+): N phiên psql chạy đua thật, lặp nhiều lần, đếm kết quả và
// kiểm bất biến. Đọc tệp đặc tả (vd. p04/dong-thoi.mjs), dựng cluster tạm + nền như
// pg/run-pg.mjs; mỗi LẦN của mỗi kịch bản chạy trên một database mới chép từ nền.
//
//   node pg/run-dong-thoi.mjs p04/dong-thoi.mjs              # cỡ đầy đủ, ghi tệp kết quả (p04/DO-DAC.md)
//   node pg/run-dong-thoi.mjs p04/dong-thoi.mjs --nho        # cỡ NHỎ (vài giây/kịch bản), KHÔNG ghi tệp kết quả
//   node pg/run-dong-thoi.mjs p04/dong-thoi.mjs --only K038  # chỉ kịch bản có mã/bài chứa chuỗi
//   node pg/run-dong-thoi.mjs p04/dong-thoi.mjs --lap 5      # đổi số lần lặp
//   node pg/run-dong-thoi.mjs p04/dong-thoi.mjs --ra tam.md --json kq.json
//
// Mã thoát: 0 mọi bất biến "phải đúng" đều đạt · 1 có bất biến bị vi phạm / kịch bản lỗi ·
// 2 lỗi dùng · 3 không có PostgreSQL.
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve, relative } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { findPgBin } from './lib/bin.mjs';
import { createCluster, LAB_SETTINGS } from './lib/cluster.mjs';
import { PsqlSession } from './lib/session.mjs';
import { Unit, newSummary } from './lib/runner.mjs';
import { splitPsql } from './lib/sqlsplit.mjs';
import { machineInfo, cpuLoad } from './lib/machine.mjs';
import { Worker, BoDem, rng, giaoDich, sleep } from './lib/dong-thoi.mjs';

const LAB_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const say = (s) => process.stderr.write(s + '\n');

function fmt(x, d = 0) {
  if (x === null || x === undefined || Number.isNaN(x)) return '—';
  return Number(x).toLocaleString('vi-VN', { minimumFractionDigits: d, maximumFractionDigits: d });
}

async function runSqlBlock(w, sql, where) {
  for (const st of splitPsql(sql)) {
    const r = await w.q(st.text);
    if (r.error) throw new Error(`${where}: lỗi ở «${st.text.slice(0, 100)}»: ${r.raw.stderr.trim()}`);
  }
}

export async function main(argv) {
  const opts = { only: [], lap: null, ra: null, json: null, nho: false };
  let specPath = null;
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--only') opts.only.push(argv[++i]);
    else if (a === '--lap') opts.lap = Number(argv[++i]);
    else if (a === '--ra') opts.ra = argv[++i];
    else if (a === '--json') opts.json = argv[++i];
    else if (a === '--nho') opts.nho = true;
    else if (a.startsWith('--')) { say(`Tùy chọn không hiểu: ${a}`); return 2; }
    else specPath = a;
  }
  if (!specPath) { say('Cách dùng: node pg/run-dong-thoi.mjs <đặc-tả.mjs> [--nho] [--only <chuỗi>] [--lap n] [--ra tệp.md] [--json tệp.json]'); return 2; }
  const spec = (await import(pathToFileURL(resolve(specPath)).href)).default;
  const pg = findPgBin();
  if (pg.error) { say(`-- [lab] ${pg.error}`); return 3; }
  const t0 = Date.now();
  const may = machineInfo();
  if (!opts.nho) {
    may.cpu_truoc = cpuLoad();
    say(`-- [lab] Tải CPU toàn máy trước khi chạy: ${may.cpu_truoc ?? '?'} %`);
  }
  const cluster = await createCluster(pg, { say });
  say(`-- [lab] PostgreSQL ${pg.version} từ ${pg.dir}; cluster ${cluster.root}, cổng ${cluster.port}`);
  const ketQua = [];
  let viPham = 0;
  let admin;
  try {
    admin = await new PsqlSession(cluster, 'postgres', 'admin').open();
    const ver = (await admin.run('SHOW server_version;')).stdout.match(/\d+\.\d+/)?.[0];
    // --- nền
    const man = JSON.parse(readFileSync(resolve(LAB_ROOT, spec.nen.manifest), 'utf8'));
    const files = man.nen[spec.nen.khoa].map((f) => resolve(LAB_ROOT, f));
    const nenDb = `nen_dt_${spec.nen.khoa}`;
    const tb = Date.now();
    await admin.run(`CREATE DATABASE ${nenDb};`);
    const summary = newSummary();
    const unit = new Unit({ cluster, write: () => {}, echo: false, baseDir: LAB_ROOT, summary }, nenDb, '');
    for (const f of files) await unit.runFile(f);
    await unit.closeAll();
    if (summary.failures.length) throw new Error(`nền lỗi: ${JSON.stringify(summary.failures.slice(0, 3))}`);
    say(`-- [lab] nền ${spec.nen.khoa}: ${((Date.now() - tb) / 1000).toFixed(1)} s`);

    let k = 0;
    const match = (kb) => opts.only.length === 0 || opts.only.some((o) => `${kb.id} ${kb.bai}`.includes(o));
    for (const kb of spec.kich_ban.filter(match)) {
      const co = opts.nho ? kb.co_nho : kb.co;
      const soLan = opts.lap ?? (opts.nho ? (spec.lap_nho ?? 1) : (kb.lap ?? spec.lap ?? 3));
      const kq = { kb, co, lan: [] };
      ketQua.push(kq);
      for (let lan = 1; lan <= soLan; lan++) {
        k++;
        const db = `dt_${String(k).padStart(3, '0')}`;
        await admin.run(`CREATE DATABASE ${db} TEMPLATE ${nenDb} STRATEGY = FILE_COPY;`);
        const dieuPhoi = await new Worker(cluster, db, 'dieu-phoi').open();
        const workers = [];
        const dem = new BoDem();
        const ghi = { lan, ok: true };
        try {
          if (kb.chuan_bi) await runSqlBlock(dieuPhoi, typeof kb.chuan_bi === 'function' ? kb.chuan_bi(co) : kb.chuan_bi, kb.id);
          for (let i = 1; i <= co.phien; i++) workers.push(await new Worker(cluster, db, `w${i}`).open());
          const tc = Date.now();
          await kb.chay({ dieuPhoi, workers, co, dem, ngau_nhien: rng(1000 * lan + 7), giaoDich, sleep, Worker, cluster, db });
          ghi.giay = (Date.now() - tc) / 1000;
          ghi.dem = dem.obj();
          ghi.bat_bien = [];
          for (const b of kb.bat_bien) {
            // Bất biến đọc bằng SQL, hoặc tính từ bộ đếm phía client (tu_dem) — vd. số lần 40P01.
            const v = b.tu_dem ? b.tu_dem(ghi.dem, co) : await dieuPhoi.v(typeof b.sql === 'function' ? b.sql(co) : b.sql);
            const mong = b.mong_doi === undefined ? null : String(typeof b.mong_doi === 'function' ? b.mong_doi(co) : b.mong_doi);
            const dat = mong === null ? null : String(v) === mong;
            ghi.bat_bien.push({ ten: b.ten, gia_tri: v, mong_doi: mong, dat, phai_dung: !!b.phai_dung });
            if (b.phai_dung && !dat) { viPham++; ghi.ok = false; }
          }
          if (kb.chi_so) ghi.chi_so = kb.chi_so(ghi, co);
        } catch (e) {
          viPham++;
          ghi.ok = false;
          ghi.loi = e.message;
          say(`-- [lab] LỖI ${kb.id} lần ${lan}: ${e.stack ?? e.message}`);
        }
        for (const w of workers) await w.close();
        await dieuPhoi.close();
        await admin.run(`DROP DATABASE IF EXISTS ${db} WITH (FORCE);`);
        kq.lan.push(ghi);
        const bb = (ghi.bat_bien ?? []).map((b) => `${b.ten}=${b.gia_tri}${b.dat === false ? (b.phai_dung ? ' ✖' : ' (vi phạm — minh hoạ)') : ''}`).join(' · ');
        say(`-- [lab] ${ghi.ok ? 'ĐẠT' : 'KHÔNG ĐẠT'} ${kb.id} lần ${lan}/${soLan} (${ghi.giay?.toFixed(1) ?? '?'} s): ${JSON.stringify(ghi.dem ?? {})} · ${bb}`);
      }
    }
    const tong = (Date.now() - t0) / 1000;
    if (!opts.nho || opts.ra) {
      const ra = resolve(LAB_ROOT, opts.ra ?? spec.ra);
      may.cpu_sau = opts.nho ? null : cpuLoad();
      writeFileSync(ra, render(spec, may, ver, pg, ketQua, tong, opts));
      say(`-- [lab] Đã ghi ${relative(LAB_ROOT, ra)}`);
    }
    if (opts.json) {
      writeFileSync(opts.json, JSON.stringify({
        ok: viPham === 0, giay: tong, pg: pg.version,
        kich_ban: ketQua.map((x) => ({ id: x.kb.id, bai: x.kb.bai, co: x.co, lan: x.lan })),
      }, null, 2));
    }
    say(`-- [lab] ${viPham === 0 ? 'KẾT QUẢ: ĐẠT — mọi bất biến "phải đúng" đều đúng' : `KẾT QUẢ: KHÔNG ĐẠT — ${viPham} vi phạm/lỗi`} · tổng ${tong.toFixed(1)} s`);
  } finally {
    try { await admin?.close(); } catch { /* bỏ qua */ }
    cluster.destroySync();
  }
  return viPham === 0 ? 0 : 1;
}

function render(spec, may, ver, pg, ketQua, giay, opts) {
  const L = [];
  L.push(`# ${spec.tieu_de}`);
  L.push('');
  L.push(`> Tệp này do \`npm run ${spec.lenh}\` (\`node pg/run-dong-thoi.mjs ${spec.tep}\`) GHI RA — đừng sửa tay; chạy lại lệnh để đo lại.`);
  L.push('> Số đếm dưới đây là kết quả THẬT của một lần chạy lệnh trên máy ở mục 1. Các phiên chạy đua thật nên số đếm (lượt thắng, số lần thử lại, số lỗi) DAO ĐỘNG giữa các lần chạy và giữa các máy; chỉ các BẤT BIẾN được kiểm cứng. Trích số đếm thì trích kèm khoảng min–max và cấu hình máy.');
  L.push('');
  L.push('## 1. Máy và cấu hình');
  L.push('');
  L.push('| Mục | Giá trị |');
  L.push('|---|---|');
  L.push(`| Thời điểm chạy | ${new Date().toISOString().replace('T', ' ').slice(0, 16)} UTC |`);
  L.push(`| CPU | ${may.cpu} · ${may.nhan} luồng logic |`);
  L.push(`| RAM | ${may.ram_gb} GB |`);
  L.push(`| Hệ điều hành | ${may.he_dieu_hanh} |`);
  L.push(`| Ổ đĩa chứa cluster tạm | ${may.o_dia} |`);
  L.push(`| PostgreSQL | ${ver} (${pg.dir}) |`);
  L.push(`| Node.js | ${may.node} |`);
  L.push(`| Cluster tạm | ${Object.entries(LAB_SETTINGS).filter(([k]) => !['log_line_prefix', 'lc_messages', 'datestyle', 'log_timezone'].includes(k)).map(([k, v]) => `${k}=${String(v).replace(/'/g, '')}`).join(', ')} |`);
  L.push(`| Cỡ | ${opts.nho ? 'NHỎ (--nho)' : 'đầy đủ'} |`);
  L.push(`| Tổng thời gian lệnh | ${fmt(giay / 60, 1)} phút |`);
  L.push(`| Tải CPU toàn máy (trung bình 3 s) | trước ${may.cpu_truoc ?? '?'} % · sau ${may.cpu_sau ?? '?'} % |`);
  L.push('');
  L.push(spec.phuong_phap.trim());
  L.push('');
  L.push('## 2. Kết quả theo kịch bản');
  for (const { kb, co, lan } of ketQua) {
    L.push('');
    L.push(`### ${kb.id} · ${kb.bai} · ${kb.ten}`);
    L.push('');
    L.push(kb.mo_ta.trim());
    L.push('');
    L.push(`Tải: ${Object.entries(co).map(([k, v]) => `${k} = ${v}`).join(' · ')} · ${lan.length} lần.`);
    L.push('');
    const keys = [...new Set(lan.flatMap((x) => Object.keys(x.dem ?? {})))].sort();
    const cs = [...new Set(lan.flatMap((x) => Object.keys(x.chi_so ?? {})))];
    L.push(`| Lần | Thời gian (s) | ${keys.join(' | ')}${cs.length ? ' | ' + cs.join(' | ') : ''} |`);
    L.push(`|---:|---:|${keys.map(() => '---:').join('|')}|${cs.map(() => '---:|').join('')}`);
    for (const x of lan) {
      if (x.loi) { L.push(`| ${x.lan} | LỖI: ${x.loi.replace(/\|/g, '/')} |`); continue; }
      L.push(`| ${x.lan} | ${fmt(x.giay, 2)} | ${keys.map((k) => fmt(x.dem[k] ?? 0)).join(' | ')}${cs.length ? ' | ' + cs.map((c) => x.chi_so[c]).join(' | ') : ''} |`);
    }
    const ok = lan.filter((x) => !x.loi);
    if (ok.length > 1) {
      const mm = (f) => { const a = ok.map(f); return `${fmt(Math.min(...a))}–${fmt(Math.max(...a))}`; };
      L.push(`| min–max | ${fmt(Math.min(...ok.map((x) => x.giay)), 2)}–${fmt(Math.max(...ok.map((x) => x.giay)), 2)} | ${keys.map((k) => mm((x) => x.dem[k] ?? 0)).join(' | ')}${cs.length ? ' | ' + cs.map(() => '').join(' | ') : ''} |`);
    }
    L.push('');
    L.push('| Bất biến | Kiểm | Giá trị qua các lần | Kết luận |');
    L.push('|---|---|---|---|');
    for (const b of kb.bat_bien) {
      const vs = ok.map((x) => x.bat_bien.find((y) => y.ten === b.ten));
      const mong = vs[0]?.mong_doi;
      const dat = vs.every((v) => v.dat);
      const kl = b.phai_dung ? (dat ? 'ĐẠT mọi lần' : 'VI PHẠM') : (mong === null || mong === undefined ? 'chỉ ghi nhận' : (dat ? 'không vi phạm lần nào (cách này KHÔNG bảo đảm)' : 'VI PHẠM — minh hoạ cách làm sai'));
      L.push(`| ${b.ten} | ${b.phai_dung ? `phải = ${mong}` : (mong !== null && mong !== undefined ? `minh hoạ, đúng là = ${mong}` : 'ghi nhận')} | ${vs.map((v) => v.gia_tri).join(' · ')} | ${kl} |`);
    }
    if (kb.ghi_chu) { L.push(''); L.push(kb.ghi_chu.trim()); }
  }
  L.push('');
  L.push('## 3. Giới hạn của các số đếm');
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
