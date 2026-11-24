#!/usr/bin/env node
// Kiểm tra repository của Minh theo tiêu chí của GIT-050 (Phần 5).
//
//   node p05/kiem-tra.mjs <đường dẫn repository cục bộ>
//
// Chỉ đọc: ls-remote để so với remote, fsck, tag -v; không đổi ref nào.

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { gitEnv } from '../lib/lab.mjs';

function git(dir, args) {
  const r = spawnSync('git', args, { cwd: dir, env: gitEnv(), encoding: 'utf8' });
  return { ok: r.status === 0, out: (r.stdout ?? '').replace(/\r\n/g, '\n').trimEnd(), err: (r.stderr ?? '').trimEnd() };
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

  const fare = git(dir, ['show', 'main:src/BaoGia/FareCalculator.cs']).out;
  const bug = git(dir, ['log', '--format=%H', '-S', 'Math.Round(atLeastMinimum', 'main']).out;
  const reverted = bug && git(dir, ['log', '--format=%h', '--grep', `This reverts commit ${bug.split('\n').pop()}`, 'main']).out;
  add(fare.includes('Math.Ceiling(atLeastMinimum') && !!reverted, 'lỗi làm tròn đã được revert, main làm tròn lên bội số 1.000 đ', reverted ? '' : 'không thấy commit revert');

  const pending = ['BISECT_LOG', 'MERGE_HEAD', 'rebase-merge', 'rebase-apply', 'CHERRY_PICK_HEAD', 'REVERT_HEAD'].filter((f) => existsSync(join(gd.out, f)));
  const status = git(dir, ['status', '--porcelain']).out;
  add(pending.length === 0 && status === '', 'không bisect hay thao tác nào dở dang, working tree sạch', [...pending, status.split('\n')[0]].filter(Boolean).join(', '));

  const attr = git(dir, ['check-attr', 'diff', '--', 'src/BaoGia/FareCalculator.cs']).out;
  add(attr.endsWith('diff: csharp'), '.gitattributes khai báo diff=csharp cho file C#', attr);

  const sub = git(dir, ['submodule', 'status']).out;
  add(/^ [0-9a-f]{40} lib\/tien-te/.test(sub), 'submodule lib/tien-te đã khởi tạo và đúng commit được ghi nhận', sub || 'không có submodule');

  const lfs = git(dir, ['lfs', 'ls-files', '-n']).out.split('\n');
  add(lfs.includes('docs/thiet-ke/bang-gia-2027.png'), 'bản thiết kế bảng giá lưu bằng Git LFS', lfs.join(', ') || 'chưa có file LFS');

  const tag = git(dir, ['tag', '-v', 'v1.2.0']);
  add(tag.ok && /Good "git" signature/.test(tag.err), 'tag v1.2.0 có chữ ký hợp lệ', tag.ok ? '' : (tag.err.split('\n').pop() || 'thiếu tag'));

  const hooksPath = git(dir, ['config', '--get', 'core.hooksPath']).out;
  const modes = git(dir, ['ls-files', '-s', '.githooks']).out.split('\n').filter(Boolean);
  const exe = modes.length >= 3 && modes.every((l) => l.startsWith('100755'));
  add(hooksPath === '.githooks' && exe, 'core.hooksPath = .githooks, hook trong index có quyền chạy (100755)', `hooksPath=${hooksPath || 'chưa đặt'}, ${modes.length} hook`);

  const graph = existsSync(join(gd.out, 'objects', 'info', 'commit-graph')) || existsSync(join(gd.out, 'objects', 'info', 'commit-graphs'));
  const fsck = git(dir, ['fsck', '--full', '--no-progress', '--no-dangling']);
  add(graph && fsck.ok, 'có commit-graph và fsck không báo lỗi', `${graph ? '' : 'chưa ghi commit-graph; '}${fsck.ok ? '' : fsck.err.split('\n')[0]}`);

  const report = git(dir, ['show', 'main:docs/bao-cao-sai-cuoc.md']);
  add(report.ok && /bisect/.test(report.out), 'main có báo cáo điều tra docs/bao-cao-sai-cuoc.md nhắc bisect', report.ok ? 'báo cáo chưa nhắc bisect' : 'chưa có file');
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
