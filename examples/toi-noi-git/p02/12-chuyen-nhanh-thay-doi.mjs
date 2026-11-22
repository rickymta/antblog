// GIT-012 · Chuyển nhánh khi còn thay đổi chưa commit

import { CALC_PEAK, CALC_ROUND_KM, FEATURE, SETTINGS_PEAK, TESTS_ROUND_KM } from '../lib/history2.mjs';
import { writeText } from '../lib/lab.mjs';
import { checkRefs } from './chung.mjs';

export default {
  id: 'GIT-012',
  start: 1,
  end: 3,
  prepare(dir) {
    // Đầu bài trong GitStudio: đang sửa dở FareCalculator.cs trên nhánh tính năng.
    writeText(dir, 'src/BaoGia/FareCalculator.cs', CALC_PEAK);
  },
  run(s, { refs2 }) {
    s.say('Minh đang sửa dở FareCalculator.cs trên nhánh tính năng');
    s.write('src/BaoGia/FareCalculator.cs', CALC_PEAK);
    s.git('status --short --branch');
    s.blank();
    s.say('Chuyển sang main: Git cho phép và MANG THEO thay đổi (file này giống nhau ở hai nhánh)');
    s.git('switch main');
    s.git('status --short --branch');
    s.git(`switch ${FEATURE}`);
    s.blank();
    s.say('Sửa thêm appsettings.json (file khác nhau giữa hai nhánh): Git từ chối chuyển');
    s.write('src/BaoGia/appsettings.json', SETTINGS_PEAK.replace('"Multiplier": 1.2', '"Multiplier": 1.25'));
    s.git('switch main', { expectFail: true });
    s.git('restore src/BaoGia/appsettings.json');
    s.blank();
    s.say('Thay đổi FareCalculator.cs đã xong: commit trên nhánh tính năng rồi mới chuyển');
    s.git('add src/BaoGia/FareCalculator.cs');
    s.git('commit -m "Nhân hệ số giờ cao điểm khi tính cước"', { as: 'minh', at: '2026-11-16T10:30:00+07:00' });
    s.git('switch main');
    s.git('status --short --branch');
    s.blank();
    s.say('Sửa gấp trên main: làm tròn quãng đường tới 0,1 km');
    s.write('src/BaoGia/FareCalculator.cs', CALC_ROUND_KM);
    s.write('tests/BaoGia.Tests/FareCalculatorTests.cs', TESTS_ROUND_KM);
    s.git('add src tests');
    s.git(
      'commit -m "Làm tròn quãng đường tới 0,1 km trước khi tính cước" -m "GPS trả quãng đường tới mét; khách thấy giá nhảy khi xe đứng yên."',
      { as: 'minh', at: '2026-11-16T11:15:00+07:00' },
    );
    s.git('log --oneline --graph --all -4');
    checkRefs(s, refs2(3));
  },
};
