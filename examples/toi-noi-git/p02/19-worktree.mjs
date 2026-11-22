// GIT-019 · Worktree: làm hai nhánh cùng lúc

import { CHANGELOG_10, RELEASE } from '../lib/history2.mjs';
import { checkRefs } from './chung.mjs';

export default {
  id: 'GIT-019',
  start: 11,
  end: 12,
  run(s, { refs2 }) {
    s.git('worktree list');
    s.blank();
    s.say('Cần sửa nhánh phát hành mà không đụng tới working tree của main');
    s.git(`worktree add ../bao-gia-1.0 ${RELEASE}`);
    s.git('worktree list');
    s.git('branch -v');
    s.blank();
    s.say('Một nhánh chỉ được checkout ở một worktree');
    s.git(`switch ${RELEASE}`, { expectFail: true });
    s.blank();
    s.say('Làm việc trong worktree thứ hai như một thư mục bình thường');
    s.cd('../bao-gia-1.0');
    s.git('status --short --branch');
    s.write('CHANGELOG.md', CHANGELOG_10);
    s.git('add CHANGELOG.md');
    s.git('commit -m "Thêm ghi chú phát hành 1.0.1"', { as: 'minh', at: '2026-11-19T09:00:00+07:00' });
    s.blank();
    s.say('Commit vừa tạo có ngay trong repository chính (cùng .git)');
    s.cd('../bao-gia');
    s.git(`log --oneline -2 ${RELEASE}`);
    s.git('status --short --branch');
    s.check('working tree của main không có CHANGELOG.md', !s.exists('CHANGELOG.md'));
    s.blank();
    s.say('Xong việc: gỡ worktree');
    s.git('worktree remove ../bao-gia-1.0');
    s.git('worktree list');
    checkRefs(s, refs2(12));
  },
};
