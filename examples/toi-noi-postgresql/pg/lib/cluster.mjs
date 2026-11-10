// Cluster PostgreSQL TẠM cho lab: initdb vào os.tmpdir()/toinoi-pg-<ngẫu nhiên>,
// chạy ở cổng trống ngẫu nhiên chỉ trên localhost, và LUÔN dừng + xoá khi xong
// (kể cả khi lỗi hoặc Ctrl-C), trừ khi gọi với keep = true (--keep).
//
// Không bao giờ ghi ra ngoài thư mục tạm đó. Không đụng service PostgreSQL
// của máy (vd. postgresql-x64-18 ở cổng 5432): cluster tạm có data directory,
// cổng và tiến trình riêng.
import { mkdtempSync, rmSync, appendFileSync, existsSync, openSync, closeSync, readFileSync, statSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { exe } from './bin.mjs';

/**
 * Cấu hình ghi đè TƯỜNG MINH cho tính tái lập. Nhiều dòng trùng giá trị mặc định
 * của PostgreSQL 18 nhưng vẫn ghi ra để người đọc biết lab chạy với gì.
 * Bảng giải thích từng dòng: README.md gốc của lab, mục «Cấu hình cluster tạm».
 */
export const LAB_SETTINGS = {
  listen_addresses: 'localhost',
  max_connections: 40,
  shared_buffers: '128MB',
  work_mem: '4MB',
  hash_mem_multiplier: 2.0,
  maintenance_work_mem: '64MB',
  effective_cache_size: '4GB',
  random_page_cost: 4.0,
  seq_page_cost: 1.0,
  effective_io_concurrency: 16,
  jit: 'off',
  max_parallel_workers_per_gather: 0,
  max_parallel_maintenance_workers: 0,
  default_statistics_target: 100,
  autovacuum: 'off',
  track_io_timing: 'off',
  max_wal_size: '4GB',
  checkpoint_timeout: '30min',
  log_checkpoints: 'off',
  log_line_prefix: "'%m [%p] '",
  log_temp_files: -1,
  shared_preload_libraries: "'pg_stat_statements'",
  timezone: "'Asia/Ho_Chi_Minh'",
  log_timezone: "'Asia/Ho_Chi_Minh'",
  datestyle: "'iso, mdy'",
  lc_messages: "'C'",
};

/** LC_CTYPE của cluster — xem lý do ở README (pg_trgm phân loại ký tự theo LC_CTYPE). */
export function labCtype(env = process.env) {
  if (env.PG_LAB_LC_CTYPE) return env.PG_LAB_LC_CTYPE;
  return process.platform === 'win32' ? 'en-US' : 'C.UTF-8';
}

function q(v) {
  if (typeof v === 'number') return String(v);
  const s = String(v);
  if (s.startsWith("'")) return s;
  return /^[A-Za-z0-9_.]+$/.test(s) ? s : `'${s}'`;
}

/** Cổng TCP trống trên localhost (hệ điều hành cấp). */
export function freePort() {
  return new Promise((resolve, reject) => {
    const srv = createServer();
    srv.unref();
    srv.on('error', reject);
    srv.listen(0, '127.0.0.1', () => {
      const { port } = srv.address();
      srv.close(() => resolve(port));
    });
  });
}

// ---------------------------------------------------------------------------
// Dọn dẹp khi tiến trình kết thúc bất thường. Mọi thao tác ở đây ĐỒNG BỘ vì
// sự kiện 'exit' không chờ được Promise.
const live = new Set();
let handlersInstalled = false;
function installHandlers() {
  if (handlersInstalled) return;
  handlersInstalled = true;
  const cleanupAll = () => {
    for (const c of live) {
      try { c.destroySync({ mode: 'immediate' }); } catch { /* đã cố hết sức */ }
    }
  };
  process.on('exit', cleanupAll);
  for (const sig of ['SIGINT', 'SIGTERM', 'SIGHUP', 'SIGBREAK']) {
    try {
      process.on(sig, () => {
        process.stderr.write(`\n-- [lab] Nhận ${sig}: dừng và xoá cluster tạm…\n`);
        cleanupAll();
        process.exit(130);
      });
    } catch { /* tín hiệu không có trên hệ điều hành này */ }
  }
  process.on('uncaughtException', (e) => {
    process.stderr.write(`-- [lab] Lỗi không bắt được: ${e?.stack ?? e}\n`);
    cleanupAll();
    process.exit(2);
  });
}

function run(cmd, args, { logFile, allowFail = false, env } = {}) {
  // stdin/stdout/stderr KHÔNG để dạng ống (pipe): trên Windows postmaster do
  // pg_ctl khởi động kế thừa handle của ống và giữ nó mở ⇒ spawnSync treo.
  // Ghi thẳng vào tệp nhật ký.
  const fd = openSync(logFile, 'a');
  try {
    const r = spawnSync(cmd, args, { stdio: ['ignore', fd, fd], env, windowsHide: true });
    if (r.error) throw r.error;
    if (r.status !== 0 && !allowFail) {
      throw new Error(`${cmd} ${args.join(' ')} thoát mã ${r.status} — xem ${logFile}`);
    }
    return r.status;
  } finally {
    closeSync(fd);
  }
}

export function childEnv(extra = {}) {
  const env = { ...process.env, PGCLIENTENCODING: 'UTF8', LC_ALL: 'C', LC_MESSAGES: 'C', LANG: 'C', ...extra };
  // Không để biến môi trường của người dùng kéo kết nối sang server khác.
  for (const k of ['PGHOST', 'PGHOSTADDR', 'PGPORT', 'PGDATABASE', 'PGUSER', 'PGPASSWORD', 'PGPASSFILE', 'PGSERVICE', 'PGSERVICEFILE', 'PGOPTIONS', 'PGSSLMODE', 'PGTZ', 'PGDATESTYLE']) delete env[k];
  return env;
}

/**
 * Tạo và khởi động cluster tạm. `settings` ghi đè thêm lên LAB_SETTINGS.
 * Trả đối tượng { dataDir, port, host, logFile, root, … }.
 */
export async function createCluster(pg, { keep = false, settings = {}, say = () => {} } = {}) {
  installHandlers();
  const root = mkdtempSync(join(tmpdir(), 'toinoi-pg-'));
  const port = await freePort();
  const cluster = clusterObject(pg, root, port, keep, settings);
  live.add(cluster);
  say(`-- [lab] initdb vào ${root} …`);
  run(exe(pg.dir, 'initdb'), [
    '-D', cluster.dataDir, '-U', 'postgres', '--auth=trust', '-E', 'UTF8', '--no-sync',
    '--locale=C', `--lc-ctype=${cluster.ctype}`,
    '--locale-provider=builtin', '--builtin-locale=C.UTF-8',
  ], { logFile: cluster.toolLog, env: cluster.env });
  const lines = ['', '# --- Cấu hình lab Tới Nơi (pg/lib/cluster.mjs) ---'];
  for (const [k, v] of Object.entries(cluster.settings)) lines.push(`${k} = ${q(v)}`);
  appendFileSync(join(cluster.dataDir, 'postgresql.conf'), lines.join('\n') + '\n');
  cluster.start();
  return cluster;
}

/**
 * GỠ LỖI: gắn vào một cluster đã giữ lại bằng --keep (đang chạy). Không bao giờ
 * dừng hay xoá cluster này. Chỉ nhận thư mục tên toinoi-pg-* trong thư mục tạm.
 */
export function attachCluster(pg, root) {
  if (!/toinoi-pg-[^\\/]+[\\/]?$/.test(root) || !existsSync(join(root, 'data', 'postgresql.conf'))) {
    throw new Error(`--attach cần thư mục cluster toinoi-pg-* do --keep để lại, nhận: ${root}`);
  }
  const conf = readFileSync(join(root, 'data', 'postgresql.conf'), 'utf8');
  const ports = [...conf.matchAll(/^port = (\d+)/gm)];
  if (!ports.length) throw new Error('Không đọc được cổng trong postgresql.conf');
  const cluster = clusterObject(pg, root, Number(ports.at(-1)[1]), true, {});
  cluster.running = true;
  cluster.attached = true;
  return cluster;
}

function clusterObject(pg, root, port, keep, settings) {
  const dataDir = join(root, 'data');
  const toolLog = join(root, 'cong-cu.log');
  const logFile = join(root, 'server.log');
  const env = childEnv();
  const cluster = {
    pg, root, dataDir, logFile, toolLog, port, host: 'localhost', keep, env,
    ctype: labCtype(),
    settings: { ...LAB_SETTINGS, port, ...settings },
    running: false,
    destroyed: false,
  };

  cluster.stop = (mode = 'fast') => {
    if (!cluster.running) return;
    run(exe(pg.dir, 'pg_ctl'), ['stop', '-D', dataDir, '-m', mode, '-w', '-t', '60'], { logFile: toolLog, allowFail: true, env });
    cluster.running = false;
  };
  cluster.start = () => {
    run(exe(pg.dir, 'pg_ctl'), ['start', '-D', dataDir, '-l', logFile, '-w', '-t', '120'], { logFile: toolLog, env });
    cluster.running = true;
  };
  cluster.restart = () => {
    cluster.stop('fast');
    cluster.start();
  };
  cluster.destroySync = ({ mode = 'fast' } = {}) => {
    if (cluster.destroyed) return;
    if (cluster.keep) {
      live.delete(cluster);
      return;
    }
    try { cluster.stop(mode); } catch { /* tiếp tục xoá */ }
    // Windows: tiến trình con của postgres có thể còn giữ thư mục data vài giây sau khi
    // pg_ctl stop báo xong ⇒ thử xoá lại tối đa ~60 giây rồi mới bỏ cuộc (không ném lỗi).
    let lastErr = null;
    for (let k = 0; k < 60; k++) {
      try {
        rmSync(root, { recursive: true, force: true, maxRetries: 4, retryDelay: 250 });
        lastErr = null;
        break;
      } catch (e) {
        lastErr = e;
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 1000);
      }
    }
    if (lastErr) process.stderr.write(`-- [lab] CẢNH BÁO: không xoá được ${root} (${lastErr.code}); xoá tay sau.\n`);
    cluster.destroyed = true;
    live.delete(cluster);
  };
  /** Kích thước tệp nhật ký server hiện tại (để đọc phần mới ghi thêm). */
  cluster.logSize = () => (existsSync(logFile) ? statSync(logFile).size : 0);
  cluster.readLogFrom = (offset) => {
    if (!existsSync(logFile)) return '';
    const buf = readFileSync(logFile);
    return buf.subarray(offset).toString('utf8');
  };
  return cluster;
}
