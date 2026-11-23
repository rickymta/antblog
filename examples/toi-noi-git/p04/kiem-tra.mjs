#!/usr/bin/env node
// Kiểm tra repository của Minh theo tiêu chí của GIT-040 (Phần 4).
//
//   node p04/kiem-tra.mjs <đường dẫn repository cục bộ>
//
// Chỉ đọc: ls-remote để so với remote, không đổi ref nào.

import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { gitEnv } from '../lib/lab.mjs';

const FEATURE = 'tinh-nang/phu-phi-dem';

function git(dir, args) {
  const r = spawnSync('git', args, { cwd: dir, env: gitEnv(), encoding: 'utf8' });
  return { ok: r.status === 0, out: (r.stdout ?? '').replace(/\r\n/g, '\n').trimEnd() };
}

export function inspect(dir) {
  const res = [];
  const add = (ok, label, detail = '') => res.push({ ok, label, detail });
  const gd = git(dir, ['rev-parse', '--absolute-git-dir']);
  if (!gd.ok) {
    add(false, 'là một Git repository', 'không tìm thấy thư mục .git');
    return res;
  }

  const remoteMain = git(dir, ['ls-remote', 'origin', 'refs/heads/main']).out.split('\t')[0];
  const localMain = git(dir, ['rev-parse', 'main']).out;
  add(remoteMain === localMain, 'main cục bộ trùng main trên remote', `cục bộ ${localMain.slice(0, 7)}, remote ${remoteMain.slice(0, 7)}`);

  const merge = git(dir, ['log', '--merges', '--format=%H', '-1', `--grep=^Merge branch '${FEATURE}'`, 'main']).out;
  add(merge !== '', `nhánh ${FEATURE} đã merge vào main`, 'không thấy merge commit của nhánh');

  const count = merge ? git(dir, ['rev-list', '--count', `${merge}^1..${merge}^2`]).out : '';
  add(count === '2', 'nhánh tính năng vào main gọn: đúng 2 commit', merge ? `${count} commit` : '');

  const messy = git(dir, ['log', '--format=%h %s', 'main'])
    .out.split('\n')
    .filter((l) => /^\S+ (fixup!|squash!|amend!|WIP|debug)/i.test(l));
  add(messy.length === 0, 'main không có commit fixup!/squash!/amend!/WIP/debug', messy.slice(0, 2).join('; '));

  const debug = git(dir, ['grep', '-n', 'Console.WriteLine', 'main', '--', 'src']).out;
  add(debug === '', 'code trên main không còn dòng in debug', debug.split('\n')[0]);

  const fee = git(dir, ['show', 'main:src/BaoGia/WaitingFee.cs']).out;
  add(fee.includes('30000m);') && !fee.includes('50000m'), 'phí chờ tối đa vẫn 30.000 đ (đã revert mức 50.000 đ)');

  add(git(dir, ['config', '--get', 'rerere.enabled']).out === 'true', 'đã bật rerere', 'rerere.enabled chưa là true');

  const runbook = git(dir, ['show', 'main:docs/runbook-hoan-tac.md']);
  const missing = ['restore', 'reset', 'revert', 'reflog'].filter((w) => !runbook.out.includes(`git ${w}`));
  add(runbook.ok && missing.length === 0, 'main có docs/runbook-hoan-tac.md nhắc restore, reset, revert, reflog', runbook.ok ? `thiếu: ${missing.join(', ')}` : 'chưa có file');

  const extra = git(dir, ['for-each-ref', '--format=%(refname:short)', 'refs/heads'])
    .out.split('\n')
    .filter((b) => b && b !== 'main' && !b.startsWith('phat-hanh/'));
  add(extra.length === 0, 'không còn nhánh cục bộ thừa (chỉ main, phat-hanh/*)', extra.join(', '));

  const status = git(dir, ['status', '--porcelain']).out;
  const pending = ['MERGE_HEAD', 'rebase-merge', 'rebase-apply', 'CHERRY_PICK_HEAD', 'REVERT_HEAD'].filter((f) => existsSync(join(gd.out, f)));
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
void readFileSync;
