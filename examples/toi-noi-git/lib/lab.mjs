// Hạ tầng chung của lab "Git qua GitStudio và Tới Nơi".
//
// Mọi lệnh git chạy với cấu hình CÔ LẬP để ai chạy cũng ra cùng kết quả:
//   - GIT_CONFIG_NOSYSTEM=1, GIT_CONFIG_GLOBAL=lab.gitconfig: không đọc cấu hình git của máy bạn;
//   - LC_ALL=C: thông báo của git bằng tiếng Anh (máy cài tiếng Việt có thể thấy bản dịch);
//   - danh tính và thời điểm commit truyền qua GIT_AUTHOR_* / GIT_COMMITTER_*.
// Nhờ vậy mã hash commit trong bài trùng với mã hash trên máy bạn.

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

export const LAB = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const WORK = join(LAB, 'work');
export const LAB_CONFIG = join(LAB, 'lab.gitconfig');

export const PEOPLE = {
  minh: { name: 'Trần Minh', email: 'minh@toinoi.test' },
  lan: { name: 'Lê Lan', email: 'lan@toinoi.test' },
};

/** Môi trường cho một lần gọi git: bỏ mọi biến GIT_* thừa hưởng, rồi đặt biến của lab. */
export function gitEnv({ as, at, committedAt, globalConfig } = {}) {
  const env = {};
  for (const [k, v] of Object.entries(process.env)) if (!k.startsWith('GIT_')) env[k] = v;
  Object.assign(env, {
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: globalConfig ?? LAB_CONFIG,
    LC_ALL: 'C',
    LANG: 'C',
    LANGUAGE: 'C',
    GIT_PAGER: 'cat',
    PAGER: 'cat',
    GIT_TERMINAL_PROMPT: '0',
    GIT_EDITOR: ':',
  });
  if (as) {
    const p = PEOPLE[as];
    if (!p) throw new Error(`Không có người tên ${as}`);
    env.GIT_AUTHOR_NAME = p.name;
    env.GIT_AUTHOR_EMAIL = p.email;
    env.GIT_COMMITTER_NAME = p.name;
    env.GIT_COMMITTER_EMAIL = p.email;
  }
  if (at) {
    env.GIT_AUTHOR_DATE = at;
    env.GIT_COMMITTER_DATE = committedAt ?? at;
  }
  return env;
}

/** Tách một dòng lệnh thành đối số, hiểu nháy kép và nháy đơn (không hiểu biến, glob). */
export function splitArgs(line) {
  const out = [];
  let cur = '';
  let quote = null;
  let has = false;
  for (const ch of line) {
    if (quote) {
      if (ch === quote) quote = null;
      else cur += ch;
    } else if (ch === '"' || ch === "'") {
      quote = ch;
      has = true;
    } else if (ch === ' ') {
      if (has || cur) out.push(cur);
      cur = '';
      has = false;
    } else {
      cur += ch;
    }
  }
  if (quote) throw new Error(`Thiếu dấu nháy đóng: ${line}`);
  if (has || cur) out.push(cur);
  return out;
}

/** Thay đường dẫn tuyệt đối của lab bằng <lab> để kết quả giống nhau trên mọi máy. */
export function normalize(text) {
  const variants = new Set([LAB, LAB.split(sep).join('/'), LAB.split('\\').join('/')]);
  // Git for Windows có lúc in ổ đĩa viết thường (h:/…) hoặc dạng /h/…
  for (const v of [...variants]) {
    if (/^[A-Za-z]:/.test(v)) {
      variants.add(v[0].toLowerCase() + v.slice(1));
      variants.add(v[0].toUpperCase() + v.slice(1));
      variants.add('/' + v[0].toLowerCase() + v.slice(2).split('\\').join('/'));
    }
  }
  let t = text.replace(/\r\n/g, '\n');
  for (const v of [...variants].sort((a, b) => b.length - a.length)) t = t.split(v).join('<lab>');
  return t.replace(/\n+$/, '');
}

/** Chạy git không in gì; ném lỗi nếu mã thoát khác 0. */
export function gitQuiet(cwd, args, opts = {}) {
  const r = spawnSync('git', args, {
    cwd,
    env: gitEnv(opts),
    input: opts.input,
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
  if (r.error) throw r.error;
  if (r.status !== 0 && !opts.allowFail) {
    throw new Error(`git ${args.join(' ')} (mã ${r.status}) trong ${cwd}\n${r.stdout}${r.stderr}`);
  }
  return { stdout: r.stdout ?? '', stderr: r.stderr ?? '', status: r.status };
}

export function writeText(dir, rel, content) {
  const full = join(dir, ...rel.split('/'));
  mkdirSync(dirname(full), { recursive: true });
  writeFileSync(full, content);
}

export function readText(dir, rel) {
  return readFileSync(join(dir, ...rel.split('/')), 'utf8');
}

export function moveFile(dir, from, to) {
  const dst = join(dir, ...to.split('/'));
  mkdirSync(dirname(dst), { recursive: true });
  renameSync(join(dir, ...from.split('/')), dst);
}

export function freshDir(dir) {
  // Xoá NỘI DUNG thay vì xoá thư mục: thư mục có thể đang mở trong GitStudio hay terminal.
  mkdirSync(dir, { recursive: true });
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    try {
      rmSync(p, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 });
    } catch (e) {
      // Thư mục con đang bị giữ (EBUSY/EPERM): dọn bên trong, để lại thư mục rỗng.
      if (e.code !== 'EBUSY' && e.code !== 'EPERM') throw e;
      freshDir(p);
    }
  }
}

/**
 * Một phiên chạy bài: in lệnh và kết quả theo dạng
 *   $ git status
 *   On branch main
 * và đếm PASS/FAIL cho các kiểm tra.
 */
export class Session {
  constructor(print = (s) => console.log(s)) {
    this.print = print;
    this.cwd = WORK;
    this.pass = 0;
    this.fail = 0;
  }

  say(text) {
    for (const line of text.split('\n')) this.print(line ? `# ${line}` : '#');
  }

  blank() {
    this.print('');
  }

  /** Đổi thư mục làm việc (tương đối với work/). In `$ cd …` nếu show. */
  cd(rel, { show = true } = {}) {
    this.cwd = resolve(this.cwd, rel);
    if (show) this.print(`$ cd ${rel}`);
  }

  write(rel, content) {
    writeText(this.cwd, rel, content);
  }

  read(rel) {
    return readText(this.cwd, rel);
  }

  move(from, to) {
    moveFile(this.cwd, from, to);
  }

  remove(rel) {
    rmSync(join(this.cwd, ...rel.split('/')), { recursive: true, force: true });
  }

  /** Liệt kê tên trong một thư mục như `Get-ChildItem -Force -Name`. */
  ls(rel = '.') {
    this.print(`$ Get-ChildItem -Force -Name ${rel}`);
    const names = readdirSync(join(this.cwd, ...rel.split('/'))).sort((a, b) => a.localeCompare(b, 'en'));
    for (const n of names) this.print(n);
  }

  /**
   * Chạy `git <cmd>`. opts: as, at, committedAt (danh tính và thời điểm commit),
   * input (stdin), expectFail, quiet (không in), show (chuỗi lệnh hiển thị thay cho cmd),
   * globalConfig (file cấu hình toàn cục thay cho lab.gitconfig).
   */
  git(cmd, opts = {}) {
    const args = splitArgs(cmd);
    const r = spawnSync('git', args, {
      cwd: this.cwd,
      env: gitEnv(opts),
      input: opts.input,
      encoding: 'utf8',
      maxBuffer: 32 * 1024 * 1024,
    });
    if (r.error) throw r.error;
    const out = normalize((r.stdout ?? '') + (r.stderr ?? ''));
    if (!opts.quiet) {
      this.print(`$ git ${opts.show ?? cmd}`);
      if (out) this.print(out);
    }
    if (r.status !== 0 && !opts.expectFail) {
      throw new Error(`git ${cmd} thoát mã ${r.status}\n${out}`);
    }
    if (r.status === 0 && opts.expectFail) {
      throw new Error(`git ${cmd} lẽ ra phải lỗi nhưng chạy thành công`);
    }
    return { out, stdout: normalize(r.stdout ?? ''), status: r.status };
  }

  /** Giá trị một lệnh git, không in. */
  value(cmd, opts = {}) {
    return this.git(cmd, { ...opts, quiet: true }).stdout.trim();
  }

  check(label, ok, detail = '') {
    if (ok) {
      this.pass++;
      this.print(`PASS ${label}`);
    } else {
      this.fail++;
      this.print(`FAIL ${label}${detail ? ` — ${detail}` : ''}`);
    }
  }

  exists(rel) {
    return existsSync(join(this.cwd, ...rel.split('/')));
  }
}
