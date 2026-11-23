// GIT-032 · Reset soft, mixed, hard

import { checkRefs4 } from './chung.mjs';

const M = (t) => ({ as: 'minh', at: `2026-11-24T${t}:00+07:00` });

export default {
  id: 'GIT-032',
  start: 0,
  end: 0,
  run(s, { refs4 }) {
    s.say('Nhánh thử thu/reset với ba commit, đã push');
    s.git('switch -c thu/reset');
    s.write('docs/thu-reset.md', 'Dòng 1\n');
    s.git('add docs/thu-reset.md');
    s.git('commit -q -m "Thử reset: dòng 1"', M('08:00'));
    s.write('docs/thu-reset.md', 'Dòng 1\nDòng 2\n');
    s.git('commit -q -am "Thử reset: dòng 2"', M('08:10'));
    s.write('docs/thu-reset.md', 'Dòng 1\nDòng 2\nDòng 3\n');
    s.git('commit -q -am "Thử reset: dòng 3"', M('08:20'));
    s.git('push -q -u origin thu/reset');
    s.git('log --oneline -4');
    s.blank();
    s.say('--soft: nhánh lùi một commit, thay đổi của commit đó nằm lại trong staging area');
    s.git('reset --soft HEAD~1');
    s.git('status -sb');
    s.git('log --oneline -1');
    s.blank();
    s.say('Commit lại (thành commit mới), rồi --mixed (mặc định): thay đổi nằm lại trong working tree, chưa stage');
    s.git('commit -q -m "Thử reset: dòng 3 (làm lại)"', M('08:30'));
    s.git('reset HEAD~1');
    s.git('status -sb');
    s.blank();
    s.say('--hard: bỏ luôn thay đổi trong staging area và working tree');
    s.git('reset --hard HEAD');
    s.git('status -sb');
    s.git('show HEAD:docs/thu-reset.md');
    s.blank();
    s.say('Nhánh đã push mà lùi lại: push thường bị từ chối');
    s.git('push', { expectFail: true });
    s.blank();
    s.say('Dọn nhánh thử');
    s.git('switch -q main');
    s.git('push -q origin --delete thu/reset');
    s.git('branch -q -D thu/reset');
    checkRefs4(s, refs4(0));
  },
};
