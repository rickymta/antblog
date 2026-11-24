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
//   #sudofile sudoedit|visudo ĐƯỜNG NGUỒN [NN]
//                     như #file cho file của root: in "sudoedit ĐƯỜNG" hoặc "sudo visudo -f ĐƯỜNG",
//                     ghi bằng quyền root (visudo: quyền 0440)
//   #login NGƯỜI KEY [otp]
//                     mô phỏng một lần đăng nhập SSH từ phía client (root chạy ssh tới 127.0.0.1
//                     bằng key /usr/share/toinoi-lab/p03/KEY, kèm mã OTP nếu có), không in
//   #ngat             mô phỏng mất kết nối SSH rồi đăng nhập lại (Phần 4): như bash khi nhận SIGHUP,
//                     gửi SIGHUP tới mọi job của shell, rồi coi như phiên mới (bảng job rỗng, về ~)

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
  // LNX-047 gắn /srv/dia/du-lieu-01.img thành loop device. Loop device thuộc kernel của máy chạy
  // Docker, không mất theo container: gỡ đúng thiết bị của lab trước khi xoá container.
  docker(['exec', CONTAINER, 'sh', '-c',
    'umount /du-lieu 2>/dev/null; for d in $(losetup -j /srv/dia/du-lieu-01.img -n -O NAME 2>/dev/null); do losetup -d "$d"; done; true']);
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
  if (/(^|[|;&(]\s*)(sudo|sudoedit)\b|^#(root|sudofile|login) /m.test(source)) out.push(`echo ${quote(PASSWORD)} | sudo -S -v 2>/dev/null`);
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
    if (line === '#ngat') {
      out.push('for __j in $(jobs -p); do kill -HUP "$__j" 2>/dev/null; done; disown -a; cd ~; sleep 1');
      continue;
    }
    if (line.startsWith('#wait ')) {
      out.push(`sleep ${Number(line.slice(6))}`);
      continue;
    }
    const sf = line.match(/^#sudofile (sudoedit|visudo) (\S+) (\S+)(?: \d\d)?$/);
    if (sf) {
      const body = readFileSync(join(dir, 'files', sf[3])).toString('base64').replace(/.{76}/g, '$&\n');
      if (echo) out.push(`__p ${quote(sf[1] === 'visudo' ? `sudo visudo -f ${sf[2]}` : `sudoedit ${sf[2]}`)}`);
      // Như visudo/sudoedit thật: file mới do `visudo -f` tạo có quyền 0640 (sudo đòi 0440, bài tự chmod),
      // file mới do sudoedit tạo là 0644; file đã có thì giữ nguyên quyền.
      const mode = sf[1] === 'visudo' ? '0640' : '0644';
      out.push(`__new=0; sudo -n test -e ${sf[2]} || __new=1`);
      out.push(`base64 -d <<'__B64__' | sudo -n tee ${sf[2]} >/dev/null\n${body.trimEnd()}\n__B64__`);
      out.push(`[ "$__new" = 1 ] && sudo -n chmod ${mode} ${sf[2]}`);
      continue;
    }
    const lg = line.match(/^#login (\S+) (\S+)( otp)?$/);
    if (lg) {
      out.push(`sudo -n bash -c ${quote(loginCommand(lg[1], lg[2], !!lg[3]))} >/dev/null 2>&1`);
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

/** Tên container của một server trong lab: app01 → toinoi-app-01. */
export const containerOf = (may) => 'toinoi-' + may.replace(/^app/, 'app-');

/**
 * Tách kịch bản thành các đoạn theo chỉ dẫn `#may appNN` (Phần 5): mỗi đoạn chạy trên một server,
 * trong một phiên đăng nhập riêng của `an`. Kịch bản không có chỉ dẫn này là một đoạn trên app01.
 */
export function segments(source) {
  const out = [{ may: 'app01', lines: [] }];
  for (const line of source.replace(/\r\n/g, '\n').split('\n')) {
    const m = line.match(/^#may (app\d\d)$/);
    if (m) out.push({ may: m[1], lines: [] });
    else out[out.length - 1].lines.push(line);
  }
  return out.filter((seg) => seg.lines.some((l) => l.trim() && !/^# /.test(l)));
}

/** Chạy một kịch bản; trả về bản ghi (transcript) dạng chữ. */
export function runScript(path, { echo = true } = {}) {
  return segments(readFileSync(path, 'utf8'))
    .map((seg) => runSegment(seg.may, seg.lines.join('\n'), { echo, dir: dirname(path) }))
    .join('');
}

export function runSegment(may, source, { echo, dir }) {
  const container = containerOf(may);
  const script = compile(source, { echo, dir });
  const put = docker(['exec', '-i', container, 'bash', '-c', 'cat > /run/lab.sh && chmod 644 /run/lab.sh'], {
    input: script,
  });
  if (put.status !== 0) throw new Error(put.stderr);
  // -t: có TTY, nên `ls` chia cột và sudo nhớ lần xác thực như trong phiên SSH.
  const r = docker(['exec', '-t', container, 'su', '-', USER, '-c', 'bash /run/lab.sh']);
  return (r.stdout + r.stderr)
    .replace(/\r\n/g, '\n')
    .replace(/\r/g, '')
    // systemctl in đường dẫn dạng liên kết OSC 8 (ESC ] 8 ;; URL ESC \); giữ chữ, bỏ mã điều khiển.
    .replace(/\x1b\]8;[^\x07\x1b]*(\x07|\x1b\\)/g, '')
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

/**
 * Lệnh (chạy bằng root trong container) mô phỏng một client đăng nhập SSH vào chính server:
 * dùng key riêng tư của lab và, nếu cần, mã OTP do oathtool tính từ bí mật của lab.
 */
export function loginCommand(user, key, otp = false) {
  const opts = [
    `-i /usr/share/toinoi-lab/p03/${key}`,
    '-o IdentitiesOnly=yes',
    '-o StrictHostKeyChecking=no',
    '-o UserKnownHostsFile=/dev/null',
    '-o LogLevel=ERROR',
    '-o ConnectTimeout=5',
  ];
  const env = otp
    ? 'SSH_ASKPASS=/usr/share/toinoi-lab/p03/askpass-otp SSH_ASKPASS_REQUIRE=force DISPLAY=lab '
    : '';
  if (!otp) opts.push('-o BatchMode=yes');
  return `${env}setsid -w ssh ${opts.join(' ')} ${user}@127.0.0.1 true < /dev/null`;
}
