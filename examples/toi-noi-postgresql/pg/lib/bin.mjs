// Tìm thư mục chứa binary PostgreSQL (initdb, pg_ctl, psql, pgbench…).
//
// Thứ tự ưu tiên:
//   1. Biến môi trường PG_BIN. Nếu đặt mà thư mục không có psql/initdb thì
//      COI NHƯ KHÔNG CÓ PostgreSQL (không dò tiếp) — để có thể giả lập máy
//      chỉ có Node: `PG_BIN=/khong-co npm test` ⇒ test P03+ được skip.
//   2. `pg_config --bindir` nếu pg_config nằm trong PATH.
//   3. Đường dẫn cài mặc định trên Windows: C:\Program Files\PostgreSQL\18\bin.
import { existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';

const EXE = process.platform === 'win32' ? '.exe' : '';
export const DEFAULT_WINDOWS_BIN = 'C:\\Program Files\\PostgreSQL\\18\\bin';
/** Phiên bản chính tối thiểu mà lab P03+ đã kiểm (18.1). */
export const MIN_MAJOR = 18;

export function exe(dir, name) {
  return join(dir, name + EXE);
}

function hasTools(dir) {
  return ['psql', 'initdb', 'pg_ctl'].every((t) => existsSync(exe(dir, t)));
}

/**
 * Trả { dir, version, major, source } hoặc { error } (không ném lỗi).
 * `version` lấy từ `psql --version`, ví dụ "18.1".
 */
export function findPgBin(env = process.env) {
  let dir = null;
  let source = null;
  if (env.PG_BIN !== undefined && env.PG_BIN !== '') {
    if (!hasTools(env.PG_BIN)) {
      return { error: `PG_BIN=${env.PG_BIN} không chứa psql/initdb/pg_ctl` };
    }
    dir = env.PG_BIN;
    source = 'PG_BIN';
  } else {
    const r = spawnSync('pg_config' + EXE, ['--bindir'], { encoding: 'utf8' });
    if (r.status === 0 && r.stdout.trim() && hasTools(r.stdout.trim())) {
      dir = r.stdout.trim();
      source = 'pg_config --bindir';
    } else if (process.platform === 'win32' && hasTools(DEFAULT_WINDOWS_BIN)) {
      dir = DEFAULT_WINDOWS_BIN;
      source = 'đường dẫn mặc định';
    }
  }
  if (!dir) {
    return { error: 'Không tìm thấy PostgreSQL: đặt PG_BIN, thêm pg_config vào PATH, hoặc cài PostgreSQL 18 vào ' + DEFAULT_WINDOWS_BIN };
  }
  const v = spawnSync(exe(dir, 'psql'), ['--version'], { encoding: 'utf8' });
  const m = /(\d+)(?:\.(\d+))?/.exec(v.stdout ?? '');
  if (v.status !== 0 || !m) return { error: `Không chạy được ${exe(dir, 'psql')} --version` };
  const major = Number(m[1]);
  if (major < MIN_MAJOR) {
    return { error: `Cần PostgreSQL ${MIN_MAJOR} trở lên, tìm thấy ${m[0]} ở ${dir}` };
  }
  return { dir, version: m[2] !== undefined ? `${m[1]}.${m[2]}` : m[1], major, source };
}
