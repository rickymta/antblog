// GIT-058 · Hiệu năng làm việc hằng ngày: alias, cấu hình chung của nhóm

import { TEAM_GITCONFIG } from '../lib/history6.mjs';
import { checkRefs6 } from './chung.mjs';

export default {
  id: 'GIT-058',
  start: 7,
  end: 8,
  run(s, { refs6 }) {
    s.say('Cấu hình khuyến nghị của nhóm nằm trong repository, review như code');
    s.write('docs/git/toinoi.gitconfig', TEAM_GITCONFIG);
    s.git('add docs/git');
    s.git('commit -q -m "docs: cấu hình git khuyến nghị cho nhóm"', { as: 'minh', at: '2026-12-04T11:00:00+07:00' });
    s.git('push -q');
    s.blank();
    s.say('Mỗi người bật một lần cho repository (Git không tự đọc cấu hình từ repository, vì lý do an toàn)');
    s.git('config --local include.path ../docs/git/toinoi.gitconfig');
    s.git('config --show-origin --get pull.ff');
    s.git('config --show-scope --get-regexp "^alias\\."');
    s.blank();
    s.say('Dùng alias');
    s.git('st');
    s.git('lg', { show: 'lg' });
    s.git('nhanh');
    s.git('tag -l "v1.*"');
    s.blank();
    s.say('Cấu hình của bạn vẫn thắng nếu đặt sau include (ví dụ muốn pull bằng rebase)');
    s.git('config --local pull.rebase true');
    s.git('config --show-origin --get-regexp "^pull\\."');
    s.git('config --local --unset pull.rebase');
    s.check('alias st chạy được và pull.ff = only', s.value('config --get pull.ff') === 'only' && s.value('st').startsWith('## main'));
    checkRefs6(s, refs6(8));
  },
};
