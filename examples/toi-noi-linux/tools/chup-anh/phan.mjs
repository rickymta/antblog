// Trình phát kịch bản dùng chung cho các phần từ Phần 2: gõ từng lệnh của pNN/MM-*.sh vào
// terminal của SSH Studio, làm thao tác giao diện ở dòng `#gui`, chụp ở dòng `#shot`.
//
// Chỉ dẫn (runner của lab hiểu cùng các chỉ dẫn này, xem lib/lab.mjs):
//   #clear              gõ `clear`
//   #shot NN            chụp ảnh ssh-studio-NN của bài
//   #gui tên            gọi thao tác giao diện cùng tên do từng phần định nghĩa
//   #root lệnh          chạy với quyền root qua `docker exec` (dữ liệu của lab)
//   #file ĐƯỜNG NGUỒN [NN]  mở vim, gõ nội dung files/NGUỒN, (chụp NN), lưu; rồi đối chiếu
//   #check ĐƯỜNG NGUỒN  đối chiếu file trên server với files/NGUỒN (sau một thao tác #gui)
//   #wait N             chờ N giây
//   #@ lệnh             bỏ qua: runner chạy thay cho thao tác giao diện đã làm thật ở đây
//   #sudofile sudoedit|visudo ĐƯỜNG NGUỒN [NN]   như #file nhưng mở bằng sudoedit / sudo visudo -f
//   #login NGƯỜI KEY [otp]   cùng lần đăng nhập mô phỏng như runner (đợi khung OTP mới nếu cần)
//
// Lệnh bắt đầu bằng sudo hoặc sudoedit: nếu sudo dừng lại hỏi mật khẩu (tiến trình sudo chạy
// bằng root mà chưa có tiến trình con), công cụ gõ mật khẩu của lab.

import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, unlinkSync } from 'node:fs';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { PASSWORD, ROOT, loginCommand, reset, root, scripts } from '../../lib/lab.mjs';
import { open } from './app.mjs';

const REPO = join(ROOT, '..', '..');
const sharp = createRequire(join(REPO, 'package.json'))('sharp');
export const LOCAL = join(ROOT, 'may-cua-ban');

/** Thư mục hiện tại của shell trong phiên SSH (shell con trực tiếp của sshd). */
function sessionCwd() {
  return root(
    "for p in $(pgrep -f '^sshd: an@pts'); do for c in $(pgrep -P $p -x bash); do readlink /proc/$c/cwd; done; done | head -n 1",
  );
}

/**
 * Shell của phiên SSH đang rảnh: nhóm tiến trình foreground của terminal chính là shell (không còn
 * lệnh nào chạy ở foreground; job chạy nền không tính). Không thấy shell nào thì coi như rảnh.
 */
export function shellIdle() {
  const busy = root(
    "for p in $(pgrep -f '^sshd: an@pts'); do for c in $(pgrep -P $p -x bash); do " +
      't=$(ps -o tpgid= -p $c | tr -d " "); [ "$t" = "$c" ] || echo ban; done; done | head -n 1',
  );
  return busy !== 'ban';
}

function resolvePath(p) {
  if (p.startsWith('~/')) return '/home/an/' + p.slice(2);
  if (p.startsWith('/')) return p;
  return sessionCwd() + '/' + p;
}

const sha = (buf) => createHash('sha256').update(buf).digest('hex');

export async function chayPhan({ part, slugs, gui: makeGui, only = [], fresh = true }) {
  const a = await open();
  const { c, sleep } = a;
  let slug = '';
  const filesDir = join(ROOT, part, 'files');

  async function shot(n) {
    await sleep(350);
    const png = join(tmpdir(), `ssh-studio-${slug}-${n}.png`);
    await c.png(png);
    const dir = join(REPO, 'apps', 'web', 'public', 'images', 'articles', slug);
    mkdirSync(dir, { recursive: true });
    await sharp(png).webp({ quality: 90, effort: 6 }).toFile(join(dir, `ssh-studio-${n}.webp`));
    console.log('  ảnh', `${slug}/ssh-studio-${n}.webp`);
  }

  /** Mở một chế độ có khung Files, khung local ở thư mục của lab, khung remote ở `remote`. */
  async function files(view, remote) {
    await a.view(view);
    await a.path('local', LOCAL);
    await a.path('remote', remote);
    await c.click('.file-pane *', 'Remote server', { exact: true }).catch(() => {});
    await sleep(300);
  }

  /** Đối chiếu nội dung file trên server với file mẫu. */
  function verify(path, src) {
    const abs = resolvePath(path);
    const remote = root(`sha256sum '${abs}' | cut -d' ' -f1`);
    const local = sha(readFileSync(join(filesDir, src)));
    if (remote !== local) {
      throw new Error(`Nội dung ${abs} khác files/${src}:\n${root(`cat -A '${abs}' | tail -n 5`)}`);
    }
    console.log('  khớp', abs, '=', src);
  }

  /** Gõ một file bằng vim: xoá nội dung cũ (nếu có), :set paste, gõ từng dòng, lưu. */
  /** sudo đang chờ mật khẩu: còn tiến trình sudo mà chưa có tiến trình con nào. */
  function sudoWaiting() {
    const out = root('for p in $(pgrep -x "sudo|sudoedit"); do [ -z "$(pgrep -P $p)" ] && echo y; done | head -n 1');
    return out === 'y';
  }
  async function answerSudo() {
    await sleep(700);
    if (sudoWaiting()) {
      await c.type(PASSWORD);
      await c.enter();
      await sleep(900);
    }
  }

  /** Mở trình soạn thảo bằng `cmd` (vim, sudoedit, sudo visudo -f), gõ nội dung files/src, lưu. */
  async function vimFile(path, src, shotNo, cmd = 'vim') {
    const abs = resolvePath(path);
    const existed = root(`test -e '${abs}' && echo y || echo n`) === 'y';
    await a.run(`${cmd} ${path}`, 1200);
    if (cmd !== 'vim') {
      await answerSudo();
      await sleep(600);
    }
    if (existed) {
      await c.type(':%d');
      await c.enter();
    }
    await c.type(':set paste');
    await c.enter();
    await c.type('i');
    const lines = readFileSync(join(filesDir, src), 'utf8').replace(/\n$/, '').split('\n');
    for (let i = 0; i < lines.length; i++) {
      if (lines[i]) await c.type(lines[i]);
      if (i < lines.length - 1) await c.enter();
    }
    await c.esc();
    await c.type(':set nopaste');
    await c.enter();
    await c.type('gg');
    await sleep(300);
    if (shotNo) await shot(shotNo);
    await c.type(':wq');
    await c.enter();
    await sleep(700);
    verify(path, src);
  }

  const ctx = { a, c, sleep, shot, files, verify, vimFile, answerSudo, LOCAL, unlinkLocal: (f) => existsSync(join(LOCAL, f)) && unlinkSync(join(LOCAL, f)) };
  const gui = makeGui(ctx);

  async function play(script) {
    slug = slugs[script.id];
    console.log(`${script.name} → ${slug}`);
    const lines = readFileSync(script.path, 'utf8').replace(/\r\n/g, '\n').split('\n');
    for (const line of lines) {
      if (!line.trim()) continue;
      let m;
      if (line === '#clear') await a.clear();
      else if ((m = line.match(/^#shot (\d\d)$/))) await shot(m[1]);
      else if ((m = line.match(/^#gui (\S+)$/))) {
        if (!gui[m[1]]) throw new Error('Chưa có thao tác giao diện: ' + m[1]);
        await gui[m[1]]();
      } else if ((m = line.match(/^#root (.+)$/))) root(m[1]);
      else if ((m = line.match(/^#file (\S+) (\S+)(?: (\d\d))?$/))) await vimFile(m[1], m[2], m[3]);
      else if ((m = line.match(/^#sudofile (sudoedit|visudo) (\S+) (\S+)(?: (\d\d))?$/)))
        await vimFile(m[2], m[3], m[4], m[1] === 'visudo' ? 'sudo visudo -f' : 'sudoedit');
      else if ((m = line.match(/^#login (\S+) (\S+)( otp)?$/))) {
        // Mã OTP chỉ dùng được một lần: chờ sang khung 30 giây mới.
        if (m[3]) await sleep((31 - (Math.floor(Date.now() / 1000) % 30)) * 1000);
        root(`${loginCommand(m[1], m[2], !!m[3])} >/dev/null 2>&1 || true`);
      } else if ((m = line.match(/^#check (\S+) (\S+)$/))) verify(m[1], m[2]);
      else if ((m = line.match(/^#wait (\d+)$/))) await sleep(Number(m[1]) * 1000);
      else if (line.startsWith('#')) continue;
      else {
        if (line.endsWith('\\') || /<</.test(line)) throw new Error('Chưa hỗ trợ lệnh nhiều dòng: ' + line);
        const slow = /^(sudo|find|tree|grep -r|man|cp -r|zcat|tar|shellcheck|xz|zstd|env -i|bash -x)/.test(line);
        await a.run(line, slow ? 1100 : 500);
        if (/^(sudo|sudoedit)\b/.test(line)) await answerSudo();
        // Lệnh chạy lâu (apt, timeout …): chờ shell rảnh rồi mới gõ lệnh sau hay chụp ảnh.
        for (let t = 0; t < 600 && !shellIdle(); t++) await sleep(250);
      }
    }
    await a.run('cd', 250);
  }

  const all = scripts(part);
  const todo = only.length ? only.map((id) => all.find((s) => s.id === id)) : all;
  for (const s of todo) {
    // Mỗi bài là một phiên đăng nhập mới, như runner của lab: thoát rồi kết nối lại.
    await play(s);
  }
  c.close();
}

export { reset, readdirSync };
