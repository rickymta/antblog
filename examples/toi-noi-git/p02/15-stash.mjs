// GIT-015 · Stash: cất thay đổi dở dang

import { TESTS_PEAK } from '../lib/history2.mjs';
import { writeText } from '../lib/lab.mjs';
import { checkRefs } from './chung.mjs';

const at = (t) => ({ as: 'minh', at: `2026-11-17T${t}:00+07:00` });

export default {
  id: 'GIT-015',
  start: 6,
  end: 7,
  prepare(dir) {
    writeText(dir, 'tests/BaoGia.Tests/FareCalculatorTests.cs', TESTS_PEAK);
    writeText(dir, 'ghi-chu-test.md', '- [ ] thêm test ngoài giờ cao điểm\n');
  },
  run(s, { refs2 }) {
    s.say('Minh đang viết test giờ cao điểm, kèm một file ghi chú chưa theo dõi');
    s.write('tests/BaoGia.Tests/FareCalculatorTests.cs', TESTS_PEAK);
    s.write('ghi-chu-test.md', '- [ ] thêm test ngoài giờ cao điểm\n');
    s.git('status --short');
    s.blank();
    s.say('Lan báo một test cũ đang đỏ. Cần chạy test trên code sạch: cất thay đổi đi');
    s.git('stash push -m "Test phụ phí giờ cao điểm dở dang"', at('09:40'));
    s.git('status --short');
    s.git('stash list');
    s.git('stash show --stat stash@{0}');
    s.blank();
    s.say('Kiểm tra xong, lấy thay đổi về và xoá stash');
    s.git('stash pop', { quiet: true });
    s.print('$ git stash pop');
    s.git('status --short');
    s.git('stash list');
    s.check('stash đã được xoá sau pop', s.value('stash list') === '');
    s.blank();
    s.say('Stash cả file chưa theo dõi (-u), rồi apply: stash vẫn còn');
    s.git('stash push -u -m "Kèm ghi chú"', at('09:50'));
    s.git('status --short');
    s.git('stash apply', { quiet: true });
    s.print('$ git stash apply');
    s.git('stash list');
    s.git('stash drop', { quiet: true });
    s.print('$ git stash drop');
    s.blank();
    s.say('Chỉ cất phần đã stage (--staged)');
    s.git('add tests/BaoGia.Tests/FareCalculatorTests.cs');
    s.git('stash push --staged -m "Chỉ phần đã stage"', at('10:00'));
    s.git('status --short');
    s.git('stash pop --index', { quiet: true });
    s.print('$ git stash pop --index');
    s.git('status --short');
    s.blank();
    s.say('Commit test, bỏ file ghi chú');
    s.remove('ghi-chu-test.md');
    s.git('commit -m "Thêm test cho phụ phí giờ cao điểm"', at('10:20'));
    s.check('không còn stash', s.value('stash list') === '');
    checkRefs(s, refs2(7));
  },
};
