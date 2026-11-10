// Lệnh ĐO cho bài hiệu năng (KHÔNG nằm trong `npm test`).
//
// Một tệp đặc tả (vd. p03/do-dac.mjs) khai báo các NHÓM đo. Mỗi nhóm chạy trên một
// database mới chép từ nền (như tệp bài), làm `chuan_bi` một lần, rồi với từng phép đo:
//   thiet_lap (một lần) → lặp [truoc_moi_lan → ĐO `do` → sau_moi_lan] (khoi_dong + lap lần)
//   → go_bo. Các lần khởi động (khoi_dong) bị bỏ, không tính.
// Cách đo:
//   'explain' — EXPLAIN (ANALYZE, TIMING OFF, BUFFERS, FORMAT JSON): lấy Execution Time
//               (thời gian thực thi phía server, không gồm gửi kết quả về client) + số buffer.
//               TIMING OFF để giảm phụ phí đo từng nút.
//   'client'  — \timing của psql: thời gian từ lúc gửi tới lúc nhận xong (gồm commit,
//               mạng localhost, định dạng kết quả). Dùng cho câu GHI.
// Nhóm `pgbench`: chạy pgbench với script tuỳ chỉnh ở nhiều mức phiên đồng thời và
// work_mem; ghi tps, độ trễ, file tạm, đỉnh bộ nhớ riêng của các backend (Windows).
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { exe } from './bin.mjs';
import { PsqlSession } from './session.mjs';
import { splitPsql } from './sqlsplit.mjs';
import { transformStderr } from './output.mjs';
import { privateMemoryPeak } from './machine.mjs';

export function stats(xs) {
  const a = [...xs].sort((x, y) => x - y);
  const n = a.length;
  const med = n === 0 ? NaN : n % 2 ? a[(n - 1) / 2] : (a[n / 2 - 1] + a[n / 2]) / 2;
  return { n, trung_vi: med, min: a[0], max: a[n - 1] };
}

/** Chạy một khối SQL nhiều bước trên phiên; ném lỗi nếu bước nào lỗi. */
export async function runSql(sess, sql, where) {
  for (const st of splitPsql(sql)) {
    const r = await sess.run(st.text);
    const e = transformStderr(r.stderr).error;
    if (e || (st.kind !== 'meta' && r.error)) {
      throw new Error(`${where}: lỗi ở «${st.text.slice(0, 80)}…»: ${e?.message ?? r.stderr.trim()}`);
    }
  }
}

function planSignature(node, out = []) {
  let s = node['Node Type'];
  if (node['Index Name']) s += ` (${node['Index Name']})`;
  else if (node['Relation Name']) s += ` (${node['Relation Name']})`;
  if (/Scan|Join|Nested Loop|Sort|Aggregate|Limit|Hash/.test(node['Node Type'])) out.push(s);
  for (const c of node.Plans ?? []) planSignature(c, out);
  return out;
}

function sortInfo(node, out = []) {
  if (node['Sort Method']) out.push(`${node['Sort Method']} ${node['Sort Space Used']}kB ${node['Sort Space Type']}`);
  if (node['Hash Batches'] !== undefined) out.push(`Hash Batches ${node['Hash Batches']}`);
  if (node['HashAgg Batches'] !== undefined) out.push(`HashAgg Batches ${node['HashAgg Batches']}`);
  for (const c of node.Plans ?? []) sortInfo(c, out);
  return out;
}

export async function measureOne(sess, m, defaults) {
  const lap = m.lap ?? defaults.lap;
  const khoi = m.khoi_dong ?? defaults.khoi_dong;
  const cach = m.cach_do ?? 'explain';
  const where = m.id;
  if (m.thiet_lap) await runSql(sess, m.thiet_lap, where);
  const ms = [];
  const bufs = [];
  const plans = new Set();
  const extras = new Set();
  for (let i = 0; i < khoi + lap; i++) {
    if (m.truoc_moi_lan) await runSql(sess, m.truoc_moi_lan, where);
    let t;
    if (cach === 'explain') {
      const r = await sess.run(`EXPLAIN (ANALYZE, TIMING OFF, BUFFERS, FORMAT JSON) ${m.do.replace(/;\s*$/, '')};`);
      const e = transformStderr(r.stderr).error;
      if (e) throw new Error(`${where}: ${e.message}`);
      const j = JSON.parse(r.stdout.trim());
      t = j[0]['Execution Time'];
      const p = j[0].Plan;
      if (i >= khoi) {
        bufs.push((p['Shared Hit Blocks'] ?? 0) + (p['Shared Read Blocks'] ?? 0));
        plans.add(planSignature(p).join(' → '));
        for (const x of sortInfo(p)) extras.add(x);
      }
    } else {
      await sess.run('\\timing on');
      const r = await sess.run(m.do);
      await sess.run('\\timing off');
      const e = transformStderr(r.stderr).error;
      if (e) throw new Error(`${where}: ${e.message}`);
      const mm = /Time: ([\d.,]+) ms/.exec(r.stdout);
      if (!mm) throw new Error(`${where}: không đọc được \\timing trong «${r.stdout.slice(-200)}»`);
      t = Number(mm[1].replace(',', '.'));
    }
    if (m.sau_moi_lan) await runSql(sess, m.sau_moi_lan, where);
    if (i >= khoi) ms.push(t);
  }
  if (m.go_bo) await runSql(sess, m.go_bo, where);
  return {
    id: m.id, bai: m.bai, ten: m.ten, cach_do: cach, cache: m.cache ?? 'nóng',
    ...stats(ms), cac_lan: ms, buffers: bufs.length ? stats(bufs).trung_vi : null,
    ke_hoach: [...plans], ghi_chu_sort: [...extras],
  };
}

/** Chạy pgbench một lần; trả tps, độ trễ trung bình, đỉnh bộ nhớ riêng của backend pgbench. */
export async function runPgbench(cluster, db, obs, cfg, scriptPath) {
  const args = ['-n', '-f', scriptPath, '-c', String(cfg.clients), '-j', String(Math.min(cfg.clients, 4)),
    '-T', String(cfg.giay), '-h', cluster.host, '-p', String(cluster.port), '-U', 'postgres', db];
  const env = { ...cluster.env, PGOPTIONS: cfg.pgoptions ?? '', PGAPPNAME: 'pgbench' };
  const p = spawn(exe(cluster.pg.dir, 'pgbench'), args, { env, windowsHide: true });
  let out = '';
  let err = '';
  p.stdout.on('data', (d) => { out += d; });
  p.stderr.on('data', (d) => { err += d; });
  const done = new Promise((res) => p.on('exit', (code) => res(code)));
  // Chờ các kết nối pgbench lên rồi lấy PID backend của chúng.
  await new Promise((r) => setTimeout(r, 1500));
  const pr = await obs.run(`SELECT string_agg(pid::text, ',') FROM pg_stat_activity WHERE application_name = 'pgbench';`);
  const pids = pr.stdout.trim() ? pr.stdout.trim().split(',').map(Number) : [];
  const mem = await privateMemoryPeak(pids, Math.max(1, cfg.giay - 3));
  const code = await done;
  if (code !== 0) throw new Error(`pgbench thoát mã ${code}: ${err.slice(-500)}`);
  const tps = /tps = ([\d.]+) \(without initial connection time\)/.exec(out);
  const lat = /latency average = ([\d.]+) ms/.exec(out);
  const tx = /number of transactions actually processed: (\d+)/.exec(out);
  return {
    tps: tps ? Number(tps[1]) : null,
    latency_ms: lat ? Number(lat[1]) : null,
    giao_dich: tx ? Number(tx[1]) : null,
    so_backend: pids.length,
    dinh_bo_nho_rieng_mb: mem ? Math.round(mem.dinh / 1048576) : null,
    so_mau_bo_nho: mem?.mau ?? 0,
  };
}

export async function tempStats(sess) {
  await sess.run('SELECT pg_stat_force_next_flush();');
  await sess.run('SELECT pg_stat_clear_snapshot();');
  const r = await sess.run(`SELECT temp_files || ' ' || temp_bytes FROM pg_stat_database WHERE datname = current_database();`);
  const [f, b] = r.stdout.trim().split(' ').map(Number);
  return { files: f, bytes: b };
}

export function writeScript(cluster, name, text) {
  const p = join(cluster.root, name);
  writeFileSync(p, text);
  return resolve(p);
}
