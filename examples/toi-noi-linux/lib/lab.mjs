// Thư viện của lab Linux: điều khiển container bằng docker và chạy "kịch bản lệnh" của từng bài.
//
// Một kịch bản là file .sh thường. Runner chạy nó trong MỘT phiên đăng nhập của người dùng `an`
// (qua `su - an`, có TTY rộng 100 cột) và in lại từng lệnh kèm dấu nhắc như trên terminal thật:
//
//   an@toinoi-app-01:~$ ls -l huong-dan.txt
//   -rw-r--r-- 1 an an 330 Nov 20 08:30 huong-dan.txt
//
// Quy ước trong file kịch bản:
//   ## Tiêu đề        in thành một dòng "# Tiêu đề" trong kết quả
//   # ghi chú         bỏ qua
//   #@ lệnh           chạy nhưng không in (chuẩn bị trạng thái, thay cho thao tác trên giao diện)
//   lệnh \            dòng kết thúc bằng \ nối với dòng sau
//   lệnh <<'EOF' …    heredoc được giữ nguyên tới dòng EOF
//   #root lệnh        chạy với quyền root, không in (dữ liệu của lab, ví dụ log mới)
//   #file ĐƯỜNG NGUỒN in "vim ĐƯỜNG" như một lệnh rồi ghi nội dung của files/NGUỒN vào ĐƯỜNG:
//                     thay cho việc gõ file đó bằng vim (công cụ chụp ảnh gõ thật)
//   #check ĐƯỜNG NGUỒN ghi files/NGUỒN vào ĐƯỜNG mà không in: kết quả của thao tác sửa bằng vim
//                     hoặc giao diện ngay trước đó (công cụ chụp ảnh đối chiếu nội dung)
//   #wait N           chờ N giây (ví dụ chờ cron chạy)

import { spawnSync } from 'node:child_process';
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
export const CONTAINER = 'toinoi-app-01';
export const USER = 'an';
/** Mật khẩu của lab. Chỉ dùng cho container này; cổng SSH chỉ mở ở 127.0.0.1. */
export const PASSWORD = 'toinoi-lab';

// Git Bash trên Windows tự đổi "/home/an" thành đường dẫn Windows nếu không tắt.
const env = { ...process.env, MSYS_NO_PATHCONV: '1', MSYS2_ARG_CONV_EXCL: '*' };

export function docker(args, opts = {}) {
  const r = spawnSync('docker', args, { encoding: 'utf8', env, cwd: ROOT, maxBuffer: 64 << 20, ...opts });
  if (r.error) throw r.error;
  return r;
}

export function compose(args, opts = {}) {
  return docker(['compose', ...args], { stdio: 'inherit', ...opts });
}

/** Chạy một lệnh shell trong container với quyền root, trả về stdout (đã bỏ khoảng trắng cuối). */
export function root(cmd) {
  const r = docker(['exec', CONTAINER, 'bash', '-c', cmd]);
  if (r.status !== 0) throw new Error(`root: ${cmd}\n${r.stderr || r.stdout}`);
  return r.stdout.trimEnd();
}

/** Chạy một lệnh shell với tư cách người dùng của lab (phiên đăng nhập mới, không TTY). */
export function asUser(cmd, user = USER) {
  const r = docker(['exec', CONTAINER, 'su', '-', user, '-c', cmd]);
  return { status: r.status, stdout: r.stdout.trimEnd(), stderr: r.stderr.trimEnd() };
}

export function up() {
  compose(['up', '-d', '--build', '--wait']);
}

export function down() {
  compose(['down', '--volumes', '--remove-orphans']);
}

/** Tạo lại container từ ảnh: mọi thay đổi trên server của lab bị xoá. */
export function reset() {
  compose(['down', '--volumes', '--remove-orphans'], { stdio: 'ignore' });
  compose(['up', '-d', '--build', '--wait'], { stdio: ['ignore', 'ignore', 'inherit'] });
}

export function scripts(part) {
  const dir = join(ROOT, part);
  return readdirSync(dir)
    .filter((f) => /^\d\d-.*\.sh$/.test(f))
    .sort()
    .map((f) => ({ id: f.slice(0, 2), name: f, path: join(dir, f) }));
}

const quote = (s) => `'${s.replace(/'/g, `'\\''`)}'`;

/** Đổi file kịch bản thành script bash in lại từng lệnh kèm dấu nhắc. */
export function compile(source, { echo = true, dir = '.' } = {}) {
  const out = [
    'stty cols 100 rows 40 2>/dev/null',
    // Shell tương tác của Ubuntu đặt sẵn hai alias này trong ~/.bashrc. Script không tương tác
    // thì không có, nên runner bật lại để kết quả giống hệt khi bạn tự gõ (màu bị lọc khỏi bản ghi).
    'shopt -s expand_aliases',
    // ~/.bashrc thoát ngay khi shell không tương tác (khối `case $- in`). Runner nạp phần còn lại
    // để có cùng alias (ls, grep, ll…) và cùng những dòng bạn tự thêm, như một phiên SSH thật.
    // Lệnh `source ~/.bashrc` trong kịch bản cũng được chạy theo cách này.
    "__rc() { source <(sed '/^case \\$- in$/,/^esac$/d' ~/.bashrc); }",
    '__rc',
  ];
  // Trong phiên SSH thật, sudo hỏi mật khẩu ở lần gọi đầu tiên. Runner xác thực sẵn (chỉ ở
  // kịch bản có dùng sudo) nên bản ghi không có dòng "[sudo] password for an:".
  if (/^\s*sudo\b|^#root /m.test(source)) out.push(`echo ${quote(PASSWORD)} | sudo -S -v 2>/dev/null`);
  out.push(
    // Dấu nhắc giống PS1 mặc định của Ubuntu: user@host:thư-mục$. Hàm trả lại mã thoát của
    // lệnh đứng trước, để `echo $?` trong kịch bản vẫn đúng.
    '__p() { local s=$? d="${PWD/#$HOME/\\~}"; printf \'%s@%s:%s$ %s\\n\' "$USER" "$HOSTNAME" "$d" "$1"; shift; for l in "$@"; do printf \'> %s\\n\' "$l"; done; return $s; }',
  );
  const lines = source.replace(/\r\n/g, '\n').split('\n');
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (!line.trim()) continue;
    if (line.startsWith('## ')) {
      if (echo) out.push(`printf '\\n# %s\\n' ${quote(line.slice(3))}`);
      continue;
    }
    if (line.startsWith('#wait ')) {
      out.push(`sleep ${Number(line.slice(6))}`);
      continue;
    }
    if (line.startsWith('#root ')) {
      out.push(`sudo -n bash -c ${quote(line.slice(6))} >/dev/null`);
      continue;
    }
    // Số thứ hai sau #file (nếu có) là ảnh mà công cụ chụp ảnh chụp lúc file còn mở trong vim.
    const file = line.match(/^#(file|check) (\S+) (\S+)(?: \d\d)?$/);
    if (file) {
      const body = readFileSync(join(dir, 'files', file[3])).toString('base64').replace(/.{76}/g, '$&\n');
      if (file[1] === 'file' && echo) out.push(`__p ${quote('vim ' + file[2])}`);
      out.push(`base64 -d > ${file[2]} <<'__B64__'\n${body.trimEnd()}\n__B64__`);
      continue;
    }
    if (line.startsWith('#@ ')) {
      out.push(line.slice(3));
      continue;
    }
    if (line.startsWith('#')) continue;

    const block = [line];
    const heredoc = line.match(/<<-?\s*'?(\w+)'?/);
    if (heredoc) {
      while (i + 1 < lines.length) {
        block.push(lines[++i]);
        if (lines[i] === heredoc[1]) break;
      }
    } else {
      while (block[block.length - 1].endsWith('\\') && i + 1 < lines.length) block.push(lines[++i]);
    }
    if (echo) out.push(`__p ${block.map(quote).join(' ')}`);
    out.push(/^(source|\.) ~\/\.bashrc$/.test(line) ? '__rc' : block.join('\n'));
  }
  return out.join('\n') + '\n';
}

/** Chạy một kịch bản; trả về bản ghi (transcript) dạng chữ. */
export function runScript(path, { echo = true } = {}) {
  const script = compile(readFileSync(path, 'utf8'), { echo, dir: dirname(path) });
  const put = docker(['exec', '-i', CONTAINER, 'bash', '-c', 'cat > /run/lab.sh && chmod 644 /run/lab.sh'], {
    input: script,
  });
  if (put.status !== 0) throw new Error(put.stderr);
  // -t: có TTY, nên `ls` chia cột và sudo nhớ lần xác thực như trong phiên SSH.
  const r = docker(['exec', '-t', CONTAINER, 'su', '-', USER, '-c', 'bash /run/lab.sh']);
  return (r.stdout + r.stderr)
    .replace(/\r\n/g, '\n')
    .replace(/\r/g, '')
    .replace(/\x1b\[[0-9;?]*[A-Za-z]/g, '')
    // Shell tương tác báo lỗi bằng tên "-bash", không phải tên file kịch bản.
    .replace(/^\/run\/lab\.sh: line \d+: /gm, '-bash: ')
    .split('\n')
    .map(expandTabs)
    .join('\n')
    .replace(/[ \t]+$/gm, '');
}

/** `ls` trên TTY căn cột bằng tab; đổi thành dấu cách (mốc 8 cột) để bản ghi không phụ thuộc trình xem. */
function expandTabs(line) {
  let out = '';
  for (const ch of line) out += ch === '\t' ? ' '.repeat(8 - (out.length % 8)) : ch;
  return out;
}
