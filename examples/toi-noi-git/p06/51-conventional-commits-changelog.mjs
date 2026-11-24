// GIT-051 · Conventional Commits và changelog

import { CC_COMMITS, CHANGELOG_HEAD, CHANGELOG_SH, CONTRIBUTING, HOOK_COMMIT_MSG_CC, addChangelogEntry } from '../lib/history6.mjs';
import { checkRefs6, sh } from './chung.mjs';

const M = (t) => ({ as: 'minh', at: `2026-11-30T${t}:00+07:00` });

export default {
  id: 'GIT-051',
  start: 0,
  end: 1,
  run(s, { refs6 }) {
    s.say('Tiêu đề commit tới nay viết tự do: khó lọc "tính năng mới" hay "sửa lỗi" để ghi CHANGELOG');
    s.git('log --format=%s -6');
    s.blank();
    s.say('Quy ước của nhóm (CONTRIBUTING.md) và hook commit-msg kiểm tra tiêu đề');
    s.write('CONTRIBUTING.md', CONTRIBUTING);
    s.write('.githooks/commit-msg', HOOK_COMMIT_MSG_CC);
    s.git('add CONTRIBUTING.md .githooks/commit-msg');
    s.git('commit -q -m "docs: quy ước commit theo Conventional Commits"', M('09:00'));
    s.blank();
    s.say('Tiêu đề kiểu cũ bị hook từ chối');
    s.git('commit --allow-empty -m "Sửa lỗi phí chờ"', { ...M('09:10'), expectFail: true });
    s.blank();
    s.say('Các commit trong ngày theo quy ước');
    for (const [msg, file, body, who, t] of CC_COMMITS) {
      if (typeof body === 'function') s.write(file, body(s.read(file)));
      else s.write(file, body);
      s.git(`add ${file}`, { quiet: true });
      const opts = who === CC_COMMITS[0][3] ? { as: 'lan', by: 'minh' } : { as: 'minh' };
      s.git(`commit -q -m "${msg}"`, { ...opts, at: `2026-11-30T${t}:00+07:00` });
    }
    s.git('log --oneline v1.2.0..main');
    s.blank();
    s.say('Lọc theo loại: chỉ tính năng mới, chỉ sửa lỗi');
    s.git('log --format=%s -E --grep="^feat(\\(.*\\))?!?:" v1.2.0..main');
    s.git('log --format=%s -E --grep="^fix(\\(.*\\))?!?:" v1.2.0..main');
    s.blank();
    s.say('Script sinh một mục CHANGELOG từ tiêu đề commit');
    s.write('scripts/changelog.sh', CHANGELOG_SH);
    s.git('add scripts/changelog.sh');
    s.git('update-index --chmod=+x scripts/changelog.sh');
    s.git('commit -q -m "build: script sinh changelog từ tiêu đề commit"', M('14:00'));
    const entry = sh(s, ['scripts/changelog.sh', '1.3.0', '2026-11-30', 'v1.2.0']).out;
    s.write('CHANGELOG.md', addChangelogEntry(CHANGELOG_HEAD, entry));
    s.git('add CHANGELOG.md');
    s.git('commit -q -m "chore(release): 1.3.0"', M('14:30'));
    s.git('push -q');
    s.git('show --stat --format=%s HEAD');
    s.check('CHANGELOG có mục 1.3.0 với một tính năng và một sửa lỗi', /## 1\.3\.0[\s\S]*Tính năng mới\n- cuoc: phụ phí ngày lễ 30%[\s\S]*Sửa lỗi\n- phi-cho:/.test(s.read('CHANGELOG.md')));
    checkRefs6(s, refs6(1));
  },
};
