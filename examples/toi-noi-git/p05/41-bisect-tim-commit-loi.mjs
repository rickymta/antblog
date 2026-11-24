// GIT-041 · Bisect: tìm commit làm sai cước

import { checkRefs5 } from './chung.mjs';

export default {
  id: 'GIT-041',
  start: 1,
  end: 2,
  run(s, { refs5 }) {
    s.say('Báo lỗi: chuyến 3,7 km (30.500 đ) phải làm tròn lên 31.000 đ, app báo 30.000 đ.');
    s.say('Bản 1.1.0 còn đúng. Từ đó tới nay có bao nhiêu commit?');
    s.git('rev-list --count v1.1.0..main');
    s.git('grep -n "atLeastMinimum / 1000m" v1.1.0 -- src');
    s.git('grep -n "atLeastMinimum / 1000m" main -- src');
    s.blank();
    s.say('Bắt đầu bisect: main lỗi, v1.1.0 tốt. Git checkout commit ở giữa để thử.');
    s.git('bisect start');
    s.git('bisect bad main');
    s.git('bisect good v1.1.0');
    s.say('Tự kiểm tra commit Git chọn: dòng làm tròn còn Math.Ceiling không?');
    s.git('grep -n "atLeastMinimum / 1000m" -- src');
    const ok = s.read('src/BaoGia/FareCalculator.cs').includes('Math.Ceiling(atLeastMinimum');
    s.git(`bisect ${ok ? 'good' : 'bad'}`);
    s.blank();
    s.say('Phần còn lại để Git tự chạy: mã thoát 0 = tốt, khác 0 = lỗi');
    s.git('bisect run sh ../../p05/kiem-tra-lam-tron.sh');
    s.git('bisect log');
    s.git('bisect reset');
    s.blank();
    s.say('Commit lỗi đã có trên main và đã push: hoàn tác bằng revert (GIT-033)');
    const bad = s.value('log --format=%h --grep="Làm gọn phép làm tròn" main');
    s.git(`revert --no-edit ${bad}`, { as: 'minh', at: '2026-11-26T14:00:00+07:00' });
    s.git('grep -n "atLeastMinimum / 1000m" -- src');
    s.git('push -q');
    s.check('main làm tròn lên trở lại', s.read('src/BaoGia/FareCalculator.cs').includes('Math.Ceiling(atLeastMinimum'));
    checkRefs5(s, refs5(2));
  },
};
