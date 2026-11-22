// GIT-014 · Xung đột merge và công cụ giải quyết xung đột

import { CALC_MERGED, FEATURE } from '../lib/history2.mjs';
import { checkRefs } from './chung.mjs';

const AT = { as: 'minh', at: '2026-11-17T09:00:00+07:00' };

export default {
  id: 'GIT-014',
  start: 5,
  end: 6,
  run(s, { refs2 }) {
    s.say('Hai nhánh tách nhau từ commit nào, mỗi bên đã đổi gì');
    s.git(`merge-base main ${FEATURE}`);
    s.git(`log --oneline main..${FEATURE}`);
    s.git(`log --oneline ${FEATURE}..main`);
    s.blank();
    s.say('Merge nhánh tính năng vào main');
    s.git(`merge ${FEATURE}`, { ...AT, expectFail: true });
    s.git('status');
    s.say('Phần xung đột trong file (diff gộp của hai bên)');
    s.git('diff');
    s.blank();
    s.say('Chưa muốn giải quyết ngay: huỷ merge, repository trở lại như trước');
    s.git('merge --abort');
    s.git('status --short --branch');
    s.blank();
    s.say('Merge lại, lần này giải quyết: giữ cả làm tròn 0,1 km lẫn hệ số giờ cao điểm');
    s.git(`merge ${FEATURE}`, { ...AT, expectFail: true, quiet: true });
    s.print(`$ git merge ${FEATURE}`);
    s.write('src/BaoGia/FareCalculator.cs', CALC_MERGED);
    s.check('không còn dấu xung đột trong file', !/^(<<<<<<<|=======|>>>>>>>)/m.test(s.read('src/BaoGia/FareCalculator.cs')));
    s.git('add src/BaoGia/FareCalculator.cs');
    s.git('status --short');
    s.git('merge --continue', AT);
    s.git('log --oneline --graph -6');
    s.blank();
    s.say('Xem lại cách đã giải quyết xung đột');
    s.git('show --remerge-diff --format=%h%x20%s HEAD');
    s.git(`branch -d ${FEATURE}`);
    checkRefs(s, refs2(6));
  },
};
