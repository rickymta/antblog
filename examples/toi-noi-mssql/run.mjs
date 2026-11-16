#!/usr/bin/env node
// Runner của lab "SQL Server qua dự án Tới Nơi".
//
//   node run.mjs p01                       # mọi p01/NN-*.sql theo thứ tự tên
//   node run.mjs p01/00-reset.sql p01/01-schema.sql p02
//   LAB_DB=ToiNoi_thu node run.mjs p01     # chạy trên database khác (mặc định ToiNoi)
//
// Mỗi file chạy bằng một lần gọi sqlcmd BÊN TRONG container `db` (không cần cài
// công cụ nào trên máy): `docker compose exec -T db sqlcmd … -i /lab/<file>`.
// Mỗi file là một phiên (kết nối) riêng: SET, biến, bảng tạm không sang file sau.
//
// sqlcmd chạy với -b: lỗi KHÔNG được bắt trong file sẽ dừng file đó và runner
// báo FAIL. Lỗi mong đợi được bắt bằng TRY…CATCH qua lab.expect_error.
// Kết quả: in lại output của sqlcmd, cuối cùng là bảng đếm PASS/FAIL.

import { spawnSync } from 'node:child_process';
import { readdirSync, statSync } from 'node:fs';
import { dirname, join, posix, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const SA_PASSWORD = process.env.MSSQL_SA_PASSWORD ?? 'ToiNoi_lab_2022!';
// Tên database của lab; đổi bằng LAB_DB=… để chạy song song trên database riêng.
const LAB_DB = process.env.LAB_DB ?? 'ToiNoi';
if (!/^[A-Za-z][A-Za-z0-9_]{0,63}$/.test(LAB_DB)) throw new Error('LAB_DB không hợp lệ: ' + LAB_DB);
const SQLCMD = '/opt/mssql-tools18/bin/sqlcmd';

function expand(arg) {
  const full = resolve(root, arg);
  if (statSync(full).isDirectory()) {
    return readdirSync(full)
      .filter((f) => /^\d\d-.*\.sql$/.test(f) && !f.endsWith('-rollback.sql'))
      .sort()
      .map((f) => join(full, f));
  }
  return [full];
}

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Cách dùng: node run.mjs <thư mục phần | file.sql> …');
  process.exit(2);
}
const files = args.flatMap(expand);

let pass = 0;
let fail = 0;
let fileErrors = 0;

for (const file of files) {
  const rel = relative(root, file).split('\\').join('/');
  const inContainer = posix.join('/lab', rel);
  console.log(`\n-- ==================== ${rel} ====================`);
  const r = spawnSync(
    'docker',
    [
      'compose', 'exec', '-T', 'db', SQLCMD,
      '-C', '-S', 'localhost', '-U', 'sa', '-P', SA_PASSWORD,
      '-b', '-I', '-W', '-s', '|', '-f', 'i:65001,o:65001',
      '-v', `LAB_DB=${LAB_DB}`,
      '-i', inContainer,
    ],
    { cwd: root, encoding: 'utf8', env: { ...process.env, MSYS_NO_PATHCONV: '1' } },
  );
  if (r.error) throw r.error;
  const out = (r.stdout ?? '') + (r.stderr ?? '');
  const text = out.replace(/\r\n/g, '\n').replace(/\n+$/, '');
  if (text) console.log(text);
  for (const line of text.split('\n')) {
    if (line.startsWith('PASS ')) pass++;
    else if (line.startsWith('FAIL ')) fail++;
  }
  if (r.status !== 0) {
    fileErrors++;
    console.log(`-- [lab] FAIL: ${rel} dừng với mã thoát ${r.status} (lỗi không được bắt)`);
  }
}

console.log('\n-- [lab] File đã chạy: ' + files.length);
console.log('-- [lab] PASS: ' + pass);
console.log('-- [lab] FAIL: ' + (fail + fileErrors));
console.log('-- [lab] KẾT QUẢ: ' + (fail + fileErrors === 0 ? 'ĐẠT' : 'KHÔNG ĐẠT'));
process.exitCode = fail + fileErrors === 0 ? 0 : 1;
