// GIT-052 · Phát hành theo semver và nhánh phát hành

import { HOLIDAY_FIX, HOLIDAY_TEST_ROUND, addChangelogEntry } from '../lib/history6.mjs';
import { checkRefs6, sh } from './chung.mjs';

const M = (t) => ({ as: 'minh', at: `2026-12-01T${t}:00+07:00` });

export default {
  id: 'GIT-052',
  start: 1,
  end: 2,
  run(s, { refs6 }) {
    s.say('Từ 1.2.0 có một tính năng mới (feat) và một sửa lỗi: tăng MINOR → 1.3.0');
    s.git('tag -l "v*" --sort=-version:refname');
    s.git('tag -s v1.3.0 -m "Phát hành 1.3.0: phụ phí ngày lễ"', M('09:00'));
    s.git('tag -v v1.3.0');
    s.git('push -q origin main v1.3.0');
    s.blank();
    s.say('Nhánh phát hành: nơi vá bản 1.3.x trong khi main đi tiếp');
    s.git('switch -c phat-hanh/1.3 v1.3.0');
    s.git('push -q -u origin phat-hanh/1.3');
    s.blank();
    s.say('Lỗi trong 1.3.0: phụ phí ngày lễ ra 39.650 đ, không làm tròn lên bội số 1.000 đ. Vá trên nhánh phát hành');
    s.write('src/BaoGia/HolidaySurcharge.cs', HOLIDAY_FIX(s.read('src/BaoGia/HolidaySurcharge.cs')));
    s.write('tests/BaoGia.Tests/HolidaySurchargeTests.cs', HOLIDAY_TEST_ROUND(s.read('tests/BaoGia.Tests/HolidaySurchargeTests.cs')));
    s.git('commit -q -am "fix(cuoc): làm tròn phụ phí ngày lễ lên bội số 1.000 đ"', { as: 'lan', by: 'minh', at: '2026-12-01T10:00:00+07:00' });
    s.say('Chỉ có fix: tăng PATCH → 1.3.1');
    const entry = sh(s, ['scripts/changelog.sh', '1.3.1', '2026-12-01', 'v1.3.0']).out;
    s.write('CHANGELOG.md', addChangelogEntry(s.read('CHANGELOG.md'), entry));
    s.git('commit -q -am "chore(release): 1.3.1"', M('10:45'));
    s.git('tag -s v1.3.1 -m "Phát hành 1.3.1: làm tròn phụ phí ngày lễ"', M('11:00'));
    s.git('push -q origin phat-hanh/1.3 v1.3.1');
    s.blank();
    s.say('Đưa bản vá và mục CHANGELOG về main');
    s.git('switch main');
    s.git('merge --no-ff --no-edit phat-hanh/1.3', M('11:15'));
    s.git('push -q');
    s.git('log --oneline --graph -5');
    s.blank();
    s.say('Phiên bản của một commit bất kỳ: tag gần nhất + số commit sau tag + mã commit');
    s.git('describe');
    s.git('describe v1.3.1');
    s.git('describe --tags --abbrev=0 main');
    s.git('branch -r --contains v1.3.1');
    s.git('tag --merged phat-hanh/1.3 --sort=-version:refname -l "v1.3*"');
    s.check('main chứa bản vá 1.3.1', s.read('src/BaoGia/HolidaySurcharge.cs').includes('Math.Ceiling(fare * (1 + Rate)'));
    checkRefs6(s, refs6(2));
  },
};
