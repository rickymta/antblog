#!/usr/bin/env node
// Kiểm tra repository của Minh theo tiêu chí của GIT-060 (Phần 6).
//
//   node p06/kiem-tra.mjs <đường dẫn repository cục bộ>
//
// Chỉ đọc: ls-remote để so với remote, tag -v, subtree split (chỉ tính toán, không tạo nhánh).

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { gitEnv, shPath } from '../lib/lab.mjs';

export const CC_RE = /^(feat|fix|perf|docs|test|refactor|style|build|ci|chore|revert)(\([a-z0-9-]+\))?!?: .{3,}/;
export const SECRET_RE = 'tn_live_[A-Za-z0-9_]{8,}';

function run(cmd, dir, args) {
  const r = spawnSync(cmd, args, { cwd: dir, env: gitEnv(), encoding: 'utf8' });
  return { ok: r.status === 0, out: (r.stdout ?? '').replace(/\r\n/g, '\n').trimEnd(), err: (r.stderr ?? '').replace(/\r\n/g, '\n').trimEnd() };
}
const git = (dir, args) => run('git', dir, args);

export function inspect(dir) {
  const res = [];
  const add = (ok, label, detail = '') => res.push({ ok, label, detail });
  const gd = git(dir, ['rev-parse', '--absolute-git-dir']);
  if (!gd.ok) {
    add(false, 'là một Git repository', 'không tìm thấy thư mục .git');
    return res;
  }
  const remote = Object.fromEntries(
    git(dir, ['ls-remote', 'origin'])
      .out.split('\n')
      .filter(Boolean)
      .map((l) => l.split('\t').reverse()),
  );

  // 1
  const localMain = git(dir, ['rev-parse', 'main']).out;
  const remoteMain = remote['refs/heads/main'] ?? '';
  add(localMain === remoteMain, 'main cục bộ trùng main trên remote', `cục bộ ${localMain.slice(0, 7)}, remote ${remoteMain.slice(0, 7) || '—'}`);

  // 2
  const pending = ['MERGE_HEAD', 'rebase-merge', 'rebase-apply', 'CHERRY_PICK_HEAD', 'REVERT_HEAD', 'BISECT_LOG', 'index.lock'].filter((f) => existsSync(join(gd.out, f)));
  const status = git(dir, ['status', '--porcelain']).out;
  add(pending.length === 0 && status === '', 'working tree sạch, không thao tác dở, không còn index.lock', [...pending, status.split('\n')[0]].filter(Boolean).join(', '));

  // 3 (commit của thư viện gộp vào bằng subtree giữ tiêu đề cũ: không tính, nhận ra qua subtree split)
  const split = git(dir, ['subtree', 'split', '-q', '--prefix=lib/tien-te', 'main']).out.split('\n').pop();
  const start = git(dir, ['log', '--format=%H', '--grep', '^docs: quy ước commit theo Conventional Commits', 'main']).out.split('\n')[0];
  const notLib = split ? ['--not', split] : [];
  const subjects = start ? git(dir, ['log', '--no-merges', '--format=%s', `${start}~1..main`, ...notLib]).out.split('\n').filter(Boolean) : [];
  const bad = subjects.filter((x) => !CC_RE.test(x) && !x.startsWith('Revert "'));
  add(start && bad.length === 0, 'mọi commit từ khi áp dụng quy ước đều theo Conventional Commits', start ? bad.slice(0, 2).join(' | ') : 'chưa có commit áp dụng quy ước');

  // 4
  const cl = git(dir, ['show', 'main:CHANGELOG.md']);
  const vers = [...cl.out.matchAll(/^## (\d+\.\d+\.\d+) /gm)].map((m) => m[1]);
  const runbook = git(dir, ['cat-file', '-e', 'main:docs/quy-trinh-phat-hanh.md']).ok;
  add(vers.slice(0, 3).join(',') === '1.4.0,1.3.1,1.3.0' && runbook, 'CHANGELOG có 1.4.0, 1.3.1, 1.3.0 (mới nhất trên cùng) và có runbook phát hành', `${vers.join(', ') || 'chưa có CHANGELOG'}${runbook ? '' : '; thiếu docs/quy-trinh-phat-hanh.md'}`);

  // 5
  const tagV = git(dir, ['tag', '-v', 'v1.4.0']);
  const exact = git(dir, ['describe', '--exact-match', 'main']).out;
  const remoteTag = remote['refs/tags/v1.4.0'];
  add(tagV.ok && /Good "git" signature/.test(tagV.err) && exact === 'v1.4.0' && remoteTag === git(dir, ['rev-parse', 'v1.4.0']).out, 'tag v1.4.0 có chữ ký hợp lệ, trỏ đúng main và đã push', `${tagV.ok ? '' : 'chữ ký/tag lỗi; '}describe=${exact || '—'}; remote=${remoteTag ? 'có' : 'chưa có'}`);

  // 6
  const b14 = remote['refs/heads/phat-hanh/1.4'];
  const b13 = remote['refs/heads/phat-hanh/1.3'];
  const in13 = b13 && git(dir, ['merge-base', '--is-ancestor', 'v1.3.1', b13]).ok;
  add(b14 === git(dir, ['rev-parse', 'v1.4.0^{commit}']).out && !!in13, 'remote có phat-hanh/1.4 ở v1.4.0 và phat-hanh/1.3 chứa v1.3.1', `phat-hanh/1.4: ${b14 ? b14.slice(0, 7) : 'chưa có'}; phat-hanh/1.3: ${b13 ? (in13 ? 'đúng' : 'thiếu v1.3.1') : 'chưa có'}`);

  // 7
  const leak = git(dir, ['log', '--all', '--format=%h %s', '-G', SECRET_RE]).out;
  add(leak === '', 'không còn khoá API trong lịch sử của mọi nhánh', leak.split('\n')[0]);

  // 8
  const ci = git(dir, ['show', 'main:.github/workflows/ci.yml']).out;
  const ver = run(shPath(), dir, ['scripts/phien-ban.sh']).out;
  add(/fetch-depth: 0/.test(ci) && ver === '1.4.0', 'CI checkout đủ lịch sử (fetch-depth: 0), scripts/phien-ban.sh ra 1.4.0', `phien-ban=${ver || 'lỗi'}${/fetch-depth: 0/.test(ci) ? '' : '; ci.yml thiếu fetch-depth: 0'}`);

  // 9
  const hooksPath = git(dir, ['config', '--get', 'core.hooksPath']).out;
  const msgHook = git(dir, ['show', 'main:.githooks/commit-msg']).out;
  const preHook = git(dir, ['show', 'main:.githooks/pre-commit']).out;
  add(hooksPath === '.githooks' && msgHook.includes('feat|fix') && preHook.includes('tn_live_'), 'core.hooksPath = .githooks; commit-msg kiểm tra Conventional Commits, pre-commit chặn khoá API', `hooksPath=${hooksPath || 'chưa đặt'}`);

  // 10
  const gm = git(dir, ['cat-file', '-e', 'main:.gitmodules']).ok;
  const libTip = git(dir, ['ls-remote', '../remote/tien-te.git', 'refs/heads/main']).out.split('\t')[0];
  const owners = git(dir, ['cat-file', '-e', 'main:.github/CODEOWNERS']).ok;
  add(!gm && split && split === libTip && owners, 'thư viện tiền tệ gộp bằng subtree (split ra đúng commit cuối của tien-te), có CODEOWNERS', `${gm ? 'còn .gitmodules; ' : ''}split=${split ? split.slice(0, 7) : '—'}, tien-te=${libTip.slice(0, 7) || '—'}${owners ? '' : '; thiếu CODEOWNERS'}`);
  return res;
}

export function report(dir, print = console.log) {
  const res = inspect(dir);
  for (const r of res) print(`${r.ok ? 'ĐẠT     ' : 'CHƯA ĐẠT'} ${r.label}${!r.ok && r.detail ? ` — ${r.detail}` : ''}`);
  const okAll = res.every((r) => r.ok);
  print(`${res.filter((r) => r.ok).length}/${res.length} tiêu chí đạt`);
  return { okAll, res };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const { okAll } = report(resolve(process.argv[2] ?? '.'));
  process.exitCode = okAll ? 0 : 1;
}
