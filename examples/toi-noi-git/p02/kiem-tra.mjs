#!/usr/bin/env node
// Kiểm tra một repository theo tiêu chí của bài thực hành GIT-020 (Phần 2).
//
//   node p02/kiem-tra.mjs <đường dẫn repository>
//
// Chỉ ĐỌC repository. In một dòng ĐẠT / CHƯA ĐẠT cho mỗi tiêu chí; mã thoát 0 khi đạt hết.

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { gitEnv } from '../lib/lab.mjs';

function git(dir, args) {
  const r = spawnSync('git', args, { cwd: dir, env: gitEnv(), encoding: 'utf8' });
  return { ok: r.status === 0, out: (r.stdout ?? '').replace(/\r\n/g, '\n').trimEnd() };
}

export function inspect(dir) {
  const res = [];
  const add = (ok, label, detail = '') => res.push({ ok, label, detail });
  const gitDir = git(dir, ['rev-parse', '--absolute-git-dir']);
  if (!gitDir.ok) {
    add(false, 'là một Git repository', 'không tìm thấy thư mục .git');
    return res;
  }
  const gd = gitDir.out;

  const pending = ['MERGE_HEAD', 'CHERRY_PICK_HEAD', 'REVERT_HEAD', 'rebase-merge', 'rebase-apply'].filter((f) => existsSync(join(gd, f)));
  add(pending.length === 0, 'không có merge/rebase/cherry-pick/revert dở dang', pending.join(', '));

  const status = git(dir, ['status', '--porcelain']).out;
  add(status === '', 'working tree sạch', status.split('\n').slice(0, 3).join(', '));

  const markers = git(dir, ['grep', '-n', '-E', '^(<<<<<<< |>>>>>>> |=======$)']).out;
  add(markers === '', 'không còn dấu xung đột trong file đã commit', markers.split('\n').slice(0, 2).join(', '));

  const merges = git(dir, ['rev-list', '--merges', '--count', 'main']);
  add(merges.ok && Number(merges.out) >= 1, 'main có ít nhất một merge commit', merges.ok ? `${merges.out} merge commit` : 'không có nhánh main');

  const tagType = (t) => git(dir, ['cat-file', '-t', t]).out;
  const tags = ['v1.0.0', 'v1.1.0', 'v1.0.1'];
  const badTags = tags.filter((t) => tagType(t) !== 'tag');
  add(badTags.length === 0, 'có tag annotated v1.0.0, v1.1.0, v1.0.1', badTags.map((t) => `${t}: ${tagType(t) || 'không có'}`).join(', '));

  add(git(dir, ['merge-base', '--is-ancestor', 'v1.1.0', 'main']).ok, 'v1.1.0 nằm trên lịch sử của main');

  const rel = git(dir, ['rev-parse', '--verify', '-q', 'refs/heads/phat-hanh/1.0']);
  const picked = rel.ok && git(dir, ['log', '--format=%b', 'v1.0.0..phat-hanh/1.0']).out.includes('(cherry picked from commit');
  add(rel.ok && picked, 'nhánh phat-hanh/1.0 có commit cherry-pick ghi nguồn (-x)', rel.ok ? '' : 'không có nhánh phat-hanh/1.0');
  add(
    rel.ok && git(dir, ['merge-base', '--is-ancestor', 'v1.0.1', 'phat-hanh/1.0']).ok && !git(dir, ['merge-base', '--is-ancestor', 'v1.0.1', 'main']).ok,
    'v1.0.1 nằm trên nhánh phát hành, không nằm trên main',
  );

  const leftovers = git(dir, ['branch', '--merged', 'main', '--format=%(refname:short)'])
    .out.split('\n')
    .filter((b) => b && b !== 'main' && !b.startsWith('phat-hanh/'));
  add(leftovers.length === 0, 'nhánh đã merge vào main đã được xoá', leftovers.join(', '));

  const stash = git(dir, ['stash', 'list']).out;
  add(stash === '', 'không còn stash bị bỏ quên', stash.split('\n')[0]);

  const wts = git(dir, ['worktree', 'list', '--porcelain']).out.split('\n').filter((l) => l.startsWith('worktree '));
  add(wts.length === 1, 'không còn worktree phụ', wts.slice(1).map((l) => l.slice(9)).join(', '));

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
