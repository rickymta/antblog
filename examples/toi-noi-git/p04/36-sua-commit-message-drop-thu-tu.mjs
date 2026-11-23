// GIT-036 · Sửa commit: sửa nội dung, bỏ commit, đổi thứ tự

import { NIGHT_BRANCH, todoEditor } from '../lib/history4.mjs';
import { checkRefs4, short } from './chung.mjs';

const C = (t) => ({ committedAt: `2026-11-24T${t}:00+07:00` });

export default {
  id: 'GIT-036',
  start: 2,
  end: 3,
  run(s, { refs4 }) {
    s.say(`Nhánh ${NIGHT_BRANCH} sau một buổi chiều: tên sai chính tả, commit debug, commit sửa nằm cách xa commit gốc`);
    s.git('log --oneline main..HEAD');
    s.blank();
    s.say('Sửa nội dung một commit cũ (cách GitStudio làm): commit "amend!" rỗng, rồi rebase --autosquash');
    s.git('commit --allow-empty --only -m "amend! Thêm phụ phi đêm" -m "Thêm phụ phí đêm"', { as: 'minh', at: '2026-11-24T16:00:00+07:00' });
    s.git('rebase -i --autosquash main', { ...C('16:00'), env: todoEditor() });
    s.git('log --oneline main..HEAD');
    s.blank();
    s.say('Bỏ commit debug: rebase --onto <cha của nó> <nó> chép lại các commit phía sau, bỏ đúng commit đó');
    const debug = short(s, 'HEAD~2');
    s.git(`rebase --onto ${debug}~1 ${debug}`, C('16:10'));
    s.git('log --oneline main..HEAD');
    s.git('grep -n Console.WriteLine -- src', { expectFail: true });
    s.blank();
    s.say('Đổi thứ tự: đưa commit sửa giờ lên ngay sau commit đầu (đổi chỗ hai dòng trong todo)');
    const test = short(s, 'HEAD~1');
    const fix = short(s, 'HEAD');
    s.git('rebase -i HEAD~2', {
      ...C('16:20'),
      env: todoEditor(`s/^pick ${test} /pick TEST /`, `s/^pick ${fix} /pick ${test} /`, `s/^pick TEST /pick ${fix} /`),
    });
    s.git('log --oneline main..HEAD');
    checkRefs4(s, refs4(3));
  },
};
