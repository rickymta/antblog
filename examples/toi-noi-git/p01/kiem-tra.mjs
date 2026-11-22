#!/usr/bin/env node
// Kiểm tra một repository theo tiêu chí của bài thực hành GIT-010.
//
//   node p01/kiem-tra.mjs <đường dẫn repository>
//
// In một dòng ĐẠT / CHƯA ĐẠT cho mỗi tiêu chí; mã thoát 0 khi đạt hết.
// Chỉ ĐỌC repository (git status, ls-files, log, check-ignore, check-attr), không sửa gì.

import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { gitEnv } from '../lib/lab.mjs';

const GENERIC = /^(update|fix|wip|sửa|cập nhật|test|commit|misc|\.+)$/i;

function git(dir, args) {
  const r = spawnSync('git', args, { cwd: dir, env: gitEnv(), encoding: 'utf8' });
  return { ok: r.status === 0, out: (r.stdout ?? '').replace(/\r\n/g, '\n').trimEnd() };
}

/** Trả về danh sách { ok, label, detail }. */
export function inspect(dir) {
  const res = [];
  const add = (ok, label, detail = '') => res.push({ ok, label, detail });

  const top = git(dir, ['rev-parse', '--show-toplevel']);
  if (!top.ok) {
    add(false, 'là một Git repository', 'không tìm thấy thư mục .git');
    return res;
  }
  const commits = git(dir, ['rev-list', '--count', 'HEAD']);
  add(commits.ok && Number(commits.out) >= 3, 'có ít nhất 3 commit', commits.ok ? `${commits.out} commit` : 'chưa có commit');
  if (!commits.ok) return res;

  add(git(dir, ['branch', '--show-current']).out === 'main', 'nhánh hiện tại là main');
  const status = git(dir, ['status', '--porcelain']).out;
  add(status === '', 'working tree sạch (đã commit hết)', status.split('\n').slice(0, 3).join(', '));
  add(git(dir, ['ls-files', 'README.md']).out === 'README.md', 'có README.md');

  const probes = [
    'src/X/bin/Debug/a.dll',
    'src/X/obj/project.assets.json',
    '.env',
    'src/X/appsettings.Development.json',
    'src/X/X.csproj.user',
  ];
  const notIgnored = probes.filter((p) => !git(dir, ['check-ignore', '-q', '--no-index', p]).ok);
  add(notIgnored.length === 0, '.gitignore bỏ qua bin/, obj/, .env, appsettings.Development.json, *.user', notIgnored.join(', '));

  const tracked = git(dir, ['ls-files']).out.split('\n').filter(Boolean);
  const bad = tracked.filter((f) => /(^|\/)(bin|obj)\/|(^|\/)\.env$|appsettings\.Development\.json$|\.user$/.test(f));
  add(bad.length === 0, 'không theo dõi kết quả build hay file bí mật', bad.join(', '));

  const attr = git(dir, ['check-attr', 'text', 'eol', '--', 'README.md']).out;
  add(/eol: lf/.test(attr), '.gitattributes đặt eol=lf cho file văn bản', attr.replace(/\n/g, '; '));

  const crlf = git(dir, ['ls-files', '--eol'])
    .out.split('\n')
    .filter((l) => l.startsWith('i/crlf') || l.startsWith('i/mixed'));
  add(crlf.length === 0, 'không file nào lưu CRLF trong repository', crlf.map((l) => l.split('\t').pop()).join(', '));

  const log = git(dir, ['log', '--format=%h%x09%ae%x09%s']).out.split('\n');
  const long = log.filter((l) => [...l.split('\t')[2]].length > 72);
  add(long.length === 0, 'summary mọi commit ≤ 72 ký tự', long.map((l) => l.split('\t')[0]).join(', '));
  const generic = log.filter((l) => GENERIC.test(l.split('\t')[2].trim()));
  add(generic.length === 0, 'không có summary chung chung ("update", "fix", "wip"…)', generic.map((l) => l.split('\t')[0]).join(', '));
  const noMail = log.filter((l) => !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(l.split('\t')[1]));
  add(noMail.length === 0, 'mọi commit có email tác giả hợp lệ', noMail.map((l) => l.split('\t')[0]).join(', '));

  return res;
}

export function report(dir, print = console.log) {
  const res = inspect(dir);
  for (const r of res) {
    print(`${r.ok ? 'ĐẠT     ' : 'CHƯA ĐẠT'} ${r.label}${!r.ok && r.detail ? ` — ${r.detail}` : ''}`);
  }
  const okAll = res.every((r) => r.ok);
  print(`${res.filter((r) => r.ok).length}/${res.length} tiêu chí đạt`);
  return { okAll, res };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const dir = resolve(process.argv[2] ?? '.');
  const { okAll } = report(dir);
  process.exitCode = okAll ? 0 : 1;
}
