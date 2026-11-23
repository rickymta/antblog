#!/usr/bin/env node
// Kiểm tra repository của một thành viên và remote chung theo tiêu chí của GIT-030 (Phần 3).
//
//   node p03/kiem-tra.mjs <đường dẫn repository cục bộ>
//
// Script chạy `git fetch --dry-run` để so với remote nhưng không đổi ref nào; còn lại chỉ đọc.

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { gitEnv } from '../lib/lab.mjs';

function git(dir, args) {
  const r = spawnSync('git', args, { cwd: dir, env: gitEnv(), encoding: 'utf8' });
  return { ok: r.status === 0, out: ((r.stdout ?? '') + (args[0] === 'fetch' ? r.stderr ?? '' : '')).replace(/\r\n/g, '\n').trimEnd() };
}

export function inspect(dir) {
  const res = [];
  const add = (ok, label, detail = '') => res.push({ ok, label, detail });
  const gd = git(dir, ['rev-parse', '--absolute-git-dir']);
  if (!gd.ok) {
    add(false, 'là một Git repository', 'không tìm thấy thư mục .git');
    return res;
  }

  const origin = git(dir, ['remote', 'get-url', 'origin']);
  add(origin.ok, 'có remote origin', origin.ok ? '' : 'chưa có remote origin');
  if (!origin.ok) return res;

  const up = git(dir, ['rev-parse', '--abbrev-ref', 'main@{upstream}']);
  add(up.ok && up.out === 'origin/main', 'main theo dõi origin/main', up.ok ? up.out : 'chưa đặt upstream');

  const remoteMain = git(dir, ['ls-remote', 'origin', 'refs/heads/main']).out.split('\t')[0];
  const localMain = git(dir, ['rev-parse', 'main']).out;
  add(remoteMain === localMain, 'main cục bộ trùng main trên remote (đã push, đã pull)', `cục bộ ${localMain.slice(0, 7)}, remote ${remoteMain.slice(0, 7)}`);

  const staleFetch = git(dir, ['fetch', '--dry-run', '--prune', 'origin']).out;
  add(staleFetch === '', 'ref remote-tracking đã cập nhật (fetch --prune không còn gì để làm)', staleFetch.split('\n').slice(0, 2).join('; '));

  const tags = git(dir, ['ls-remote', '--tags', 'origin']).out;
  const missing = ['v1.0.0', 'v1.1.0', 'v1.0.1'].filter((t) => !tags.includes(`refs/tags/${t}`));
  add(missing.length === 0, 'remote có tag v1.0.0, v1.1.0, v1.0.1', missing.join(', '));

  const heads = git(dir, ['ls-remote', '--heads', 'origin'])
    .out.split('\n')
    .filter(Boolean)
    .map((l) => l.split('\t')[1].replace('refs/heads/', ''));
  const leftover = heads.filter((b) => b !== 'main' && !b.startsWith('phat-hanh/') && git(dir, ['merge-base', '--is-ancestor', `origin/${b}`, 'origin/main']).ok);
  add(leftover.length === 0, 'remote không còn nhánh đã merge', leftover.join(', '));

  const messy = git(dir, ['log', '--format=%h %s', 'main']).out.split('\n').filter((l) => /Merge branch '[^']+' of /.test(l));
  add(messy.length === 0, 'main không có merge commit thừa do pull ("Merge branch … of …")', messy.slice(0, 2).join('; '));

  const pullCfg = git(dir, ['config', '--get', 'pull.rebase']).out || git(dir, ['config', '--get', 'pull.ff']).out;
  add(pullCfg !== '', 'đã cấu hình cách pull (pull.rebase hoặc pull.ff)', 'chưa đặt pull.rebase hay pull.ff');

  const gone = git(dir, ['branch', '-vv']).out.split('\n').filter((l) => l.includes(': gone]'));
  add(gone.length === 0, 'không còn nhánh cục bộ theo dõi nhánh remote đã xoá', gone.map((l) => l.trim().split(/\s+/)[l.trim().startsWith('*') ? 1 : 0]).join(', '));

  const status = git(dir, ['status', '--porcelain']).out;
  const pending = ['MERGE_HEAD', 'rebase-merge', 'rebase-apply', 'CHERRY_PICK_HEAD'].filter((f) => existsSync(join(gd.out, f)));
  add(status === '' && pending.length === 0, 'working tree sạch, không có thao tác dở dang', [status.split('\n')[0], ...pending].filter(Boolean).join(', '));

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
