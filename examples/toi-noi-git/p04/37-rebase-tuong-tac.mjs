// GIT-037 · Rebase tương tác: squash, fixup, reword, --autosquash

import { NIGHT_TESTS_22H, todoEditor } from '../lib/history4.mjs';
import { checkRefs4, short } from './chung.mjs';

export default {
  id: 'GIT-037',
  start: 3,
  end: 4,
  run(s, { refs4 }) {
    s.git('log --oneline main..HEAD');
    s.blank();
    s.say('Thêm ca biên 22:00 vào test, ghi thành commit fixup! cho commit test');
    s.write('tests/BaoGia.Tests/NightSurchargeTests.cs', NIGHT_TESTS_22H(s.read('tests/BaoGia.Tests/NightSurchargeTests.cs')));
    s.git('commit -a --fixup HEAD', { as: 'minh', at: '2026-11-25T09:00:00+07:00' });
    s.git('log --oneline main..HEAD');
    s.blank();
    s.say('rebase -i --autosquash main: Git tự xếp fixup! ngay dưới commit đích;');
    s.say('ta đổi thêm commit sửa giờ thành fixup để gộp vào commit đầu (dòng in ra là todo Git đề xuất)');
    const fix = short(s, 'HEAD~2');
    s.git('rebase -i --autosquash main', { committedAt: '2026-11-25T09:10:00+07:00', env: todoEditor(`s/^pick ${fix} /fixup ${fix} /`) });
    s.git('log --format="%h %ai %s" main..HEAD');
    s.git('show --stat --format="%h %s" HEAD~1');
    s.git('diff --stat main');
    s.check('nhánh còn đúng 2 commit', s.value('rev-list --count main..HEAD') === '2');
    checkRefs4(s, refs4(4));
  },
};
