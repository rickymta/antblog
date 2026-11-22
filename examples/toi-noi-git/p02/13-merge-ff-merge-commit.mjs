// GIT-013 · Merge: fast-forward và merge commit

import { FEATURE, PRICE_DOC_PEAK, PRICE_DOC_ROUND } from '../lib/history2.mjs';
import { checkRefs } from './chung.mjs';

export default {
  id: 'GIT-013',
  start: 3,
  end: 5,
  run(s, { refs2 }) {
    s.say('Lan sửa tài liệu trên một nhánh ngắn');
    s.git('switch -c sua/tai-lieu-lam-tron');
    s.write('docs/bang-gia-cuoc.md', PRICE_DOC_ROUND);
    s.git('commit -am "Ghi cách làm tròn quãng đường vào tài liệu"', { as: 'lan', at: '2026-11-16T14:00:00+07:00' });
    s.git('switch main');
    s.blank();
    s.say('main chưa đi tiếp từ chỗ nhánh tách ra: merge chỉ cần dời main lên (fast-forward)');
    s.git('merge sua/tai-lieu-lam-tron');
    s.git('branch -d sua/tai-lieu-lam-tron');
    s.git('log --oneline --graph -3');
    s.blank();
    s.say('Nhánh tài liệu thứ hai, merge bằng --no-ff để giữ dấu vết nhánh trong lịch sử');
    s.git('switch -c tai-lieu/gio-cao-diem');
    s.write('docs/bang-gia-cuoc.md', PRICE_DOC_PEAK);
    s.git('commit -am "Mô tả phụ phí giờ cao điểm trong tài liệu"', { as: 'lan', at: '2026-11-16T15:00:00+07:00' });
    s.git('switch main');
    s.git('merge --no-ff --no-edit tai-lieu/gio-cao-diem', { as: 'minh', at: '2026-11-16T15:30:00+07:00' });
    s.git('log --oneline --graph -4');
    s.git('cat-file -p HEAD');
    const parents = s.value('rev-list --parents -n 1 HEAD').split(' ').length - 1;
    s.check('merge commit có hai commit cha', parents === 2);
    s.git('branch -d tai-lieu/gio-cao-diem');
    s.blank();
    s.say('Nhánh nào đã/chưa merge vào main');
    s.git('branch --merged main');
    s.git('branch --no-merged main');
    s.check('nhánh tính năng chưa merge', s.value('branch --no-merged main').includes(FEATURE));
    checkRefs(s, refs2(5));
  },
};
